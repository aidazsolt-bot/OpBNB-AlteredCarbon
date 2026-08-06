//! Receipt abstraction

use alloy_consensus::{
    Eip2718EncodableReceipt, RlpDecodableReceipt, RlpEncodableReceipt, TxReceipt, Typed2718,
};
use alloy_rlp::{Decodable, Encodable};
use reth_codecs::Compact;
use serde::{Deserialize, Serialize};

/// Helper trait that unifies all behaviour required by receipt to support full node operations.
pub trait FullReceipt: Receipt + Compact {}

impl<T> FullReceipt for T where T: Receipt + Compact {}

/// Abstraction of a receipt.
pub trait Receipt:
    Send
    + Sync
    + Unpin
    + Clone
    + core::fmt::Debug
    + PartialEq
    + Eq
    + TxReceipt<Log = alloy_primitives::Log>
    + Typed2718
    + Default
    + RlpEncodableReceipt
    + RlpDecodableReceipt
    + Encodable
    + Decodable
    + Eip2718EncodableReceipt
    + Serialize
    + for<'de> Deserialize<'de>
{
    /// Returns transaction type.
    fn tx_type(&self) -> u8 {
        self.ty()
    }
}

impl<T> Receipt for T where
    T: Send
        + Sync
        + Unpin
        + Clone
        + core::fmt::Debug
        + PartialEq
        + Eq
        + TxReceipt<Log = alloy_primitives::Log>
        + Typed2718
        + Default
        + RlpEncodableReceipt
        + RlpDecodableReceipt
        + Encodable
        + Decodable
        + Eip2718EncodableReceipt
        + Serialize
        + for<'de> Deserialize<'de>
{
}

/// Retrieves gas spent by transactions as a vector of tuples (transaction index, gas used).
pub fn gas_spent_by_transactions<I, T>(receipts: I) -> alloc::vec::Vec<(u64, u64)>
where
    I: IntoIterator<Item = T>,
    T: TxReceipt,
{
    receipts
        .into_iter()
        .enumerate()
        .map(|(id, receipt)| (id as u64, receipt.cumulative_gas_used()))
        .collect()
}
