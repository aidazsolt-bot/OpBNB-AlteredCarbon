//! Helper functions for receipt root calculation on Optimism hardforks.

use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::B256;
use alloy_trie::root::ordered_trie_root_with_encoder;
use reth_optimism_forks::OptimismHardforks;
use reth_primitives_traits::Receipt;

/// Calculates the receipt root for a header.
///
/// Regolith deposit-nonce stripping requires `OpReceipt` support in
/// `reth-optimism-primitives`; until then this uses standard receipt encoding.
pub fn calculate_receipt_root_no_memo_optimism<R: Receipt>(
    receipts: &[R],
    chain_spec: &impl OptimismHardforks,
    timestamp: u64,
) -> B256 {
    let _ = (chain_spec, timestamp);
    ordered_trie_root_with_encoder(receipts, |r, buf| {
        r.with_bloom_ref().encode_2718(buf);
    })
}
