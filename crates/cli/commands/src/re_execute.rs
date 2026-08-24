//! Re-execute blocks from database in parallel.
//!
//! Parent state for `--from N` is loaded at block `N - 1` via
//! [`StateProviderFactory::history_by_block_number`]. Mid-pipeline (Finish still 0, Headers ≫
//! Execution) that resolves to [`LatestStateProvider`] when `N - 1` equals the Execution
//! checkpoint (PlainState tip) — see `DatabaseProvider::try_into_history_at_block`.

use crate::common::{
    AccessRights, BlockchainProviderFor, CliComponentsBuilder, CliNodeComponents, CliNodeTypes,
    Environment, EnvironmentArgs,
};
use alloy_consensus::{transaction::TxHashRef, BlockHeader, TxReceipt};
use clap::Parser;
use eyre::WrapErr;
use reth_chainspec::{EthChainSpec, EthereumHardforks, Hardforks};
use reth_cli::chainspec::ChainSpecParser;
use reth_cli_util::cancellation::CancellationToken;
use reth_consensus::FullConsensus;
use reth_evm::{execute::Executor, ConfigureEvm};
use reth_primitives_traits::{format_gas_throughput, BlockBody, GotExpected};
use reth_provider::{
    BlockNumReader, BlockReader, ChainSpecProvider, DatabaseProviderFactory, ReceiptProvider,
    StageCheckpointReader, StaticFileProviderFactory, TransactionVariant,
};
use reth_revm::database::StateProviderDatabase;
use reth_stages::stages::calculate_gas_used_from_headers;
use reth_stages_types::StageId;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, task::JoinSet};
use tracing::*;

/// `reth re-execute` command
///
/// Re-execute blocks in parallel to verify historical sync correctness.
#[derive(Debug, Parser)]
pub struct Command<C: ChainSpecParser> {
    #[command(flatten)]
    env: EnvironmentArgs<C>,

    /// The height to start at.
    #[arg(long, default_value = "1")]
    from: u64,

    /// The height to end at (exclusive upper bound of the executed range is not used; this is the
    /// last block number **plus one** in the half-open loop `from..to`). Defaults to the highest
    /// available header on disk when Finish is still 0 (staged sync).
    #[arg(long)]
    to: Option<u64>,

    /// Number of tasks to run in parallel. Defaults to the number of available CPUs.
    #[arg(long)]
    num_tasks: Option<u64>,

    /// Continues with execution when an invalid block is encountered and collects these blocks.
    #[arg(long)]
    skip_invalid_blocks: bool,

    /// On post-execution validation failure, write a JSON receipt summary for the failing block
    /// (idx / status / gasUsed / cumulativeGasUsed / logCount / txHash) to this path.
    ///
    /// Intended for FLOW-X04 / PIPE-014 diffs against public `eth_getBlockReceipts`
    /// (see `files/harness-receipt-diff-21591154/`).
    #[arg(long, value_name = "PATH")]
    dump_receipts_on_fail: Option<std::path::PathBuf>,
}

impl<C: ChainSpecParser> Command<C> {
    /// Returns the underlying chain being used to run this command
    pub fn chain_spec(&self) -> Option<&Arc<C::ChainSpec>> {
        Some(&self.env.chain)
    }
}

