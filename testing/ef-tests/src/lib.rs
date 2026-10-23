//! Abstractions and runners for EF tests.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/aidazsolt-bot/OpBNB-AlteredCarbon/main/assets/logo.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use reth_revm as _;
use revm as _;

pub mod case;
pub mod result;
pub mod suite;

pub mod assert;
pub mod cases;
pub mod models;

pub use case::{Case, Cases};
pub use result::{CaseResult, Error};
pub use suite::Suite;
