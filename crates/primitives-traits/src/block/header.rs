//! Block header data primitive.

use crate::{BlockHeader, MaybeCompact};

/// Re-exported alias.
pub use alloy_consensus::BlockHeader as AlloyBlockHeader;

/// Helper trait that unifies all behaviour required by block header to support full node
/// operations.
pub trait FullBlockHeader: BlockHeader + MaybeCompact {}

impl<T> FullBlockHeader for T where T: BlockHeader + MaybeCompact {}