impl<C: ChainSpecParser<ChainSpec: EthChainSpec + Hardforks + EthereumHardforks>> Command<C> {
    /// Execute `re-execute` command
    pub async fn execute<N>(self, components: impl CliComponentsBuilder<N>) -> eyre::Result<()>
    where
        N: CliNodeTypes<ChainSpec = C::ChainSpec>,
    {
        let Environment { provider_factory, .. } = self.env.init::<N>(AccessRights::RO)?;

        let blockchain =
            reth_provider::providers::BlockchainProvider::new(provider_factory.clone())?;
        let components = components(provider_factory.chain_spec(), blockchain);

        let min_block = self.from;
        let provider_ro = DatabaseProviderFactory::database_provider_ro(&provider_factory)?;
        // Finish checkpoint (= `best_block_number`) is 0 until the pipeline completes — do not
        // use it as the sole `--to` cap mid-sync (would clamp to 0 and underflow the range).
        let finish_tip = provider_ro.best_block_number()?;
        let headers_tip = provider_ro.last_block_number()?;
        let execution_tip = provider_ro
            .get_stage_checkpoint(StageId::Execution)?
            .map(|c| c.block_number)
            .unwrap_or(0);
        let available_tip = headers_tip.max(finish_tip).max(execution_tip);

        let mut max_block = if finish_tip > 0 { finish_tip } else { available_tip };
        if let Some(to) = self.to {
            if to > available_tip {
                warn!(
                    requested = to,
                    available_tip,
                    finish_tip,
                    headers_tip,
                    execution_tip,
                    "Requested --to is beyond headers/execution on disk; clamping"
                );
                max_block = available_tip;
            } else {
                max_block = to;
            }
        }

        if max_block <= min_block {
            eyre::bail!(
                "invalid re-execute range: --from {min_block} --to {max_block} \
                 (need to > from; finish_tip={finish_tip} headers_tip={headers_tip} \
                 execution_tip={execution_tip})"
            );
        }

        let num_tasks = self.num_tasks.unwrap_or_else(|| {
            std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(10)
        });

        let total_blocks = max_block - min_block;
        let total_gas = calculate_gas_used_from_headers(
            &provider_factory.static_file_provider(),
            min_block..=max_block.saturating_sub(1),
        )?;
        let num_tasks = num_tasks.min(total_blocks.max(1));
        let blocks_per_task = total_blocks / num_tasks;

        info!(
            target: "reth::cli",
            from = min_block,
            to = max_block,
            execution_tip,
            finish_tip,
            headers_tip,
            num_tasks,
            "Re-execute range (parent state at from-1; Latest if from-1 == Execution tip)"
        );

        let db_at = {
            let provider_factory = provider_factory.clone();
            move |block_number: u64| {
                StateProviderDatabase(
                    provider_factory.history_by_block_number(block_number).unwrap(),
                )
            }
        };

        let skip_invalid_blocks = self.skip_invalid_blocks;
        let dump_receipts_on_fail = self.dump_receipts_on_fail.clone();
        let (stats_tx, mut stats_rx) = mpsc::unbounded_channel();
        let (info_tx, mut info_rx) = mpsc::unbounded_channel();
        let cancellation = CancellationToken::new();
        let _guard = cancellation.drop_guard();

        let mut tasks = JoinSet::new();
        for i in 0..num_tasks {
            let start_block = min_block + i * blocks_per_task;
            let end_block =
                if i == num_tasks - 1 { max_block } else { start_block + blocks_per_task };

            // Spawn thread executing blocks
            let provider_factory = provider_factory.clone();
            let evm_config = components.evm_config().clone();
            let consensus = components.consensus().clone();
            let db_at = db_at.clone();
            let stats_tx = stats_tx.clone();
            let info_tx = info_tx.clone();
            let cancellation = cancellation.clone();
            let dump_receipts_on_fail = dump_receipts_on_fail.clone();
            tasks.spawn_blocking(move || {
                let mut executor = evm_config.batch_executor(db_at(start_block - 1));
                let mut executor_created = Instant::now();
                let executor_lifetime = Duration::from_secs(120);

                'blocks: for block in start_block..end_block {
                    if cancellation.is_cancelled() {
                        // exit if the program is being terminated
                        break
                    }

                    let block = provider_factory
                        .recovered_block(block.into(), TransactionVariant::NoHash)?
                        .ok_or_else(|| {
                            eyre::eyre!(
                                "missing recovered block {block} (body/senders not in database)"
                            )
                        })?;

                    let result = match executor.execute_one(&block) {
                        Ok(result) => result,
                        Err(err) => {
                            if skip_invalid_blocks {
                                executor = evm_config.batch_executor(db_at(block.number()));
                                let _ = info_tx.send((block, eyre::Report::new(err)));
                                continue
                            }
                            return Err(err.into())
                        }
                    };

                    if let Err(err) = consensus
                        .validate_block_post_execution(&block, &result, None, None)
                        .wrap_err_with(|| {
                            format!("Failed to validate block {} {}", block.number(), block.hash())
                        })
                    {
                        if let Some(path) = dump_receipts_on_fail.as_ref() {
                            if let Err(dump_err) =
                                dump_executed_receipts_summary(path, &block, &result.receipts)
                            {
                                error!(?dump_err, path = %path.display(), "Failed to dump receipts");
                            } else {
                                error!(
                                    path = %path.display(),
                                    number = block.number(),
                                    hash = %block.hash(),
                                    "Dumped executed receipts for FLOW-X04 / public RPC diff"
                                );
                            }
                        }

                        let correct_receipts =
                            provider_factory.receipts_by_block(block.number().into())?;

                        if let Some(correct_receipts) = correct_receipts {
                            for (i, (receipt, correct_receipt)) in
                                result.receipts.iter().zip(correct_receipts.iter()).enumerate()
                            {
                                if receipt != correct_receipt {
                                    let tx_hash = block.body().transactions()[i].tx_hash();
                                    error!(
                                        ?receipt,
                                        ?correct_receipt,
                                        index = i,
                                        ?tx_hash,
                                        "Invalid receipt"
                                    );
                                    let expected_gas_used = correct_receipt.cumulative_gas_used() -
                                        if i == 0 {
                                            0
                                        } else {
                                            correct_receipts[i - 1].cumulative_gas_used()
                                        };
                                    let got_gas_used = receipt.cumulative_gas_used() -
                                        if i == 0 {
                                            0
                                        } else {
                                            result.receipts[i - 1].cumulative_gas_used()
                                        };
                                    if got_gas_used != expected_gas_used {
                                        let mismatch = GotExpected {
                                            expected: expected_gas_used,
                                            got: got_gas_used,
                                        };

                                        error!(
                                            number=?block.number(),
                                            ?mismatch,
                                            "Gas usage mismatch"
                                        );
                                        if skip_invalid_blocks {
                                            executor =
                                                evm_config.batch_executor(db_at(block.number()));
                                            let _ = info_tx.send((block, err));
                                            continue 'blocks;
                                        }
                                        return Err(err);
                                    }
                                } else {
                                    continue;
                                }
                            }
                        } else {
                            warn!(
                                number = block.number(),
                                "No local receipts to compare; use --dump-receipts-on-fail + public RPC diff"
                            );
                        }

                        return Err(err);
                    }
                    let _ = stats_tx.send(block.gas_used());

                    // Reset DB once in a while to avoid OOM or read tx timeouts
                    if executor.size_hint() > 1_000_000 ||
                        executor_created.elapsed() > executor_lifetime
                    {
                        executor = evm_config.batch_executor(db_at(block.number()));
                        executor_created = Instant::now();
                    }
                }

                eyre::Ok(())
            });
        }

