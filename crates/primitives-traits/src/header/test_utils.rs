//! Test utilities to generate random valid headers.

use crate::Header;
use alloy_primitives::B256;
use proptest::{arbitrary::any, prop_compose};
use proptest_arbitrary_interop::arb;

/// Generates a header which is valid __with respect to past and future forks__. This means, for
/// example, that if the withdrawals root is present, the base fee per gas is also present.
///
/// If blob gas used were present, then the excess blob gas and parent beacon block root are also
/// present. In this example, the withdrawals root would also be present.
///
/// This __does not, and should not guarantee__ that the header is valid with respect to __anything
/// else__.
pub const fn generate_valid_header(
    mut header: Header,
    eip_4844_active: bool,
    blob_gas_used: u64,
    excess_blob_gas: u64,
    parent_beacon_block_root: B256,
) -> Header {
    // Clear all related fields if EIP-1559 is inactive
    if header.base_fee_per_gas.is_none() {
        header.withdrawals_root = None;
    }

    // Set fields based on EIP-4844 being active
    if eip_4844_active {
        header.blob_gas_used = Some(blob_gas_used);
        header.excess_blob_gas = Some(excess_blob_gas);
        header.parent_beacon_block_root = Some(parent_beacon_block_root);
    } else {
        header.blob_gas_used = None;
        header.excess_blob_gas = None;
        header.parent_beacon_block_root = None;
    }

    // Placeholder for future EIP adjustments
    header.requests_hash = None;

    header
}

prop_compose! {
    /// Generates a proptest strategy for constructing an instance of a header which is valid __with
    /// respect to past and future forks__.
    ///
    /// See docs for [generate_valid_header] for more information.
    pub fn valid_header_strategy()(
        header in arb::<Header>(),
        eip_4844_active in any::<bool>(),
        blob_gas_used in any::<u64>(),
        excess_blob_gas in any::<u64>(),
        parent_beacon_block_root in arb::<B256>()
    ) -> Header {
        generate_valid_header(header, eip_4844_active, blob_gas_used, excess_blob_gas, parent_beacon_block_root)
    }
}

/// Test-only extension trait providing mutable setters for block header fields, used to build
/// synthetic blocks/headers in tests without needing to construct a whole new header value.
#[cfg(any(test, feature = "test-utils"))]
pub trait TestHeader: crate::BlockHeader {
    /// Updates the parent block hash.
    fn set_parent_hash(&mut self, hash: B256);

    /// Updates the block number.
    fn set_block_number(&mut self, number: alloy_primitives::BlockNumber);

    /// Updates the block state root.
    fn set_state_root(&mut self, state_root: B256);

    /// Updates the block difficulty.
    fn set_difficulty(&mut self, difficulty: alloy_primitives::U256);

    /// Updates the block timestamp.
    fn set_timestamp(&mut self, timestamp: u64);
}

#[cfg(any(test, feature = "test-utils"))]
impl TestHeader for Header {
    fn set_parent_hash(&mut self, hash: B256) {
        self.parent_hash = hash;
    }

    fn set_block_number(&mut self, number: alloy_primitives::BlockNumber) {
        self.number = number;
    }

    fn set_state_root(&mut self, state_root: B256) {
        self.state_root = state_root;
    }

    fn set_difficulty(&mut self, difficulty: alloy_primitives::U256) {
        self.difficulty = difficulty;
    }

    fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }
}
