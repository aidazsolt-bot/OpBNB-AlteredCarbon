//! Bsc block executor.

use core::fmt::{Debug, Display};
use std::{borrow::Cow, collections::HashMap, num::NonZeroUsize, sync::Arc};

use alloy_consensus::Transaction as _;
use alloy_primitives::{Address, BlockNumber, Bytes, B256, U256};
use lazy_static::lazy_static;
use lru::LruCache;
use parking_lot::RwLock;
use reth_bsc_chainspec::BscChainSpec;
use reth_bsc_consensus::{
    is_breathe_block, validate_block_post_execution_of_bsc, Parlia, ValidatorElectionInfo,
    ValidatorsInfo,
};
use reth_bsc_forks::BscHardforks;
use reth_bsc_primitives::system_contracts::{
    get_upgrade_system_contracts, is_system_transaction, SLASH_CONTRACT,
};
use reth_chainspec::{ChainSpec, EthereumHardforks};
use alloy_evm::{block::NoopHook, FromRecoveredTx};
use reth_errors::{BlockExecutionError, BlockValidationError, ProviderError};
use alloy_consensus::transaction::SignableTransaction;
use reth_evm::{ConfigureEvm, EvmEnv, OnStateHook};
use reth_primitives_traits::SignedTransaction;
use reth_primitives::{
    parlia::{ParliaConfig, Snapshot, VoteAddress, CHECKPOINT_INTERVAL, DEFAULT_TURN_LENGTH},
    BlockWithSenders, Header, Receipt, Transaction, TransactionSigned,
};
use reth_provider::{HeaderProvider, ParliaProvider};
use reth_revm::{db::states::bundle_state::BundleRetention, State};
use revm::{
    context::BlockEnv,
    context_interface::result::EVMError,
    inspector::NoOpInspector,
    primitives::hardfork::SpecId,
    state::{Account, EvmState, TransactionId},
    Database, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};
use reth_ethereum_forks::Head;
use revm::context::TxEnv;
use revm::primitives::TxKind;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

use crate::{
    bsc_evm::api::BscEvm,
    config::revm_spec,
    post_execution::PostExecutionInput,
    transaction::BscTxEnv,
    BscBlockExecutionError, BscEvmConfig,
};

fn set_transaction_nonce(transaction: &mut Transaction, nonce: u64) {
    match transaction {
        Transaction::Legacy(tx) => tx.nonce = nonce,
        Transaction::Eip2930(tx) => tx.nonce = nonce,
        Transaction::Eip1559(tx) => tx.nonce = nonce,
        Transaction::Eip4844(tx) => tx.nonce = nonce,
        Transaction::Eip7702(tx) => tx.nonce = nonce,
    }
}

const SNAP_CACHE_NUM: usize = 2048;

lazy_static! {
    // snapshot cache map by block_hash: snapshot
    static ref RECENT_SNAPS: RwLock<LruCache<B256, Snapshot>> = RwLock::new(LruCache::new(NonZeroUsize::new(SNAP_CACHE_NUM).unwrap()));
}

/// Provides executors to execute regular bsc blocks
#[derive(Debug, Clone)]
pub struct BscExecutorProvider<P> {
    chain_spec: Arc<BscChainSpec>,
    evm_config: BscEvmConfig,
    parlia_config: ParliaConfig,
    provider: P,
}

impl<P> BscExecutorProvider<P> {
    /// Creates a new default bsc executor provider.
    pub fn bsc(chain_spec: Arc<BscChainSpec>, provider: P) -> Self {
        Self::new(chain_spec.clone(), BscEvmConfig::new(chain_spec), Default::default(), provider)
    }
}

impl<P> BscExecutorProvider<P> {
    /// Creates a new executor provider.
    pub const fn new(
        chain_spec: Arc<BscChainSpec>,
        evm_config: BscEvmConfig,
        parlia_config: ParliaConfig,
        provider: P,
    ) -> Self {
        Self { chain_spec, evm_config, parlia_config, provider }
    }
}

