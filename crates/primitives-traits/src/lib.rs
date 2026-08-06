//! Common abstracted types in Reth.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

/// Common constants.
pub mod constants;

pub use constants::gas_units::{format_gas, format_gas_throughput};

/// Minimal account
pub mod account;
pub use account::{Account, Bytecode};

pub mod receipt;
pub use receipt::{FullReceipt, Receipt};

pub mod transaction;
pub use transaction::{
    execute::FillTxEnv, signed::{FullSignedTx, SignedTransaction}, FullTransaction, Transaction,
};
pub use alloy_consensus::transaction::{Recovered, SignerRecoverable, TransactionMeta, TxHashRef};

pub mod crypto;
pub mod proofs;
pub use alloy_eips::eip2718::WithEncoded;

mod integer_list;
pub use integer_list::{IntegerList, IntegerListError};

pub mod block;
pub use block::{
    body::{BlockBody, FullBlockBody}, error::BlockRecoveryError, header::{AlloyBlockHeader, FullBlockHeader},
    recovered::IndexedTx, Block, FullBlock, RecoveredBlock, SealedBlock,
};

mod withdrawal;
pub use withdrawal::Withdrawals;

mod error;
pub use error::{GotExpected, GotExpectedBoxed};

mod log;
pub use alloy_primitives::{logs_bloom, Log, LogData};

mod storage;
pub use storage::{StorageEntry, ValueWithSubKey};

mod size;
pub use size::InMemorySize;

mod node;
pub use node::{BlockTy, BodyTy, HeaderTy, NodePrimitives, ReceiptTy, TxTy};

/// Transaction types
pub mod tx_type;
pub use tx_type::TxType;

/// Common header types
pub mod header;

mod blob_sidecar;
pub use blob_sidecar::{BlobSidecar, BlobSidecars};

#[cfg(any(test, feature = "arbitrary", feature = "test-utils"))]
pub use header::test_utils;
pub use header::{BlockHeader, Header, HeaderError, SealedHeader, SealedHeaderFor};

/// Fast monotonic clock used in hot metrics paths.
#[cfg(feature = "std")]
pub use std::time::Instant as FastInstant;

/// Re-exports of `std::sync` primitives used by restored/compat crates that were previously
/// exposed indirectly via `reth-primitives-traits`.
#[cfg(feature = "std")]
pub mod sync {
    pub use std::sync::{LazyLock, OnceLock};
}

#[cfg(not(feature = "std"))]
/// Compatibility re-exports for sync primitives in `no_std` builds.
pub mod sync {
    pub use core::cell::OnceCell as OnceLock;

    /// Placeholder `LazyLock` for compatibility in `no_std` builds.
    #[derive(Debug)]
    pub struct LazyLock<T>(OnceLock<T>);
}

#[cfg(feature = "serde")]
pub trait MaybeSerde: serde::Serialize + for<'de> serde::Deserialize<'de> {}
#[cfg(not(feature = "serde"))]
/// Noop. Helper trait that would require serde when the feature is enabled.
pub trait MaybeSerde {}

#[cfg(feature = "serde")]
impl<T> MaybeSerde for T where T: serde::Serialize + for<'de> serde::Deserialize<'de> {}

#[cfg(not(feature = "serde"))]
impl<T> MaybeSerde for T {}

#[cfg(feature = "reth-codec")]
pub trait MaybeCompact: reth_codecs::Compact {}
#[cfg(not(feature = "reth-codec"))]
/// Noop. Helper trait that would require compact encoding when the feature is enabled.
pub trait MaybeCompact {}

#[cfg(feature = "reth-codec")]
impl<T> MaybeCompact for T where T: reth_codecs::Compact {}
#[cfg(not(feature = "reth-codec"))]
impl<T> MaybeCompact for T {}

#[cfg(feature = "serde-bincode-compat")]
pub trait MaybeSerdeBincodeCompat: crate::serde_bincode_compat::SerdeBincodeCompat {}
#[cfg(not(feature = "serde-bincode-compat"))]
/// Noop. Helper trait that would require bincode-compatible serde when enabled.
pub trait MaybeSerdeBincodeCompat {}

#[cfg(feature = "serde-bincode-compat")]
impl<T> MaybeSerdeBincodeCompat for T where T: crate::serde_bincode_compat::SerdeBincodeCompat {}
#[cfg(not(feature = "serde-bincode-compat"))]
impl<T> MaybeSerdeBincodeCompat for T {}

/// Bincode-compatible serde implementations for common abstracted types in Reth.
///
/// `bincode` crate doesn't work with optionally serializable serde fields, but some of the
/// Reth types require optional serialization for RPC compatibility. This module makes so that
/// all fields are serialized.
///
/// Read more: <https://github.com/bincode-org/bincode/issues/326>
#[cfg(feature = "serde-bincode-compat")]
pub mod serde_bincode_compat;
