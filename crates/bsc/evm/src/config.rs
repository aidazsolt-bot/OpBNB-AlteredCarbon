use reth_bsc_forks::BscHardfork;
use reth_chainspec::ChainSpec;
use reth_ethereum_forks::{EthereumHardfork, Head};
use revm::primitives::hardfork::SpecId;

/// Returns the stock revm [`SpecId`] at the given timestamp.
///
/// BSC business-logic forks are mapped onto the closest upstream EVM execution spec.
pub fn revm_spec_by_timestamp_after_shanghai(chain_spec: &ChainSpec, timestamp: u64) -> SpecId {
    if chain_spec.fork(BscHardfork::Bohr).active_at_timestamp(timestamp)
        || chain_spec.fork(BscHardfork::HaberFix).active_at_timestamp(timestamp)
        || chain_spec.fork(BscHardfork::Haber).active_at_timestamp(timestamp)
    {
        SpecId::CANCUN
    } else if chain_spec.fork(BscHardfork::FeynmanFix).active_at_timestamp(timestamp)
        || chain_spec.fork(BscHardfork::Feynman).active_at_timestamp(timestamp)
        || chain_spec.fork(BscHardfork::Kepler).active_at_timestamp(timestamp)
    {
        SpecId::SHANGHAI
    } else {
        SpecId::SHANGHAI
    }
}

/// return `revm_spec` from spec configuration.
pub fn revm_spec(chain_spec: &ChainSpec, block: &Head) -> SpecId {
    if chain_spec.fork(BscHardfork::Bohr).active_at_head(block)
        || chain_spec.fork(BscHardfork::HaberFix).active_at_head(block)
        || chain_spec.fork(BscHardfork::Haber).active_at_head(block)
        || chain_spec.fork(EthereumHardfork::Cancun).active_at_head(block)
    {
        SpecId::CANCUN
    } else if chain_spec.fork(BscHardfork::FeynmanFix).active_at_head(block)
        || chain_spec.fork(BscHardfork::Feynman).active_at_head(block)
        || chain_spec.fork(BscHardfork::Kepler).active_at_head(block)
        || chain_spec.fork(EthereumHardfork::Shanghai).active_at_head(block)
    {
        SpecId::SHANGHAI
    } else if chain_spec.fork(BscHardfork::HertzFix).active_at_head(block)
        || chain_spec.fork(BscHardfork::Hertz).active_at_head(block)
        || chain_spec.fork(EthereumHardfork::London).active_at_head(block)
    {
        SpecId::LONDON
    } else if chain_spec.fork(EthereumHardfork::Berlin).active_at_head(block) {
        SpecId::BERLIN
    } else if chain_spec.fork(BscHardfork::Plato).active_at_head(block)
        || chain_spec.fork(BscHardfork::Luban).active_at_head(block)
        || chain_spec.fork(BscHardfork::Planck).active_at_head(block)
        || chain_spec.fork(BscHardfork::Gibbs).active_at_head(block)
        || chain_spec.fork(BscHardfork::Moran).active_at_head(block)
        || chain_spec.fork(BscHardfork::Nano).active_at_head(block)
        || chain_spec.fork(BscHardfork::Euler).active_at_head(block)
        || chain_spec.fork(BscHardfork::Bruno).active_at_head(block)
        || chain_spec.fork(BscHardfork::MirrorSync).active_at_head(block)
        || chain_spec.fork(BscHardfork::Niels).active_at_head(block)
        || chain_spec.fork(BscHardfork::Ramanujan).active_at_head(block)
        || chain_spec.fork(EthereumHardfork::MuirGlacier).active_at_head(block)
    {
        SpecId::MUIR_GLACIER
    } else if chain_spec.fork(EthereumHardfork::Istanbul).active_at_head(block) {
        SpecId::ISTANBUL
    } else if chain_spec.fork(EthereumHardfork::Petersburg).active_at_head(block) {
        SpecId::PETERSBURG
    } else if chain_spec.fork(EthereumHardfork::Constantinople).active_at_head(block) {
        SpecId::CONSTANTINOPLE
    } else if chain_spec.fork(EthereumHardfork::Byzantium).active_at_head(block) {
        SpecId::BYZANTIUM
    } else if chain_spec.fork(EthereumHardfork::Homestead).active_at_head(block) {
        SpecId::HOMESTEAD
    } else if chain_spec.fork(EthereumHardfork::Frontier).active_at_head(block) {
        SpecId::FRONTIER
    } else {
        panic!(
            "invalid hardfork chainspec: expected at least one hardfork, got {:?}",
            chain_spec.hardforks
        )
    }
}