impl<P> BscExecutorProvider<P>
where
    P: Clone,
{
    fn bsc_executor<DB>(
        &self,
        db: DB,
        prefetch_tx: Option<UnboundedSender<EvmState>>,
    ) -> BscBlockExecutor<DB, P>
    where
        DB: Database<Error: Into<ProviderError> + Display>,
    {
        if let Some(tx) = prefetch_tx {
            BscBlockExecutor::new_with_prefetch_tx(
                self.chain_spec.clone(),
                self.evm_config.clone(),
                self.parlia_config.clone(),
                State::builder()
                    .with_database(db)
                    .with_bundle_update()
                    .build(),
                self.provider.clone(),
                tx,
            )
        } else {
            BscBlockExecutor::new(
                self.chain_spec.clone(),
                self.evm_config.clone(),
                self.parlia_config.clone(),
                State::builder()
                    .with_database(db)
                    .with_bundle_update()
                    .build(),
                self.provider.clone(),
            )
        }
    }
}

/// Helper type for the output of executing a block.
#[derive(Debug, Clone)]
pub(crate) struct BscExecuteOutput {
    receipts: Vec<Receipt>,
    gas_used: u64,
    snapshot: Option<Snapshot>,
}

/// Helper container type for EVM with chain spec.
#[derive(Debug, Clone)]
pub(crate) struct BscEvmExecutor {
    /// The chain spec
    chain_spec: Arc<BscChainSpec>,
    /// How to create an EVM.
    evm_config: BscEvmConfig,
}

impl BscEvmExecutor {
    /// Executes the transactions in the block and returns the receipts.
    ///
    /// This applies the pre-execution changes, and executes the transactions.
    ///
    /// The optional `state_hook` is unused for now.
    ///
    /// # Note
    ///
    /// It does __not__ apply post-execution changes.
    fn execute_pre_and_transactions<DB, F>(
        &self,
        block: &BlockWithSenders,
        state: &mut State<DB>,
        env: EvmEnv<SpecId>,
        _state_hook: Option<F>,
        prefetch_tx: Option<UnboundedSender<EvmState>>,
    ) -> Result<(Vec<TransactionSigned>, Vec<Receipt>, u64), BlockExecutionError>
    where
        DB: Database<Error: Into<ProviderError> + Display> + DatabaseCommit + Debug,
        F: OnStateHook,
        BscTxEnv: FromRecoveredTx<TransactionSigned>,
    {
        // execute transactions
        let mut cumulative_gas_used = 0;
        let mut system_txs = Vec::with_capacity(2); // Normally there are 2 system transactions.
        let mut receipts = Vec::with_capacity(block.body().transactions.len());
        for (sender, transaction) in block.transactions_with_sender() {
            if is_system_transaction(transaction, *sender, block.beneficiary) {
                system_txs.push(transaction.clone());
                continue;
            }
            // systemTxs should be always at the end of block.
            if self.chain_spec.is_cancun_active_at_timestamp(block.timestamp)
                && !system_txs.is_empty()
            {
                return Err(BscBlockExecutionError::UnexpectedNormalTx.into());
            }

            // The sum of the transaction’s gas limit, Tg, and the gas utilized in this block prior,
            // must be no greater than the block’s gasLimit.
            let block_available_gas = block.header().gas_limit - cumulative_gas_used;
            if transaction.gas_limit() > block_available_gas {
                return Err(BlockValidationError::TransactionGasLimitMoreThanAvailableBlockGas {
                    transaction_gas_limit: transaction.gas_limit(),
                    block_available_gas,
                }
                .into());
            }

            self.patch_mainnet_before_tx(transaction, &mut *state);
            self.patch_chapel_before_tx(transaction, &mut *state);

            let tx = BscTxEnv::from_recovered_tx(transaction, *sender);

            // Execute transaction.
            let mut evm = self.evm_config.evm_with_env(&mut *state, env.clone());
            let result = evm.transact_one(tx).map_err(|err| BlockValidationError::EVM {
                hash: *transaction.hash(),
                error: Box::new(err),
            })?;
            let tx_state = evm.finalize();

            if let Some(prefetch_tx) = prefetch_tx.as_ref() {
                prefetch_tx.send(tx_state.clone()).unwrap_or_else(|err| {
                    debug!(target: "evm_executor", ?err, "Failed to send post state to prefetch channel")
                });
            }

            evm.commit(tx_state);

            self.patch_mainnet_after_tx(transaction, &mut *state);
            self.patch_chapel_after_tx(transaction, &mut *state);

            // append gas used
            cumulative_gas_used += result.gas_used();

            // Push transaction changeset and calculate header bloom filter for receipt.
            receipts.push(Receipt {
                tx_type: transaction.tx_type(),
                // Success flag was added in `EIP-658: Embedding transaction status code in
                // receipts`.
                success: result.is_success(),
                cumulative_gas_used,
                // convert to reth log
                logs: result.into_logs(),
            });
        }

        Ok((system_txs, receipts, cumulative_gas_used))
    }
}

