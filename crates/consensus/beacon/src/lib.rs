//! Beacon consensus — stub re-exporting from engine-primitives (v2.4.1 compat).
//!
//! In v2.4.1 the original `BeaconConsensusEngine` machinery was replaced by
//! `reth-engine-tree`. This crate is kept as a thin compatibility shim so
//! downstream BSC crates that still `use reth_beacon_consensus::*` continue to
//! compile without changes.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// Core engine message/fork-choice types (moved to reth-engine-primitives in v2.4.1)
pub use reth_engine_primitives::{
    BeaconConsensusEngineEvent, BeaconEngineMessage, BeaconOnNewPayloadError, ForkchoiceStateHash,
    ForkchoiceStateTracker, ForkchoiceStatus, OnForkChoiceUpdated,
};

/// Compatibility alias — `BeaconConsensusEngineHandle` is now `ConsensusEngineHandle`.
pub use reth_engine_primitives::ConsensusEngineHandle as BeaconConsensusEngineHandle;

// EthBeaconConsensus lives in reth-ethereum-consensus
pub use reth_ethereum_consensus::EthBeaconConsensus;

/// Minimum number of blocks to trigger a pipeline run (moved to engine-primitives).
pub const MIN_BLOCKS_FOR_PIPELINE_RUN: u64 = 32;

mod invalid_headers;
pub use invalid_headers::InvalidHeaderCache;

use reth_node_types::NodeTypes;
/// Convenience trait combining the bounds previously expressed by `EngineNodeTypes`.
/// Downstream BSC crates use this as a bound on their generic `N` parameters.
pub use reth_provider::providers::ProviderNodeTypes;

/// Marker trait: any type that satisfies `ProviderNodeTypes + NodeTypes` also
/// satisfies `EngineNodeTypes`.
pub trait EngineNodeTypes: ProviderNodeTypes + NodeTypes {}
impl<T> EngineNodeTypes for T where T: ProviderNodeTypes + NodeTypes {}

/// Stub `hooks` module — re-exports the static-file hook and provides
/// minimal placeholder types for `EngineHooks` and `PruneHook`.
pub mod hooks {
    pub use crate::static_file::StaticFileHook;

    /// A collection of engine hooks (stub — filled in by engine-tree in v2.4.1).
    #[derive(Default)]
    pub struct EngineHooks {
        inner: Vec<Box<dyn std::any::Any + Send>>,
    }

    impl EngineHooks {
        /// Creates an empty hook collection.
        pub fn new() -> Self {
            Self::default()
        }

        /// Add a hook.
        pub fn add<H: std::any::Any + Send + 'static>(&mut self, hook: H) {
            self.inner.push(Box::new(hook));
        }
    }

    /// Stub prune hook.
    pub struct PruneHook;
}

mod static_file;
pub use static_file::StaticFileHook;
