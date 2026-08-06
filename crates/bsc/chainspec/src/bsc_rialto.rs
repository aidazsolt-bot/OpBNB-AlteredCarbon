//! Chain specification for the BSC Mainnet network.

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;

use alloy_chains::Chain;
use alloy_primitives::{b256, U256};
use once_cell::sync::Lazy;
use reth_bsc_forks::BscHardfork;
use reth_chainspec::{make_genesis_header, BaseFeeParams, BaseFeeParamsKind, ChainSpec};
use reth_primitives_traits::SealedHeader;

use crate::BscChainSpec;

/// The BSC mainnet spec
pub static BSC_RIALTO: Lazy<Arc<BscChainSpec>> = Lazy::new(|| {
    let genesis: alloy_genesis::Genesis =
        serde_json::from_str(include_str!("../res/genesis/bsc_rialto.json"))
            .expect("Can't deserialize BSC Rialto genesis json");
    let hardforks = BscHardfork::bsc_qa();
    BscChainSpec {
        inner: ChainSpec {
            chain: Chain::from_id(714),
            genesis_header: SealedHeader::new(
                make_genesis_header(&genesis, &hardforks),
                b256!("281bcec9447d74982bb746f753eecd1320a9b7f4fdbd2cfaf1ae7170a93cff50"),
            ),
            genesis,
            paris_block_and_final_difficulty: Some((0, U256::from(0))),
            hardforks,
            deposit_contract: None,
            base_fee_params: BaseFeeParamsKind::Constant(BaseFeeParams::new(1, 1)),
            prune_delete_limit: 3500,
            ..Default::default()
        },
    }
    .into()
});
