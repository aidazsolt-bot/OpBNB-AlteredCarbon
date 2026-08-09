//! Optimism-specific RPC conversion trait implementations.

use alloy_consensus::SignableTransaction;
use alloy_primitives::{Address, Signature};
use core::convert::Infallible;
use op_alloy_consensus::{
    OpTxEnvelope, TxDeposit,
    transaction::{OpTransaction, OpTransactionInfo},
};
use op_alloy_rpc_types::OpTransactionRequest;

use crate::{FromConsensusTx, SignTxRequestError, SignableTxRequest, TryIntoSimTx};

impl<T: OpTransaction + alloy_consensus::Transaction> FromConsensusTx<T>
    for op_alloy_rpc_types::Transaction<T>
{
    type TxInfo = OpTransactionInfo;
    type Err = Infallible;

    fn from_consensus_tx(tx: T, signer: Address, tx_info: Self::TxInfo) -> Result<Self, Self::Err> {
        Ok(Self::from_transaction(
            alloy_consensus::transaction::Recovered::new_unchecked(tx, signer),
            tx_info,
        ))
    }
}

impl TryIntoSimTx<OpTxEnvelope> for OpTransactionRequest {
    fn try_into_sim_tx(self) -> Result<OpTxEnvelope, alloy_consensus::error::ValueError<Self>> {
        let tx = self.build_typed_tx().map_err(|request| {
            alloy_consensus::error::ValueError::new(request, "Required fields missing")
        })?;

        // Empty signature for simulation.
        let signature = Signature::new(Default::default(), Default::default(), false);

        Ok(tx.into_signed(signature).into())
    }
}

impl SignableTxRequest<OpTxEnvelope> for OpTransactionRequest {
    async fn try_build_and_sign(
        self,
        signer: impl alloy_network::TxSigner<Signature> + Send,
    ) -> Result<OpTxEnvelope, SignTxRequestError> {
        let mut tx =
            self.build_typed_tx().map_err(|_| SignTxRequestError::InvalidTransactionRequest)?;

        // Deposit transactions must not be signed by the user.
        if matches!(tx, op_alloy_consensus::OpTypedTransaction::Deposit(TxDeposit { .. })) {
            return Err(SignTxRequestError::InvalidTransactionRequest);
        }

        let signature = signer.sign_transaction(&mut tx).await?;

        Ok(tx.into_signed(signature).into())
    }
}
