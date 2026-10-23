//! Reth RPC server types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/aidazsolt-bot/OpBNB-AlteredCarbon/main/assets/logo.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

/// Common RPC constants.
pub mod constants;
pub mod result;

mod module;
pub use module::{
    DefaultRpcModuleValidator, LenientRpcModuleValidator, RethRpcModule, RpcModuleSelection,
    RpcModuleValidator,
};

pub use result::ToRpcResult;