/// A basic Bsc block executor.
///
/// Expected usage:
/// - Create a new instance of the executor.
/// - Execute the block.
pub struct BscBlockExecutor<DB, P> {
    /// Chain specific evm config that's used to execute a block.
    executor: BscEvmExecutor,
    /// The state to use for execution
    pub(crate) state: State<DB>,
    /// Extra provider for bsc
    pub(crate) provider: Arc<P>,
    /// Parlia consensus instance
    pub(crate) parlia: Arc<Parlia>,
    /// Prefetch channel
    prefetch_tx: Option<UnboundedSender<EvmState>>,
}

impl<DB, P> BscBlockExecutor<DB, P> {
    /// Creates a new Parlia block executor.
    pub fn new(
        chain_spec: Arc<BscChainSpec>,
        evm_config: BscEvmConfig,
        parlia_config: ParliaConfig,
        state: State<DB>,
        provider: P,
    ) -> Self {
        let parlia = Arc::new(Parlia::new(Arc::clone(&chain_spec), parlia_config));
        let shared_provider = Arc::new(provider);
        Self {
            executor: BscEvmExecutor { chain_spec, evm_config },
            state,
            provider: shared_provider,
            parlia,
            prefetch_tx: None,
        }
    }

    /// Creates a new BSC block executor with a prefetch channel.
    pub fn new_with_prefetch_tx(
        chain_spec: Arc<BscChainSpec>,
        evm_config: BscEvmConfig,
        parlia_config: ParliaConfig,
        state: State<DB>,
        provider: P,
        tx: UnboundedSender<EvmState>,
    ) -> Self {
        let parlia = Arc::new(Parlia::new(Arc::clone(&chain_spec), parlia_config));
        let shared_provider = Arc::new(provider);
        Self {
            executor: BscEvmExecutor { chain_spec, evm_config },
            state,
            provider: shared_provider,
            parlia,
            prefetch_tx: Some(tx),
        }
    }

    #[inline]
    pub(crate) fn chain_spec(&self) -> &ChainSpec {
        &self.executor.chain_spec
    }

    #[allow(unused)]
    #[inline]
    pub(crate) fn parlia(&self) -> &Parlia {
        &self.parlia
    }

    /// Returns mutable reference to the state that wraps the underlying database.
    #[allow(unused)]
    fn state_mut(&mut self) -> &mut State<DB> {
        &mut self.state
    }
}

