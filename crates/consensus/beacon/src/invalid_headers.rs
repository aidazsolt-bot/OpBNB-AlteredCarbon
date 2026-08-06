//! Invalid header cache for the consensus engine.

use alloy_eips::eip1898::BlockWithParent;
use alloy_primitives::B256;
use reth_metrics::{
    metrics::{Counter, Gauge},
    Metrics,
};
use schnellru::{ByLength, LruMap};
use tracing::warn;

/// Maximum number of invalid headers that can be tracked by the engine.
const MAX_INVALID_HEADERS: u32 = 512u32;

/// Keeps track of invalid headers.
#[derive(Debug)]
pub struct InvalidHeaderCache {
    headers: LruMap<B256, HeaderEntry>,
}

impl InvalidHeaderCache {
    /// Creates a new [`InvalidHeaderCache`] with the given max length.
    pub fn new(max_length: u32) -> Self {
        Self { headers: LruMap::new(ByLength::new(max_length)) }
    }

    fn insert_entry(&mut self, hash: B256, header: BlockWithParent) {
        self.headers.insert(hash, HeaderEntry { header });
    }

    /// Returns the invalid ancestor's header if it exists in the cache.
    pub fn get(&mut self, hash: &B256) -> Option<BlockWithParent> {
        self.headers.get(hash).map(|e| e.header)
    }

    /// Inserts an invalid block into the cache, with a given invalid ancestor.
    pub fn insert_with_invalid_ancestor(
        &mut self,
        header_hash: B256,
        invalid_ancestor: BlockWithParent,
    ) {
        if self.get(&header_hash).is_none() {
            warn!(target: "consensus::engine", hash=?header_hash, ?invalid_ancestor, "Bad block with existing invalid ancestor");
            self.insert_entry(header_hash, invalid_ancestor);
        }
    }

    /// Inserts an invalid ancestor into the map.
    pub fn insert(&mut self, invalid_ancestor: BlockWithParent) {
        if self.get(&invalid_ancestor.block.hash).is_none() {
            warn!(target: "consensus::engine", ?invalid_ancestor, "Bad block with hash");
            self.insert_entry(invalid_ancestor.block.hash, invalid_ancestor);
        }
    }
}

struct HeaderEntry {
    header: BlockWithParent,
}
