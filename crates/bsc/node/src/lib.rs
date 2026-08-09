//! Standalone crate for ethereum-specific Reth configuration and builder types.

#![allow(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
// The `bsc` feature must be enabled to use this crate.
#![cfg(feature = "bsc")]

pub mod engine;
pub use engine::BscEngineValidator;

pub mod node;
pub use node::BscNode;
