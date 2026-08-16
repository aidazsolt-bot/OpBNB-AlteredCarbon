//! Validates execution payload for BSC engine API.

use alloy_rpc_types_engine::ExecutionData;
use reth_bsc_chainspec::BscChainSpec;
use reth_chainspec::{EthChainSpec, EthereumHardforks};
use reth_engine_primitives::{EngineApiValidator, PayloadValidator};
use reth_ethereum_payload_builder::EthereumExecutionPayloadValidator;
use reth_ethereum_primitives::Block;
use reth_node_api::PayloadTypes;
use reth_payload_primitives::{
    validate_version_specific_fields, EngineApiMessageVersion, EngineObjectValidationError,
    NewPayloadError, PayloadOrAttributes,
};
use reth_primitives_traits::SealedBlock;
use std::sync::Arc;

/// Validator for the BSC engine API.
#[derive(Debug, Clone)]
pub struct BscEngineValidator {
    inner: EthereumExecutionPayloadValidator<BscChainSpec>,
}

impl BscEngineValidator {
    /// Instantiates a new validator.
    pub fn new(chain_spec: Arc<BscChainSpec>) -> Self {
        Self { inner: EthereumExecutionPayloadValidator::new(chain_spec) }
    }

    fn chain_spec(&self) -> &BscChainSpec {
        self.inner.chain_spec()
    }
}

impl Default for BscEngineValidator {
    fn default() -> Self {
        Self::new(Arc::new(BscChainSpec::default()))
    }
}

impl<Types> PayloadValidator<Types> for BscEngineValidator
where
    Types: PayloadTypes<ExecutionData = ExecutionData>,
{
    type Block = Block;

    fn convert_payload_to_block(
        &self,
        payload: ExecutionData,
    ) -> Result<SealedBlock<Self::Block>, NewPayloadError> {
        self.inner.ensure_well_formed_payload(payload).map_err(Into::into)
    }
}

impl<Types> EngineApiValidator<Types> for BscEngineValidator
where
    Types: PayloadTypes<
        PayloadAttributes = alloy_rpc_types_engine::PayloadAttributes,
        ExecutionData = ExecutionData,
    >,
    BscChainSpec: EthChainSpec + EthereumHardforks,
{
    fn validate_version_specific_fields(
        &self,
        version: EngineApiMessageVersion,
        payload_or_attrs: PayloadOrAttributes<
            '_,
            Types::ExecutionData,
            alloy_rpc_types_engine::PayloadAttributes,
        >,
    ) -> Result<(), EngineObjectValidationError> {
        validate_version_specific_fields(self.chain_spec(), version, payload_or_attrs)
    }

    fn ensure_well_formed_attributes(
        &self,
        version: EngineApiMessageVersion,
        attributes: &alloy_rpc_types_engine::PayloadAttributes,
    ) -> Result<(), EngineObjectValidationError> {
        validate_version_specific_fields(
            self.chain_spec(),
            version,
            PayloadOrAttributes::<Types::ExecutionData, alloy_rpc_types_engine::PayloadAttributes>::PayloadAttributes(
                attributes,
            ),
        )
    }
}
