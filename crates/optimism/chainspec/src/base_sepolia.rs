//! Chain specification for the Base Sepolia testnet network.

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;

use alloc::vec;

use alloy_chains::Chain;
use alloy_primitives::{b256, U256};
use once_cell::sync::Lazy;
use reth_chainspec::{make_genesis_header, BaseFeeParams, BaseFeeParamsKind, ChainSpec};
use reth_ethereum_forks::EthereumHardfork;
use reth_ethereum_forks::Hardfork;
use reth_optimism_forks::OptimismHardfork;
use reth_primitives_traits::SealedHeader;

use crate::OpChainSpec;

/// The Base Sepolia spec
pub static BASE_SEPOLIA: Lazy<Arc<OpChainSpec>> = Lazy::new(|| {
    let genesis: alloy_genesis::Genesis =
        serde_json::from_str(include_str!("../res/genesis/sepolia_base.json"))
            .expect("Can't deserialize Base Sepolia genesis json");
    let hardforks = OptimismHardfork::base_sepolia();
    OpChainSpec {
        inner: ChainSpec {
            chain: Chain::base_sepolia(),
            genesis_header: SealedHeader::new(
                make_genesis_header(&genesis, &hardforks),
                b256!("0dcc9e089e30b90ddfc55be9a37dd15bc551aeee999d2e2b51414c54eaf934e4"),
            ),
            genesis,
            paris_block_and_final_difficulty: Some((0, U256::from(0))),
            hardforks,
            base_fee_params: BaseFeeParamsKind::Variable(
                vec![
                    (EthereumHardfork::London.boxed(), BaseFeeParams::base_sepolia()),
                    (OptimismHardfork::Canyon.boxed(), BaseFeeParams::base_sepolia_canyon()),
                ]
                .into(),
            ),
            prune_delete_limit: 10000,
            ..Default::default()
        },
    }
    .into()
});
