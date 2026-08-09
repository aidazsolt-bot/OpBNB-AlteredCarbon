//! Storage metadata models.

use reth_codecs::{add_arbitrary_tests, Compact};
use serde::{Deserialize, Serialize};

/// Storage configuration settings for this node.
///
/// Controls whether this node uses v2 storage layout (static files + `RocksDB` routing)
/// or v1/legacy layout (everything in MDBX).
///
/// These should be set during `init_genesis` or `init_db` depending on whether we want dictate
/// behaviour of new or old nodes respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Compact, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
#[add_arbitrary_tests(compact)]
pub struct StorageSettings {
    /// Whether this node uses v2 storage layout.
    ///
    /// When `true`, enables v2 storage features:
    /// - Receipts and transaction senders in static files
    /// - History indices in `RocksDB` (accounts, storages, transaction hashes)
    /// - Hashed state tables as canonical state representation
    ///
    /// Account changesets remain in MDBX until the AccountChangeSets SF segment is fully
    /// ported ([`Self::account_changesets_in_static_files`]). Storage changesets stay in MDBX.
    ///
    /// When `false`, uses v1/legacy layout (everything in MDBX).
    pub storage_v2: bool,
}

impl StorageSettings {
    /// Returns the default base `StorageSettings`.
    pub const fn base() -> Self {
        Self::v2()
    }

    /// Creates `StorageSettings` for v2 nodes:
    /// - Receipts and transaction senders in static files
    /// - History indices in `RocksDB` (storages, accounts, transaction hashes)
    /// - Hashed state as canonical state representation
    ///
    /// Account/storage changesets remain in MDBX until those SF segments are ported.
    ///
    /// Use this when the `--storage.v2` CLI flag is set.
    pub const fn v2() -> Self {
        Self { storage_v2: true }
    }

    /// Creates `StorageSettings` for v1/legacy nodes.
    ///
    /// This keeps all data in MDBX, matching the original storage layout.
    pub const fn v1() -> Self {
        Self { storage_v2: false }
    }

    /// Returns `true` if this node uses v2 storage layout.
    pub const fn is_v2(&self) -> bool {
        self.storage_v2
    }

    /// Whether receipts are stored in static files.
    pub const fn receipts_in_static_files(&self) -> bool {
        self.storage_v2
    }

    /// Whether transaction senders are stored in static files.
    pub const fn transaction_senders_in_static_files(&self) -> bool {
        self.storage_v2
    }

    /// Whether storages history is stored in `RocksDB`.
    pub const fn storages_history_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether transaction hash numbers are stored in `RocksDB`.
    pub const fn transaction_hash_numbers_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether account history is stored in `RocksDB`.
    pub const fn account_history_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether to use hashed state tables (`HashedAccounts`/`HashedStorages`) as the canonical
    /// state representation instead of plain state tables. Implied by v2 storage layout.
    pub const fn use_hashed_state(&self) -> bool {
        self.storage_v2
    }

    /// Returns `true` if any tables are configured to be stored in `RocksDB`.
    pub const fn any_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether account changesets are stored in static files.
    ///
    /// Always `false` in this fork: the AccountChangeSets static-file segment is not ported
    /// yet. Upstream v2 writes these to a dedicated SF segment; our incomplete port reused
    /// the Headers segment and broke genesis (`append Headers #0 but expected #1`).
    pub const fn account_changesets_in_static_files(&self) -> bool {
        false
    }
}
