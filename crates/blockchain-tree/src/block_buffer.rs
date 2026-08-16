use alloy_consensus::BlockHeader;
use alloy_primitives::{BlockHash, BlockNumber};
use indexmap::IndexSet;
use metrics::Gauge;
use reth_primitives_traits::{Block, SealedBlock};
#[allow(unused_imports)]
use reth_primitives_traits::BlockHeader as _;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub(crate) struct BlockBufferMetrics {
    /// Total blocks in the block buffer
    pub blocks: Gauge,
}

impl std::fmt::Debug for BlockBufferMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockBufferMetrics").finish()
    }
}

impl Default for BlockBufferMetrics {
    fn default() -> Self {
        Self { blocks: metrics::gauge!("blockchain_tree.block_buffer.blocks") }
    }
}

/// Contains the tree of pending blocks that cannot be executed due to missing parent.
/// It allows to store unconnected blocks for potential future inclusion.
///
/// The buffer has three main functionalities:
/// * [`BlockBuffer::insert_block`] for inserting blocks inside the buffer.
/// * [`BlockBuffer::remove_block_with_children`] for connecting blocks if the parent gets received
///   and inserted.
/// * [`BlockBuffer::remove_old_blocks`] to remove old blocks that precede the finalized number.
///
/// Note: Buffer is limited by number of blocks that it can contain and eviction of the block
/// is done in FIFO order (oldest inserted block is evicted first).
#[derive(Debug)]
pub struct BlockBuffer<B: Block> {
    pub(crate) blocks: HashMap<BlockHash, SealedBlock<B>>,
    pub(crate) parent_to_child: HashMap<BlockHash, IndexSet<BlockHash>>,
    pub(crate) earliest_blocks: BTreeMap<BlockNumber, HashSet<BlockHash>>,
    pub(crate) block_queue: VecDeque<BlockHash>,
    pub(crate) max_blocks: usize,
    pub(crate) metrics: BlockBufferMetrics,
}

impl<B: Block> BlockBuffer<B> {
    /// Create new buffer with max limit of blocks
    pub fn new(limit: u32) -> Self {
        Self {
            blocks: Default::default(),
            parent_to_child: Default::default(),
            earliest_blocks: Default::default(),
            block_queue: VecDeque::default(),
            max_blocks: limit as usize,
            metrics: Default::default(),
        }
    }

    /// Return reference to the requested block.
    pub fn block(&self, hash: &BlockHash) -> Option<&SealedBlock<B>> {
        self.blocks.get(hash)
    }

    /// Return a reference to the lowest ancestor of the given block in the buffer.
    pub fn lowest_ancestor(&self, hash: &BlockHash) -> Option<&SealedBlock<B>> {
        let mut current_block = self.blocks.get(hash)?;
        while let Some(parent) = self.blocks.get(&current_block.parent_hash()) {
            current_block = parent;
        }
        Some(current_block)
    }

    /// Insert a correct block inside the buffer.
    pub fn insert_block(&mut self, block: SealedBlock<B>) {
        let hash = block.hash();

        match self.blocks.entry(hash) {
            std::collections::hash_map::Entry::Occupied(_) => return,
            std::collections::hash_map::Entry::Vacant(entry) => {
                self.parent_to_child.entry(block.parent_hash()).or_default().insert(hash);
                self.earliest_blocks.entry(block.number()).or_default().insert(hash);
                entry.insert(block);
            }
        };

        if self.block_queue.len() >= self.max_blocks {
            if let Some(evicted_hash) = self.block_queue.pop_front() {
                self.remove_block(&evicted_hash);
            }
        }
        self.block_queue.push_back(hash);
        self.metrics.blocks.set(self.blocks.len() as f64);
    }

    /// Removes the given block from the buffer and also all the children of the block.
    pub fn remove_block_with_children(&mut self, parent_hash: &BlockHash) -> Vec<SealedBlock<B>> {
        let removed = self
            .remove_block(parent_hash)
            .into_iter()
            .chain(self.remove_children(vec![*parent_hash]))
            .collect();
        self.metrics.blocks.set(self.blocks.len() as f64);
        removed
    }

    /// Discard all blocks that precede block number from the buffer.
    pub fn remove_old_blocks(&mut self, block_number: BlockNumber) {
        let mut block_hashes_to_remove = Vec::new();

        while let Some(entry) = self.earliest_blocks.first_entry() {
            if *entry.key() > block_number {
                break;
            }
            let block_hashes = entry.remove();
            block_hashes_to_remove.extend(block_hashes);
        }

        for block_hash in &block_hashes_to_remove {
            self.remove_block(block_hash);
        }

        self.remove_children(block_hashes_to_remove);
        self.metrics.blocks.set(self.blocks.len() as f64);
    }

    fn remove_from_earliest_blocks(&mut self, number: BlockNumber, hash: &BlockHash) {
        if let Some(entry) = self.earliest_blocks.get_mut(&number) {
            entry.remove(hash);
            if entry.is_empty() {
                self.earliest_blocks.remove(&number);
            }
        }
    }

    fn remove_from_parent(&mut self, parent_hash: BlockHash, hash: &BlockHash) {
        if let Some(entry) = self.parent_to_child.get_mut(&parent_hash) {
            entry.swap_remove(hash);
            if entry.is_empty() {
                self.parent_to_child.remove(&parent_hash);
            }
        }
    }

    fn remove_block(&mut self, hash: &BlockHash) -> Option<SealedBlock<B>> {
        let block = self.blocks.remove(hash)?;
        self.remove_from_earliest_blocks(block.number(), hash);
        self.remove_from_parent(block.parent_hash(), hash);
        self.block_queue.retain(|h| h != hash);
        Some(block)
    }

    fn remove_children(&mut self, parent_hashes: Vec<BlockHash>) -> Vec<SealedBlock<B>> {
        let mut remove_parent_children = parent_hashes;
        let mut removed_blocks = Vec::new();
        while let Some(parent_hash) = remove_parent_children.pop() {
            if let Some(parent_children) = self.parent_to_child.remove(&parent_hash) {
                for child_hash in &parent_children {
                    if let Some(block) = self.remove_block(child_hash) {
                        removed_blocks.push(block);
                    }
                }
                remove_parent_children.extend(parent_children);
            }
        }
        removed_blocks
    }
}
