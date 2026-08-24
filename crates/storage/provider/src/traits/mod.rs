//! Collection of common provider traits.

// Re-export all the traits
pub use reth_storage_api::*;

pub use reth_chainspec::ChainSpecProvider;

mod static_file_provider;
pub use static_file_provider::StaticFileProviderFactory;

mod rocksdb_provider;
pub use rocksdb_provider::RocksDBProviderFactory;

mod full;
pub use full::FullProvider;

mod parlia_provider;
pub use parlia_provider::ParliaProvider;

mod parlia_snapshot;
pub use parlia_snapshot::{ParliaSnapshotReader, ParliaSnapshotWriter};