impl<DB, P> BscBlockExecutor<DB, P>
where
    DB: Database<Error: Into<ProviderError> + Display> + DatabaseCommit + Debug,
    P: ParliaProvider + HeaderProvider<Header = Header>,
{
    /// Configures a new evm configuration and block environment for the given block.
    ///
    /// Caution: this does not initialize the tx environment.
    fn evm_env_for_block(&self, header: &Header, total_difficulty: U256) -> EvmEnv<SpecId> {
        let mut env = self.executor.evm_config.evm_env(header).expect("infallible");
        let spec_id = revm_spec(
            self.chain_spec(),
            &Head {
                number: header.number,
                timestamp: header.timestamp,
                difficulty: header.difficulty,
                total_difficulty,
                hash: Default::default(),
            },
        );
        env.cfg_env.spec = spec_id;
        if spec_id >= SpecId::MERGE {
            env.block_env.difficulty = U256::ZERO;
            env.block_env.prevrandao = Some(header.difficulty.into());
        } else {
            env.block_env.difficulty = header.difficulty;
            env.block_env.prevrandao = None;
        }
        env
    }

    /// Convenience method to invoke `execute_without_verification_with_state_hook` setting the
    /// state hook as `None`.
    fn execute_without_verification(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
    ) -> Result<BscExecuteOutput, BlockExecutionError> {
        self.execute_without_verification_with_state_hook(
            block,
            total_difficulty,
            ancestor,
            Option::<NoopHook>::None,
        )
    }

    /// Execute a single block and apply the state changes to the internal state.
    ///
    /// Returns the receipts of the transactions in the block and the total gas used.
    ///
    /// Returns an error if execution fails.
    fn execute_without_verification_with_state_hook<F>(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
        state_hook: Option<F>,
    ) -> Result<BscExecuteOutput, BlockExecutionError>
    where
        F: OnStateHook,
    {
        // 1. get parent header and snapshot
        let parent = &(self.get_header_by_hash(block.parent_hash, ancestor)?);
        let snapshot_reader = SnapshotReader::new(self.provider.clone(), self.parlia.clone());
        let snap = &(snapshot_reader.snapshot(parent, ancestor)?);

        // 2. prepare state on new block
        self.on_new_block(block.header(), parent, ancestor, snap)?;

        // 3. get data from contracts before execute transactions
        let post_execution_input =
            self.do_system_call_before_execution(block.header(), total_difficulty, parent)?;

        // 4. execute normal transactions
        let env = self.evm_env_for_block(block.header(), total_difficulty);

        if !self.chain_spec().is_feynman_active_at_timestamp(block.timestamp) {
            // apply system contract upgrade
            self.upgrade_system_contracts(block.number, block.timestamp, parent.timestamp)?;
        }

        let (mut system_txs, mut receipts, mut gas_used) =
            self.executor.execute_pre_and_transactions(
                block,
                &mut self.state,
                env.clone(),
                state_hook,
                self.prefetch_tx.clone(),
            )?;

        // 5. apply post execution changes
        self.post_execution(
            block,
            parent,
            ancestor,
            snap,
            post_execution_input,
            &mut system_txs,
            &mut receipts,
            &mut gas_used,
            env,
        )?;

        if snap.block_number % CHECKPOINT_INTERVAL == 0 {
            Ok(BscExecuteOutput { receipts, gas_used, snapshot: Some(snap.clone()) })
        } else {
            Ok(BscExecuteOutput { receipts, gas_used, snapshot: None })
        }
    }

    pub(crate) fn get_justified_header(
        &self,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
        snap: &Snapshot,
    ) -> Result<Header, BlockExecutionError> {
        if snap.vote_data.source_hash == B256::ZERO && snap.vote_data.target_hash == B256::ZERO {
            return self
                .provider
                .header_by_number(0)
                .map_err(|err| BscBlockExecutionError::ProviderInnerError { error: err.into() })?
                .ok_or_else(|| {
                    BscBlockExecutionError::UnknownHeader { block_hash: B256::ZERO }.into()
                });
        }

        self.get_header_by_hash(snap.vote_data.target_hash, ancestor)
    }

    pub(crate) fn get_header_by_hash(
        &self,
        block_hash: B256,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
    ) -> Result<Header, BlockExecutionError> {
        ancestor
            .and_then(|m| m.get(&block_hash).cloned())
            .or_else(|| {
                self.provider
                    .header(block_hash)
                    .map_err(|err| BscBlockExecutionError::ProviderInnerError { error: err.into() })
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| BscBlockExecutionError::UnknownHeader { block_hash }.into())
    }

    /// Upgrade system contracts based on the hardfork rules.
    pub(crate) fn upgrade_system_contracts(
        &mut self,
        block_number: BlockNumber,
        block_time: u64,
        parent_block_time: u64,
    ) -> Result<bool, BscBlockExecutionError> {
        if let Ok(contracts) = get_upgrade_system_contracts(
            self.chain_spec(),
            block_number,
            block_time,
            parent_block_time,
        ) {
            for (k, v) in contracts {
                debug!("Upgrade system contract {:?} at height {:?}", k, block_number);

                let account = self.state.load_cache_account(k).map_err(|err| {
                    BscBlockExecutionError::ProviderInnerError { error: Box::new(err.into()) }
                })?;

                let mut new_info = account.account_info().unwrap_or_default();
                new_info.code_hash = v.clone().unwrap().hash_slow();
                new_info.code = v;
                let transition = account.change(Cow::Owned(
                    Account::new_not_existing(TransactionId::default()).with_info(new_info),
                ));

                self.state.apply_transition(vec![(k, transition)]);
            }

            Ok(true)
        } else {
            Err(BscBlockExecutionError::SystemContractUpgradeError)
        }
    }

    pub(crate) fn eth_call(
        &mut self,
        to: Address,
        data: Bytes,
        mut env: EvmEnv<SpecId>,
    ) -> Result<Bytes, BlockExecutionError> {
        env.block_env.basefee = 0;
        let mut evm = self.executor.evm_config.evm_with_env(&mut self.state, env);

        let tx = BscTxEnv {
            base: TxEnv::builder()
                .caller(Address::default())
                .kind(TxKind::Call(to))
                .gas_limit(u64::MAX / 2)
                .value(U256::ZERO)
                .data(data)
                .gas_price(0)
                .build()
                .map_err(|_| BscBlockExecutionError::EthCallFailed)?,
            is_system_transaction: false,
        };

        // Execute call.
        let result = evm.transact_one(tx).map_err(|err| {
            BlockValidationError::EVM { hash: B256::default(), error: Box::new(err) }
        })?;

        if !result.is_success() {
            return Err(BscBlockExecutionError::EthCallFailed.into());
        }

        let output = result.output().ok_or(BscBlockExecutionError::EthCallFailed)?;
        Ok(output.clone())
    }

    pub(crate) fn transact_system_tx(
        &mut self,
        mut transaction: Transaction,
        sender: Address,
        system_txs: &mut Vec<TransactionSigned>,
        receipts: &mut Vec<Receipt>,
        cumulative_gas_used: &mut u64,
        mut env: EvmEnv<SpecId>,
    ) -> Result<(), BlockExecutionError> {
        env.block_env.basefee = 0;

        let nonce = self
            .state
            .basic(sender)
            .map_err(|err| BscBlockExecutionError::ProviderInnerError {
                error: Box::new(ProviderError::other(err)),
            })?
            .unwrap_or_default()
            .nonce;
        set_transaction_nonce(&mut transaction, nonce);
        let mut evm = self.executor.evm_config.evm_with_env(&mut self.state, env);

        let hash = SignableTransaction::signature_hash(&transaction);
        if system_txs.is_empty() || hash != system_txs[0].signature_hash() {
            // slash tx could fail and not in the block
            if let Some(to) = transaction.to() {
                if to == SLASH_CONTRACT.parse::<Address>().unwrap()
                    && (system_txs.is_empty()
                        || system_txs[0].to().unwrap_or_default()
                            != SLASH_CONTRACT.parse::<Address>().unwrap())
                {
                    warn!("slash validator failed");
                    return Ok(());
                }
            }

            debug!("unexpected transaction: {:?}", transaction);
            for tx in system_txs.iter() {
                debug!("left system tx: {:?}", tx);
            }
            return Err(BscBlockExecutionError::UnexpectedSystemTx.into());
        }
        system_txs.remove(0);

        let tx = BscTxEnv {
            base: TxEnv::builder()
                .caller(sender)
                .kind(TxKind::Call(transaction.to().unwrap()))
                .nonce(nonce)
                .gas_limit(u64::MAX / 2)
                .value(transaction.value())
                .data(transaction.input().clone())
                .gas_price(0)
                .chain_id(transaction.chain_id())
                .build()
                .map_err(|_| BscBlockExecutionError::UnexpectedSystemTx)?,
            is_system_transaction: true,
        };

        // Execute transaction.
        let result = evm.transact_one(tx).map_err(|err| {
            BlockValidationError::EVM { hash, error: Box::new(err) }
        })?;
        let tx_state = evm.finalize();
        evm.commit(tx_state);

        // append gas used
        *cumulative_gas_used += result.gas_used();

        // Push transaction changeset and calculate header bloom filter for receipt.
        receipts.push(Receipt {
            tx_type: transaction.tx_type(),
            // Success flag was added in `EIP-658: Embedding transaction status code in
            // receipts`.
            success: result.is_success(),
            cumulative_gas_used: *cumulative_gas_used,
            // convert to reth log
            logs: result.into_logs().into_iter().map(Into::into).collect(),
        });

        Ok(())
    }

    fn do_system_call_before_execution(
        &mut self,
        header: &Header,
        total_difficulty: U256,
        parent: &Header,
    ) -> Result<PostExecutionInput, BlockExecutionError> {
        // env of parent state
        let env =
            self.evm_env_for_block(parent, total_difficulty.saturating_sub(header.difficulty));
        let mut output = PostExecutionInput {
            current_validators: None,
            max_elected_validators: None,
            validators_election_info: None,
        };

        // 1. get current validators info
        if header.number % self.parlia().epoch() == 0 {
            let (validators, vote_addrs) = self.get_current_validators(parent.number, env.clone());

            let vote_addrs_map = if vote_addrs.is_empty() {
                HashMap::new()
            } else {
                validators
                    .iter()
                    .copied()
                    .zip(vote_addrs)
                    .collect::<std::collections::HashMap<_, _>>()
            };

            output.current_validators = Some((validators, vote_addrs_map));
        };

        // 2. get election info
        if self.chain_spec().is_feynman_active_at_timestamp(header.timestamp)
            && is_breathe_block(parent.timestamp, header.timestamp)
            && !self.chain_spec().is_on_feynman_at_timestamp(header.timestamp, parent.timestamp)
        {
            let (to, data) = self.parlia().get_max_elected_validators();
            let bz = self.eth_call(to, data, env.clone())?;
            output.max_elected_validators =
                Some(self.parlia().unpack_data_into_max_elected_validators(bz.as_ref()));

            let (to, data) = self.parlia().get_validator_election_info();
            let bz = self.eth_call(to, data, env)?;

            let (validators, voting_powers, vote_addrs, total_length) =
                self.parlia().unpack_data_into_validator_election_info(bz.as_ref());

            let total_length = total_length.to::<u64>() as usize;
            if validators.len() != total_length
                || voting_powers.len() != total_length
                || vote_addrs.len() != total_length
            {
                return Err(BscBlockExecutionError::GetTopValidatorsFailed.into());
            }

            let validator_election_info = validators
                .into_iter()
                .zip(voting_powers)
                .zip(vote_addrs)
                .map(|((validator, voting_power), vote_addr)| ValidatorElectionInfo {
                    address: validator,
                    voting_power,
                    vote_address: vote_addr,
                })
                .collect();

            output.validators_election_info = Some(validator_election_info);
        }

        Ok(output)
    }

    fn get_current_validators(
        &mut self,
        number: BlockNumber,
        env: EvmEnv<SpecId>,
    ) -> (Vec<Address>, Vec<VoteAddress>) {
        if self.chain_spec().is_luban_active_at_block(number) {
            let (to, data) = self.parlia().get_current_validators();
            let output = self.eth_call(to, data, env).unwrap();

            self.parlia().unpack_data_into_validator_set(output.as_ref())
        } else {
            let (to, data) = self.parlia().get_current_validators_before_luban(number);
            let output = self.eth_call(to, data, env).unwrap();

            (self.parlia().unpack_data_into_validator_set_before_luban(output.as_ref()), Vec::new())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotReader<P> {
    /// Extra provider for bsc
    provider: Arc<P>,
    /// Parlia consensus instance
    parlia: Arc<Parlia>,
}

impl<P> SnapshotReader<P>
where
    P: ParliaProvider + HeaderProvider<Header = Header>,
{
    pub const fn new(provider: Arc<P>, parlia: Arc<Parlia>) -> Self {
        Self { provider, parlia }
    }

    pub fn snapshot(
        &self,
        header: &Header,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
    ) -> Result<Snapshot, BlockExecutionError> {
        let mut cache = RECENT_SNAPS.write();

        let mut header = header.clone();
        let mut block_number = header.number;
        let mut block_hash = header.hash_slow();
        let mut skip_headers = Vec::new();

        let snap: Option<Snapshot>;
        loop {
            // Read from cache
            if let Some(cached) = cache.get(&block_hash) {
                snap = Some(cached.clone());
                break;
            }

            // Read from db
            if block_number % CHECKPOINT_INTERVAL == 0 {
                if let Some(cached) =
                    self.provider.get_parlia_snapshot(block_hash).map_err(|err| {
                        BscBlockExecutionError::ProviderInnerError { error: err.into() }
                    })?
                {
                    snap = Some(cached);
                    break;
                }
            }

            // If we're at the genesis, snapshot the initial state.
            if block_number == 0 {
                let ValidatorsInfo { consensus_addrs, vote_addrs } =
                    self.parlia.parse_validators_from_header(&header).map_err(|err| {
                        BscBlockExecutionError::ParliaConsensusInnerError { error: err.into() }
                    })?;
                snap = Some(Snapshot::new(
                    consensus_addrs,
                    block_number,
                    block_hash,
                    self.parlia.epoch(),
                    vote_addrs,
                ));
                break;
            }

            // No snapshot for this header, gather the header and move backward
            skip_headers.push(header.clone());
            if let Ok(h) = self.get_header_by_hash(header.parent_hash, ancestor) {
                block_number = h.number;
                block_hash = header.parent_hash;
                header = h;
            } else {
                return Err(BscBlockExecutionError::UnknownHeader {
                    block_hash: header.parent_hash,
                }
                .into());
            }
        }

        let mut snap = snap.ok_or(BscBlockExecutionError::SnapshotNotFound)?;

        // the old snapshots don't have turn length, make sure we initialize it with default
        // before accessing it
        if snap.turn_length.is_none() || snap.turn_length == Some(0) {
            snap.turn_length = Some(DEFAULT_TURN_LENGTH);
        }

        // apply skip headers
        skip_headers.reverse();
        for header in &skip_headers {
            let (ValidatorsInfo { consensus_addrs, vote_addrs }, turn_length) = if header.number > 0
                && header.number % self.parlia.epoch() == snap.miner_history_check_len()
            {
                // change validator set
                let checkpoint_header =
                    self.find_ancient_header(header, ancestor, snap.miner_history_check_len())?;

                let validators_info = self
                    .parlia
                    .parse_validators_from_header(&checkpoint_header)
                    .map_err(|err| BscBlockExecutionError::ParliaConsensusInnerError {
                        error: err.into(),
                    })?;

                let turn_length =
                    self.parlia.get_turn_length_from_header(&checkpoint_header).map_err(|err| {
                        BscBlockExecutionError::ParliaConsensusInnerError { error: err.into() }
                    })?;

                (validators_info, turn_length)
            } else {
                (ValidatorsInfo::default(), None)
            };

            let validator = self.parlia.recover_proposer(header).map_err(|err| {
                BscBlockExecutionError::ParliaConsensusInnerError { error: err.into() }
            })?;
            let attestation =
                self.parlia.get_vote_attestation_from_header(header).map_err(|err| {
                    BscBlockExecutionError::ParliaConsensusInnerError { error: err.into() }
                })?;

            snap = snap
                .apply(
                    validator,
                    header,
                    consensus_addrs,
                    vote_addrs,
                    attestation,
                    turn_length,
                    self.parlia.chain_spec().is_bohr_active_at_timestamp(header.timestamp),
                )
                .ok_or(BscBlockExecutionError::ApplySnapshotFailed)?;

            cache.put(snap.block_hash, snap.clone());
        }

        Ok(snap)
    }

    fn get_header_by_hash(
        &self,
        block_hash: B256,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
    ) -> Result<Header, BlockExecutionError> {
        ancestor
            .and_then(|m| m.get(&block_hash).cloned())
            .or_else(|| {
                self.provider
                    .header(block_hash)
                    .map_err(|err| BscBlockExecutionError::ProviderInnerError { error: err.into() })
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| BscBlockExecutionError::UnknownHeader { block_hash }.into())
    }

    fn find_ancient_header(
        &self,
        header: &Header,
        ancestor: Option<&alloy_primitives::map::HashMap<B256, Header>>,
        count: u64,
    ) -> Result<Header, BlockExecutionError> {
        let mut result = header.clone();
        for _ in 0..count {
            result = self.get_header_by_hash(result.parent_hash, ancestor)?;
        }
        Ok(result)
    }
}
