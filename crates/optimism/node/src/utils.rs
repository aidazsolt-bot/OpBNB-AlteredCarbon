use crate::{OpBuiltPayload, OpNode as OtherOpNode};
use alloy_genesis::Genesis;
use alloy_primitives::{Address, B256};
use reth_e2e_test_utils::{
    NodeHelperType, TmpDB, transaction::TransactionTestContext, wallet::Wallet,
};
use reth_node_api::NodeTypesWithDBAdapter;
use reth_optimism_chainspec::OpChainSpecBuilder;
use reth_optimism_payload_builder::OpPayloadBuilderAttributes;
use reth_optimism_primitives::OpTransactionSigned;
use reth_provider::providers::BlockchainProvider;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Optimism Node Helper type
pub(crate) type OpNode =
    NodeHelperType<OtherOpNode, BlockchainProvider<NodeTypesWithDBAdapter<OtherOpNode, TmpDB>>>;

/// Creates the initial setup with `num_nodes` of the node config, started and connected.
pub async fn setup(num_nodes: usize) -> eyre::Result<(Vec<OpNode>, Wallet)> {
    let genesis: Genesis =
        serde_json::from_str(include_str!("../tests/assets/genesis.json")).unwrap();
    let (nodes, _runtime, wallet) = reth_e2e_test_utils::setup_engine(
        num_nodes,
        Arc::new(
            OpChainSpecBuilder::optimism_sepolia().genesis(genesis).ecotone_activated().build(),
        ),
        false,
        Default::default(),
        optimism_payload_attributes,
    )
    .await?;
    Ok((nodes, wallet))
}

/// Advance the chain with sequential payloads returning them in the end.
pub async fn advance_chain(
    length: usize,
    node: &mut OpNode,
    wallet: Arc<Mutex<Wallet>>,
) -> eyre::Result<Vec<OpBuiltPayload>> {
    node.advance(length as u64, |_| {
        let wallet = wallet.clone();
        Box::pin(async move {
            let mut wallet = wallet.lock().await;
            let tx_fut = TransactionTestContext::optimism_l1_block_info_tx(
                wallet.chain_id,
                wallet.inner.clone(),
                wallet.inner_nonce,
            );
            wallet.inner_nonce += 1;
            tx_fut.await
        })
    })
    .await
}

/// Helper function to create optimism payload builder attributes
pub fn optimism_payload_attributes(timestamp: u64) -> OpPayloadBuilderAttributes<OpTransactionSigned> {
    OpPayloadBuilderAttributes {
        timestamp,
        prev_randao: B256::ZERO,
        suggested_fee_recipient: Address::ZERO,
        withdrawals: vec![].into(),
        parent_beacon_block_root: Some(B256::ZERO),
        gas_limit: Some(30_000_000),
        ..Default::default()
    }
}
