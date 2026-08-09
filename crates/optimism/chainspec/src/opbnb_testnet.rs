//! Chain specification for the Opbnb testnet network.

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

/// The opbnb testnet spec
pub static OPBNB_TESTNET: Lazy<Arc<OpChainSpec>> = Lazy::new(|| {
    let genesis: alloy_genesis::Genesis =
        serde_json::from_str(include_str!("../res/genesis/opbnb_testnet.json"))
            .expect("Can't deserialize opBNB testnet genesis json");
    let hardforks = OptimismHardfork::opbnb_testnet();
    OpChainSpec {
        inner: ChainSpec {
            chain: Chain::opbnb_testnet(),
            genesis_header: SealedHeader::new(
                make_genesis_header(&genesis, &hardforks),
                b256!("51fa57729dfb1c27542c21b06cb72a0459c57440ceb43a465dae1307cd04fe80"),
            ),
            genesis,
            paris_block_and_final_difficulty: Some((0, U256::from(0))),
            hardforks,
            base_fee_params: BaseFeeParamsKind::Variable(
                vec![(EthereumHardfork::London.boxed(), BaseFeeParams::ethereum())].into(),
            ),
            prune_delete_limit: 0,
            ..Default::default()
        },
    }
    .into()
});
