//! [`libmdbx`](https://github.com/erthink/libmdbx) bindings.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/aidazsolt-bot/OpBNB-AlteredCarbon/main/assets/logo.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/aidazsolt-bot/OpBNB-AlteredCarbon/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
