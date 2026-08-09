//! Implementation of the `testing` namespace.
//!
//! This exposes `testing_buildBlockV1`, intended for non-production/debug use.

use alloy_consensus::{Header, Transaction};
use alloy_eips::eip2718::Decodable2718;
use alloy_evm::Evm;
use alloy_primitives::{map::HashSet, Address, Bytes, B256, U256};
use alloy_rpc_types_engine::{ExecutionPayloadEnvelopeV5, PayloadAttributes};
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use reth_errors::RethError;
use reth_ethereum_engine_primitives::EthBuiltPayload;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{execute::BlockBuilder, ConfigureEvm, NextBlockEnvAttributes};
use reth_primitives_traits::{
    transaction::signed::RecoveryError, AlloyBlockHeader as BlockTrait, NodePrimitives, TxTy,
    SignedTransaction,
};
use reth_revm::{database::StateProviderDatabase, db::State};
use reth_rpc_api::TestingApiServer;
use reth_rpc_eth_api::{helpers::Call, FromEthApiError};
use reth_rpc_eth_types::EthApiError;
use reth_storage_api::{BlockReader, HeaderProvider};
use revm::context::Block;
use revm_primitives::map::DefaultHashBuilder;
use std::sync::Arc;
use tracing::debug;

/// Testing API handler.
#[derive(Debug, Clone)]
pub struct TestingApi<Eth, Evm> {
    eth_api: Eth,
    evm_config: Evm,
    /// If true, skip invalid transactions instead of failing.
    skip_invalid_transactions: bool,
}

impl<Eth, Evm> TestingApi<Eth, Evm> {
    /// Create a new testing API handler.
    pub const fn new(eth_api: Eth, evm_config: Evm) -> Self {
        Self { eth_api, evm_config, skip_invalid_transactions: false }
    }

    /// Enable skipping invalid transactions instead of failing.
    pub const fn with_skip_invalid_transactions(mut self) -> Self {
        self.skip_invalid_transactions = true;
        self
    }
}

impl<Eth, Evm> TestingApi<Eth, Evm>
where
    Eth: Call<Provider: BlockReader<Header = Header>>,
    Evm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes, Primitives = EthPrimitives>
        + 'static,
{
    async fn build_block_internal(
        &self,
        parent_block_hash: B256,
        payload_attributes: PayloadAttributes,
        transactions: Vec<Bytes>,
        extra_data: Option<Bytes>,
    ) -> Result<ExecutionPayloadEnvelopeV5, Eth::Error> {
        let evm_config = self.evm_config.clone();
        let skip_invalid_transactions = self.skip_invalid_transactions;
        self.eth_api
            .spawn_with_state_at_block(parent_block_hash, move |eth_api, state| {
                let state = state.database.0;
                let mut db = State::builder()
                    .with_bundle_update()
                    .with_database(StateProviderDatabase::new(&state))
                    .build();
                let parent = eth_api
                    .provider()
                    .sealed_header_by_hash(parent_block_hash)?
                    .ok_or_else(|| {
                        EthApiError::HeaderNotFound(parent_block_hash.into())
                    })?;

                let env_attrs = NextBlockEnvAttributes {
                    timestamp: payload_attributes.timestamp,
                    suggested_fee_recipient: payload_attributes.suggested_fee_recipient,
                    prev_randao: payload_attributes.prev_randao,
                    gas_limit: parent.gas_limit(),
                    parent_beacon_block_root: payload_attributes.parent_beacon_block_root,
                    withdrawals: payload_attributes.withdrawals.map(Into::into),
                    extra_data: extra_data.unwrap_or_default(),
                    slot_number: None,
                };

                let mut builder = evm_config
                    .builder_for_next_block(&mut db, &parent, env_attrs)
                    .map_err(RethError::other)
                    .map_err(Eth::Error::from_eth_err)?;
                builder.apply_pre_execution_changes().map_err(Eth::Error::from_eth_err)?;

                let mut total_fees = U256::ZERO;
                let base_fee = builder.evm_mut().block().basefee();

                let mut invalid_senders: HashSet<Address, DefaultHashBuilder> = HashSet::default();

                let mut recovered_txs = Vec::with_capacity(transactions.len());
                for tx in transactions {
                    let decoded = TxTy::<Evm::Primitives>::decode_2718_exact(tx.as_ref())
                        .map_err(|_| EthApiError::InvalidTransactionSignature)?;
                    let signer = decoded
                        .recover_signer()
                        .ok_or(EthApiError::InvalidTransactionSignature)?;
                    recovered_txs.push(alloy_consensus::transaction::Recovered::new_unchecked(
                        decoded, signer,
                    ));
                }

                for (idx, tx) in recovered_txs.into_iter().enumerate() {
                    let signer = tx.signer();
                    if skip_invalid_transactions && invalid_senders.contains(&signer) {
                        continue;
                    }

                    let tip = tx.effective_tip_per_gas(base_fee).unwrap_or_default();
                    let gas_used = match builder.execute_transaction(tx) {
                        Ok(gas_used) => gas_used.tx_gas_used(),
                        Err(err) => {
                            if skip_invalid_transactions {
                                debug!(
                                    target: "rpc::testing",
                                    tx_idx = idx,
                                    ?signer,
                                    error = ?err,
                                    "Skipping invalid transaction"
                                );
                                invalid_senders.insert(signer);
                                continue;
                            }
                            return Err(Eth::Error::from_eth_err(err));
                        }
                    };

                    total_fees += U256::from(tip) * U256::from(gas_used);
                }
                let outcome = builder.finish(&state, None).map_err(Eth::Error::from_eth_err)?;

                let requests = outcome
                    .block
                    .requests_hash()
                    .is_some()
                    .then_some(outcome.execution_result.requests);

                EthBuiltPayload::new(
                    Arc::new(outcome.block),
                    total_fees,
                    requests,
                    None,
                )
                .try_into_v5()
                .map_err(RethError::other)
                .map_err(Eth::Error::from_eth_err)
            })
            .await
    }
}

#[async_trait]
impl<Eth, Evm> TestingApiServer for TestingApi<Eth, Evm>
where
    Eth: Call<Provider: BlockReader<Header = Header>>,
    Evm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes, Primitives = EthPrimitives>
        + 'static,
{
    async fn build_block_v1(
        &self,
        parent_block_hash: B256,
        payload_attributes: PayloadAttributes,
        transactions: Option<Vec<Bytes>>,
        extra_data: Option<Bytes>,
    ) -> RpcResult<ExecutionPayloadEnvelopeV5> {
        self.build_block_internal(
            parent_block_hash,
            payload_attributes,
            transactions.unwrap_or_default(),
            extra_data,
        )
        .await
        .map_err(Into::into)
    }

    async fn commit_block_v1(
        &self,
        _payload_attributes: PayloadAttributes,
        _transactions: Option<Vec<Bytes>>,
        _extra_data: Option<Bytes>,
    ) -> RpcResult<B256> {
        Err(jsonrpsee::types::error::ErrorObject::owned(
            jsonrpsee::types::error::METHOD_NOT_FOUND_CODE,
            "testing_commitBlockV1 is not available",
            None::<()>,
        ))
    }
}
