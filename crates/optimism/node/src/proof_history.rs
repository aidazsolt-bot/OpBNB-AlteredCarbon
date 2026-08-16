//! Node launcher with optional proof-history support.
//!
//! Full historical-proofs ExEx (`reth-optimism-trie` / `reth-optimism-exex`) is deferred.
//! Until then this always launches a plain [`OpNode`].

use crate::{args::RollupArgs, OpNode};
use eyre::ErrReport;
use reth_db::DatabaseEnv;
use reth_node_builder::{NodeBuilder, WithLaunchContext};
use reth_optimism_chainspec::OpChainSpec;
use std::sync::Arc;
use tracing::info;

/// Launch the node.
///
/// When `--rollup.proofs-history` is set, logs a warning and falls back to a normal launch
/// until the trie-backed proofs stack is ported.
pub async fn launch_node(
    builder: WithLaunchContext<NodeBuilder<Arc<DatabaseEnv>, OpChainSpec>>,
    args: RollupArgs,
) -> eyre::Result<(), ErrReport> {
    if args.proofs_history {
        info!(
            target: "reth::cli",
            "proofs-history requested but reth-optimism-trie/exex is not ported yet; launching without historical proofs"
        );
    }

    let handle = builder.node(OpNode::new(args)).launch_with_debug_capabilities().await?;
    handle.node_exit_future.await
}
