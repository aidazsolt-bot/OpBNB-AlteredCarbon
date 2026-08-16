//! Implementation of a tree-like structure for blockchains.
//!
//! ## Feature Flags
//!
//! - `test-utils`: Export utilities for testing

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Re-export of the blockchain tree API.
pub use reth_blockchain_tree_api::*;

/// Buffer of not executed blocks.
pub mod block_buffer;
pub use block_buffer::BlockBuffer;

use aquamarine as _;
