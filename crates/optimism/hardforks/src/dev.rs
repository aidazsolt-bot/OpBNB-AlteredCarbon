use alloc::vec;

use alloy_primitives::U256;
use once_cell::sync::Lazy;
use reth_ethereum_forks::{ChainHardforks, EthereumHardfork, ForkCondition, Hardfork};

use crate::OptimismHardfork;

/// Dev hardforks with all optimism forks activated at genesis.
pub static DEV_HARDFORKS: Lazy<ChainHardforks> = Lazy::new(|| {
    ChainHardforks::new(vec![
        (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::MuirGlacier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::ArrowGlacier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::GrayGlacier.boxed(), ForkCondition::Block(0)),
        (
            EthereumHardfork::Paris.boxed(),
            ForkCondition::TTD {
                activation_block_number: 0,
                fork_block: None,
                total_difficulty: U256::ZERO,
            },
        ),
        (OptimismHardfork::Bedrock.boxed(), ForkCondition::Block(0)),
        (OptimismHardfork::Regolith.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::PreContractForkBlock.boxed(), ForkCondition::Block(0)),
        (OptimismHardfork::Fermat.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Snow.boxed(), ForkCondition::Timestamp(0)),
        (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Canyon.boxed(), ForkCondition::Timestamp(0)),
        (EthereumHardfork::Cancun.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Ecotone.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Haber.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Wright.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Fjord.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Volta.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Fourier.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Granite.boxed(), ForkCondition::Timestamp(0)),
        (OptimismHardfork::Holocene.boxed(), ForkCondition::Timestamp(0)),
    ])
});
