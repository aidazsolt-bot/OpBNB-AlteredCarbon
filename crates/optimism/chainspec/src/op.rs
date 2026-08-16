//! Chain specification for the Optimism Mainnet network.

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

/// The Optimism Mainnet spec
pub static OP_MAINNET: Lazy<Arc<OpChainSpec>> = Lazy::new(|| {
    let genesis: alloy_genesis::Genesis =
        serde_json::from_str(include_str!("../res/genesis/optimism.json"))
            .expect("Can't deserialize Optimism Mainnet genesis json");
    let hardforks = OptimismHardfork::op_mainnet();
    OpChainSpec {
        inner: ChainSpec {
            chain: Chain::optimism_mainnet(),
            genesis_header: SealedHeader::new(
                make_genesis_header(&genesis, &hardforks),
                b256!("7ca38a1916c42007829c55e69d3e9a73265554b586a499015373241b8a3fa48b"),
            ),
            genesis,
            paris_block_and_final_difficulty: Some((0, U256::from(0))),
            hardforks,
            base_fee_params: BaseFeeParamsKind::Variable(
                vec![
                    (EthereumHardfork::London.boxed(), BaseFeeParams::optimism()),
                    (OptimismHardfork::Canyon.boxed(), BaseFeeParams::optimism_canyon()),
                ]
                .into(),
            ),
            prune_delete_limit: 10000,
            ..Default::default()
        },
    }
    .into()
});
