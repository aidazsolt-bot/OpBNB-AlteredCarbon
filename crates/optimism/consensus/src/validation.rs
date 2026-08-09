//! Post-execution validation for Optimism blocks.

use alloy_consensus::BlockHeader;
use reth_consensus::ConsensusError;
use reth_execution_types::BlockExecutionResult;
use reth_optimism_forks::OptimismHardforks;
use reth_primitives_traits::{receipt::gas_spent_by_transactions, GotExpected, Receipt};

/// Validate a block with regard to execution results.
///
/// Full receipt-root validation requires `OpReceipt` + deposit-nonce handling; this validates gas
/// usage for now.
pub fn validate_block_post_execution<R: Receipt>(
    header: impl BlockHeader,
    _chain_spec: &impl OptimismHardforks,
    result: &BlockExecutionResult<R>,
    _receipt_root_bloom: Option<(alloy_primitives::B256, alloy_primitives::Bloom)>,
) -> Result<(), ConsensusError> {
    let cumulative_gas_used =
        result.receipts.last().map(|r| r.cumulative_gas_used()).unwrap_or(0);
    if header.gas_used() != cumulative_gas_used {
        return Err(ConsensusError::BlockGasUsed {
            gas: GotExpected { got: cumulative_gas_used, expected: header.gas_used() },
            gas_spent_by_tx: gas_spent_by_transactions(&result.receipts),
        })
    }
    Ok(())
}
