use alloy_primitives::B256;
use reth_primitives::parlia::Snapshot;
use reth_storage_errors::provider::ProviderResult;

/// The trait for fetching parlia snapshot related data.
#[auto_impl::auto_impl(&, Arc)]
pub trait ParliaSnapshotReader: Send + Sync {
    /// Fetch the snapshot for the given block hash.
    fn get_parlia_snapshot(&self, block_hash: B256) -> ProviderResult<Option<Snapshot>>;
}

/// The trait for persisting Parlia checkpoint snapshots (BSC).
#[auto_impl::auto_impl(&)]
pub trait ParliaSnapshotWriter: Send + Sync {
    /// Store a Parlia snapshot keyed by block hash.
    fn put_parlia_snapshot(&self, block_hash: B256, snapshot: Snapshot) -> ProviderResult<()>;

    /// Remove a Parlia snapshot keyed by block hash.
    fn delete_parlia_snapshot(&self, block_hash: B256) -> ProviderResult<()>;
}
