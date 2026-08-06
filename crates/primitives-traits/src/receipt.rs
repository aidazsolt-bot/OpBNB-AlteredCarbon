//! Receipt abstraction

use alloy_consensus::{RlpDecodableReceipt, RlpEncodableReceipt, TxReceipt, Typed2718};
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
    + TxReceipt
    + Typed2718
    + Default
    + RlpEncodableReceipt
    + RlpDecodableReceipt
    + Encodable
    + Decodable
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
        + TxReceipt
        + Typed2718
        + Default
        + RlpEncodableReceipt
        + RlpDecodableReceipt
        + Encodable
        + Decodable
        + Serialize
        + for<'de> Deserialize<'de>
{
}
