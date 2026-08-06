mod signature;

pub use signature::*;

use std::fmt;

use alloy_consensus::Transaction as _;
use alloy_rpc_types_eth::{
    transaction::{TransactionInput, TransactionRequest},
    TransactionInfo,
};
use reth_primitives::{TransactionSigned, TransactionSignedEcRecovered, TxType};
use serde::{Deserialize, Serialize};

pub fn from_recovered_with_block_context<T: TransactionCompat>(
    tx: TransactionSignedEcRecovered,
    tx_info: TransactionInfo,
    resp_builder: &T,
) -> T::Transaction {
    resp_builder.fill(tx, tx_info)
}

pub fn from_recovered<T: TransactionCompat>(
    tx: TransactionSignedEcRecovered,
    resp_builder: &T,
) -> T::Transaction {
    resp_builder.fill(tx, TransactionInfo::default())
}

pub trait TransactionCompat: Send + Sync + Unpin + Clone + fmt::Debug {
    type Transaction: Serialize
        + for<'de> Deserialize<'de>
        + Send
        + Sync
        + Unpin
        + Clone
        + Default
        + fmt::Debug;

    fn gas_price(signed_tx: &TransactionSigned, base_fee: Option<u64>) -> GasPrice {
        #[allow(unreachable_patterns)]
        match signed_tx.tx_type() {
            TxType::Legacy | TxType::Eip2930 => {
                GasPrice { gas_price: Some(signed_tx.max_fee_per_gas()), max_fee_per_gas: None }
            }
            TxType::Eip1559 | TxType::Eip4844 | TxType::Eip7702 => {
                let gas_price = base_fee
                    .and_then(|base_fee| {
                        signed_tx.effective_tip_per_gas(base_fee).map(|tip| tip + base_fee as u128)
                    })
                    .unwrap_or_else(|| signed_tx.max_fee_per_gas());

                GasPrice {
                    gas_price: Some(gas_price),
                    max_fee_per_gas: Some(signed_tx.max_fee_per_gas()),
                }
            }
            _ => GasPrice::default(),
        }
    }

    fn fill(&self, tx: TransactionSignedEcRecovered, tx_inf: TransactionInfo) -> Self::Transaction;
    fn otterscan_api_truncate_input(tx: &mut Self::Transaction);
    fn tx_type(tx: &Self::Transaction) -> u8;
}

#[derive(Debug, Default)]
pub struct GasPrice {
    pub gas_price: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
}

pub fn transaction_to_call_request(tx: TransactionSignedEcRecovered) -> TransactionRequest {
    let from = tx.signer();
    let to = Some(tx.to().into());
    let gas = tx.gas_limit();
    let value = tx.value();
    let input = tx.input().clone();
    let nonce = tx.nonce();
    let chain_id = tx.chain_id();
    let access_list = tx.access_list().cloned();
    let max_fee_per_blob_gas = tx.max_fee_per_blob_gas();
    let authorization_list = tx.authorization_list().map(|l| l.to_vec());
    let blob_versioned_hashes = tx.blob_versioned_hashes().map(|hashes| hashes.to_vec());
    let tx_type = tx.tx_type();

    let (gas_price, max_fee_per_gas) = if tx.is_dynamic_fee() {
        (None, Some(tx.max_fee_per_gas()))
    } else {
        (Some(tx.max_fee_per_gas()), None)
    };
    let max_priority_fee_per_gas = tx.max_priority_fee_per_gas();

    TransactionRequest {
        from: Some(from),
        to,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        gas: Some(gas),
        value: Some(value),
        input: TransactionInput::new(input),
        nonce: Some(nonce),
        chain_id,
        access_list,
        max_fee_per_blob_gas,
        blob_versioned_hashes,
        transaction_type: Some(tx_type.into()),
        sidecar: None,
        authorization_list,
    }
}