        let instant = Instant::now();
        let mut total_executed_blocks = 0;
        let mut total_executed_gas = 0;

        let mut last_logged_gas = 0;
        let mut last_logged_blocks = 0;
        let mut last_logged_time = Instant::now();
        let mut invalid_blocks = Vec::new();

        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                Some(gas_used) = stats_rx.recv() => {
                    total_executed_blocks += 1;
                    total_executed_gas += gas_used;
                }
                Some((block, err)) = info_rx.recv() => {
                    error!(?err, block=?block.num_hash(), "Invalid block");
                    invalid_blocks.push(block.num_hash());
                }
                result = tasks.join_next() => {
                    if let Some(result) = result {
                        if matches!(result, Err(_) | Ok(Err(_))) {
                            error!(?result);
                            return Err(eyre::eyre!("Re-execution failed: {result:?}"));
                        }
                    } else {
                        break;
                    }
                }
                _ = interval.tick() => {
                    let blocks_executed = total_executed_blocks - last_logged_blocks;
                    let gas_executed = total_executed_gas - last_logged_gas;

                    if blocks_executed > 0 {
                        let progress = 100.0 * total_executed_gas as f64 / total_gas as f64;
                        info!(
                            throughput=?format_gas_throughput(gas_executed, last_logged_time.elapsed()),
                            progress=format!("{progress:.2}%"),
                            "Executed {blocks_executed} blocks"
                        );
                    }

                    last_logged_blocks = total_executed_blocks;
                    last_logged_gas = total_executed_gas;
                    last_logged_time = Instant::now();
                }
            }
        }

        if invalid_blocks.is_empty() {
            info!(
                start_block = min_block,
                end_block = max_block,
                %total_executed_blocks,
                throughput=?format_gas_throughput(total_executed_gas, instant.elapsed()),
                "Re-executed successfully"
            );
        } else {
            info!(
                start_block = min_block,
                end_block = max_block,
                %total_executed_blocks,
                invalid_block_count = invalid_blocks.len(),
                ?invalid_blocks,
                throughput=?format_gas_throughput(total_executed_gas, instant.elapsed()),
                "Re-executed with invalid blocks"
            );
        }

        Ok(())
    }
}

/// Compact receipt rows for FLOW-X04 public-RPC diffs
/// (`files/harness-receipt-diff-*/diff_receipts.py`).
fn dump_executed_receipts_summary<B, R>(
    path: &std::path::Path,
    block: &reth_primitives_traits::RecoveredBlock<B>,
    receipts: &[R],
) -> eyre::Result<()>
where
    B: reth_primitives_traits::Block,
    R: TxReceipt,
{
    use alloy_primitives::hex;
    use std::{fs, io::Write};

    let mut prev_cum = 0u64;
    let mut rows = Vec::with_capacity(receipts.len());
    for (i, receipt) in receipts.iter().enumerate() {
        let cum = receipt.cumulative_gas_used();
        let gas_used = cum.saturating_sub(prev_cum);
        prev_cum = cum;
        let tx_hash = block.body().transactions().get(i).map(|tx| *tx.tx_hash());
        rows.push(serde_json::json!({
            "i": i,
            "txHash": tx_hash.map(|h| format!("0x{}", hex::encode(h))),
            "status": if receipt.status() { 1 } else { 0 },
            "gasUsed": gas_used,
            "cumulativeGasUsed": cum,
            "logCount": receipt.logs().len(),
        }));
    }

    let doc = serde_json::json!({
        "source": "op-reth re-execute --dump-receipts-on-fail",
        "blockNumber": block.number(),
        "blockHash": format!("0x{}", hex::encode(block.hash())),
        "receiptsRootHeader": format!("0x{}", hex::encode(block.receipts_root())),
        "receipts": rows,
    });

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    serde_json::to_writer_pretty(&mut f, &doc)?;
    f.write_all(b"\n")?;
    Ok(())
}
