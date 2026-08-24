//! Optimism transaction types

mod tx_type;

/// Kept for consistency tests
#[cfg(test)]
mod signed;

pub use op_alloy_consensus::{
    build_post_exec_tx, OpTransaction, OpTxEnvelope, OpTxType, OpTypedTransaction, PostExecPayload,
    SDMGasEntry, TxPostExec, POST_EXEC_TX_TYPE_ID,
};

/// Signed transaction.
pub type OpTransactionSigned = OpTxEnvelope;
