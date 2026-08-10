//! Spec selection for Optimism / opBNB EVMs.
//!
//! Stock OP forks are selected via [`alloy_op_evm`]. opBNB Fermat/Haber/Wright historically
//! used extra SpecIds in patched bnb-chain/revm; those SpecIds no longer exist in stock
//! `op-revm`, so [`opbnb_precompile_flags`] restores the same ladder as a side-channel for
//! precompile overlays (and Wright is handled in consensus / L1-fee paths).

use alloy_consensus::BlockHeader;
use alloy_op_evm::{
    spec as alloy_revm_spec, spec_by_timestamp_after_bedrock as alloy_revm_spec_by_timestamp,
};
use op_alloy_rpc_types_engine::OpFlashblockPayloadBase;
use op_revm::OpSpecId;
use reth_optimism_forks::{Hardforks, OpHardforks, OptimismHardfork};
use revm::primitives::{Address, Bytes, B256};

use crate::opbnb_precompiles::OpBnbPrecompileFlags;

/// Map the latest active hardfork at the given header to an [`OpSpecId`].
pub fn revm_spec(chain_spec: impl OpHardforks, header: impl BlockHeader) -> OpSpecId {
    alloy_revm_spec(chain_spec, header)
}

/// Returns the [`OpSpecId`] at the given timestamp (post-Bedrock timestamp forks).
pub fn revm_spec_by_timestamp_after_bedrock(
    chain_spec: impl OpHardforks,
    timestamp: u64,
) -> OpSpecId {
    alloy_revm_spec_by_timestamp(chain_spec, timestamp)
}

/// Historical SpecId-ladder side-channel: which Fermat/Haber precompile overlays apply.
///
/// Order mirrors the pre-`465b8249d` ladder (`… → Fjord → Wright → Haber → Ecotone → … →
/// Fermat → …`), except Fermat is activated by **block** (op-geth) and Wright does not add
/// precompiles beyond Haber.
pub fn opbnb_precompile_flags(
    chain_spec: &(impl Hardforks + OpHardforks),
    block_number: u64,
    timestamp: u64,
) -> OpBnbPrecompileFlags {
    let fermat = chain_spec.fork(OptimismHardfork::Fermat).active_at_block(block_number);
    // Haber adds early P256; Fjord's stock OpPrecompiles already include it.
    let haber_p256 = chain_spec.fork(OptimismHardfork::Haber).active_at_timestamp(timestamp) &&
        !chain_spec.is_fjord_active_at_timestamp(timestamp);
    OpBnbPrecompileFlags { fermat, haber_p256 }
}

/// Context relevant for execution of a next block w.r.t OP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpNextBlockEnvAttributes {
    /// The timestamp of the next block.
    pub timestamp: u64,
    /// The suggested fee recipient for the next block.
    pub suggested_fee_recipient: Address,
    /// The randomness value for the next block.
    pub prev_randao: B256,
    /// Block gas limit.
    pub gas_limit: u64,
    /// The parent beacon block root.
    pub parent_beacon_block_root: Option<B256>,
    /// Encoded EIP-1559 parameters to include into block's `extra_data` field.
    pub extra_data: Bytes,
}

#[cfg(feature = "rpc")]
impl<H: alloy_consensus::BlockHeader> reth_rpc_eth_api::helpers::pending_block::BuildPendingEnv<H>
    for OpNextBlockEnvAttributes
{
    fn build_pending_env(
        parent: &crate::SealedHeader<H>,
        block_overrides: Option<&alloy_rpc_types_eth::BlockOverrides>,
    ) -> Self {
        let mut attributes = Self {
            timestamp: parent.timestamp().saturating_add(12),
            suggested_fee_recipient: parent.beneficiary(),
            prev_randao: B256::random(),
            gas_limit: parent.gas_limit(),
            parent_beacon_block_root: parent.parent_beacon_block_root(),
            extra_data: parent.extra_data().clone(),
        };

        // Only the beacon root override must be applied here: it is consumed during EVM
        // environment construction. All other `BlockOverrides` fields are applied directly
        // to the constructed environment by the caller, matching the upstream
        // `NextBlockEnvAttributes::build_pending_env` behavior.
        if attributes.parent_beacon_block_root.is_some() &&
            let Some(beacon_root) = block_overrides.and_then(|overrides| overrides.beacon_root)
        {
            attributes.parent_beacon_block_root = Some(beacon_root);
        }

        attributes
    }
}

impl From<OpFlashblockPayloadBase> for OpNextBlockEnvAttributes {
    fn from(base: OpFlashblockPayloadBase) -> Self {
        Self {
            timestamp: base.timestamp,
            suggested_fee_recipient: base.fee_recipient,
            prev_randao: base.prev_randao,
            gas_limit: base.gas_limit,
            parent_beacon_block_root: Some(base.parent_beacon_block_root),
            extra_data: base.extra_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_optimism_chainspec::OPBNB_MAINNET;

    #[test]
    fn fermat_flags_by_block() {
        let spec = OPBNB_MAINNET.clone();
        let before = opbnb_precompile_flags(spec.as_ref(), 9_397_476, 0);
        assert!(!before.fermat);
        let after = opbnb_precompile_flags(spec.as_ref(), 9_397_477, 0);
        assert!(after.fermat);
        assert!(!after.haber_p256);
    }

    #[test]
    fn haber_p256_before_fjord_only() {
        let spec = OPBNB_MAINNET.clone();
        let haber = opbnb_precompile_flags(spec.as_ref(), 10_000_000, 1718872200);
        assert!(haber.fermat);
        assert!(haber.haber_p256);
        let fjord = opbnb_precompile_flags(spec.as_ref(), 10_000_000, 1727157600);
        assert!(fjord.fermat);
        assert!(!fjord.haber_p256);
    }
}
