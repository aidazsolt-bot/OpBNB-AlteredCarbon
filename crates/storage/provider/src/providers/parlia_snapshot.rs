//! Parlia snapshot delegation for factory and blockchain providers.

use crate::{
    providers::{BlockchainProvider, ProviderFactory, ProviderNodeTypes},
    ParliaSnapshotReader, ParliaSnapshotWriter, ProviderResult,
};
use alloy_primitives::B256;
use reth_primitives::parlia::Snapshot;

impl<N: ProviderNodeTypes> ParliaSnapshotReader for ProviderFactory<N> {
    fn get_parlia_snapshot(&self, block_hash: B256) -> ProviderResult<Option<Snapshot>> {
        self.provider()?.get_parlia_snapshot(block_hash)
    }
}

impl<N: ProviderNodeTypes> ParliaSnapshotReader for BlockchainProvider<N> {
    fn get_parlia_snapshot(&self, block_hash: B256) -> ProviderResult<Option<Snapshot>> {
        self.consistent_provider()?.get_parlia_snapshot(block_hash)
    }
}

impl<N: ProviderNodeTypes> ParliaSnapshotWriter for ProviderFactory<N> {
    fn put_parlia_snapshot(&self, block_hash: B256, snapshot: Snapshot) -> ProviderResult<()> {
        self.provider_rw()?.put_parlia_snapshot(block_hash, snapshot)
    }

    fn delete_parlia_snapshot(&self, block_hash: B256) -> ProviderResult<()> {
        self.provider_rw()?.delete_parlia_snapshot(block_hash)
    }
}
