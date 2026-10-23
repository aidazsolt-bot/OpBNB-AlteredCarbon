//! Payload Validation support.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/aidazsolt-bot/OpBNB-AlteredCarbon/main/assets/logo.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod amsterdam;
pub mod cancun;
pub mod prague;
pub mod shanghai;
