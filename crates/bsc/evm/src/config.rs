use reth_bsc_forks::{BscHardfork, BscHardforks};
use reth_chainspec::ChainSpec;
use reth_ethereum_forks::{EthereumHardfork, EthereumHardforks, Head};
use revm::primitives::hardfork::SpecId;

/// Returns the stock revm [`SpecId`] at the given timestamp.
///
/// BSC business-logic forks are mapped onto the closest upstream EVM execution spec.
pub fn revm_spec_by_timestamp_after_shanghai(chain_spec: &ChainSpec, timestamp: u64) -> SpecId {
    if chain_spec.is_bohr_active_at_timestamp(timestamp)
        || chain_spec.is_haber_fix_active_at_timestamp(timestamp)
        || chain_spec.is_haber_active_at_timestamp(timestamp)
    {
        SpecId::CANCUN
    } else if chain_spec.is_feynman_fix_active_at_timestamp(timestamp)
        || chain_spec.is_feynman_active_at_timestamp(timestamp)
        || chain_spec.is_kepler_active_at_timestamp(timestamp)
    {
        SpecId::SHANGHAI
    } else {
        SpecId::SHANGHAI
    }
}

/// Return `revm_spec` from spec configuration.
pub fn revm_spec(chain_spec: &ChainSpec, block: &Head) -> SpecId {
    if chain_spec.is_bohr_active_at_timestamp(block.timestamp)
        || chain_spec.is_haber_fix_active_at_timestamp(block.timestamp)
        || chain_spec.is_haber_active_at_timestamp(block.timestamp)
        || chain_spec.is_cancun_active_at_timestamp(block.timestamp)
    {
        SpecId::CANCUN
    } else if chain_spec.is_feynman_fix_active_at_timestamp(block.timestamp)
        || chain_spec.is_feynman_active_at_timestamp(block.timestamp)
        || chain_spec.is_kepler_active_at_timestamp(block.timestamp)
        || chain_spec.is_shanghai_active_at_timestamp(block.timestamp)
    {
        SpecId::SHANGHAI
    } else if chain_spec.is_fork_active_at_block(BscHardfork::HertzFix, block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Hertz, block.number)
        || chain_spec.is_london_active_at_block(block.number)
    {
        SpecId::LONDON
    } else if chain_spec.is_berlin_active_at_block(block.number) {
        SpecId::BERLIN
    } else if chain_spec.is_plato_active_at_block(block.number)
        || chain_spec.is_luban_active_at_block(block.number)
        || chain_spec.is_planck_active_at_block(block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Gibbs, block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Moran, block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Nano, block.number)
        || chain_spec.is_euler_active_at_block(block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Bruno, block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::MirrorSync, block.number)
        || chain_spec.is_fork_active_at_block(BscHardfork::Niels, block.number)
        || chain_spec.is_ramanujan_active_at_block(block.number)
        || chain_spec.is_fork_active_at_block(EthereumHardfork::MuirGlacier, block.number)
    {
        SpecId::BERLIN
    } else if chain_spec.is_istanbul_active_at_block(block.number) {
        SpecId::ISTANBUL
    } else if chain_spec.is_petersburg_active_at_block(block.number) {
        SpecId::PETERSBURG
    } else if chain_spec.is_constantinople_active_at_block(block.number) {
        SpecId::PETERSBURG
    } else if chain_spec.is_byzantium_active_at_block(block.number) {
        SpecId::BYZANTIUM
    } else if chain_spec.is_homestead_active_at_block(block.number) {
        SpecId::HOMESTEAD
    } else if chain_spec.is_fork_active_at_block(EthereumHardfork::Frontier, block.number) {
        SpecId::FRONTIER
    } else {
        panic!(
            "invalid hardfork chainspec: expected at least one hardfork, got {:?}",
            chain_spec.hardforks
        )
    }
}
