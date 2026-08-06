mod sealed;
pub use sealed::SealedHeader;

mod error;
pub use error::HeaderError;

#[cfg(any(test, feature = "test-utils", feature = "arbitrary"))]
pub mod test_utils;

pub use alloy_consensus::Header;

use crate::InMemorySize;
use alloy_primitives::Sealable;
use core::{fmt, hash::Hash};

/// Bincode-compatible header type serde implementations.
#[cfg(feature = "serde-bincode-compat")]
pub mod serde_bincode_compat {
    pub use super::sealed::serde_bincode_compat::SealedHeader;
}

/// Abstraction of a block header.
///
/// This combines [`alloy_consensus::BlockHeader`] (the header field accessor trait) with the
/// additional bounds required by the [`crate::Block`]/[`crate::SealedBlock`]/
/// [`crate::RecoveredBlock`] machinery to hash, (de)serialize and RLP-(de)code a header.
pub trait BlockHeader:
    Send
    + Sync
    + Unpin
    + Clone
    + Hash
    + Default
    + fmt::Debug
    + PartialEq
    + Eq
    + alloy_rlp::Encodable
    + alloy_rlp::Decodable
    + alloy_consensus::BlockHeader
    + Sealable
    + InMemorySize
    + serde::Serialize
    + for<'a> serde::Deserialize<'a>
    + AsRef<Self>
    + 'static
{
}

impl BlockHeader for Header {}
