//! Payload related types

use alloy_eips::eip7685::Requests;
use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_rpc_types_engine::{
    BlobsBundleV1, BlobsBundleV2, ExecutionData, ExecutionPayload, ExecutionPayloadEnvelopeV2,
    ExecutionPayloadEnvelopeV3, ExecutionPayloadEnvelopeV4, ExecutionPayloadEnvelopeV5,
    ExecutionPayloadEnvelopeV6, ExecutionPayloadFieldV2, ExecutionPayloadV1,
    ExecutionPayloadV3, ExecutionPayloadV4, PayloadAttributes, PayloadId,
};
use core::convert::Infallible;
use reth_ethereum_engine_primitives::{BlobSidecars, EthPayloadBuilderAttributes};
use reth_ethereum_primitives::{Block, EthPrimitives};
use reth_payload_primitives::{BuiltPayload, PayloadBuilderAttributes};
use reth_primitives_traits::{NodePrimitives, RecoveredBlock, SealedBlock};
use std::sync::Arc;

/// Bsc Payload Builder Attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BscPayloadBuilderAttributes {
    /// Inner ethereum payload builder attributes
    pub payload_attributes: EthPayloadBuilderAttributes,
}

impl PayloadBuilderAttributes for BscPayloadBuilderAttributes {
    type RpcPayloadAttributes = PayloadAttributes;
    type Error = Infallible;

    fn try_new(
        parent: B256,
        attributes: PayloadAttributes,
        version: u8,
    ) -> Result<Self, Infallible> {
        Ok(Self {
            payload_attributes: EthPayloadBuilderAttributes::try_new(parent, attributes, version)?,
        })
    }

    fn payload_id(&self) -> PayloadId {
        self.payload_attributes.id
    }

    fn parent(&self) -> B256 {
        self.payload_attributes.parent
    }

    fn timestamp(&self) -> u64 {
        self.payload_attributes.timestamp
    }

    fn parent_beacon_block_root(&self) -> Option<B256> {
        self.payload_attributes.parent_beacon_block_root
    }

    fn suggested_fee_recipient(&self) -> Address {
        self.payload_attributes.suggested_fee_recipient
    }

    fn prev_randao(&self) -> B256 {
        self.payload_attributes.prev_randao
    }

    fn withdrawals(&self) -> &alloy_eips::eip4895::Withdrawals {
        &self.payload_attributes.withdrawals
    }
}

/// Contains the built payload.
#[derive(Debug, Clone)]
pub struct BscBuiltPayload {
    /// The built block
    pub(crate) block: Arc<RecoveredBlock<<EthPrimitives as NodePrimitives>::Block>>,
    /// The fees of the block
    pub(crate) fees: U256,
    /// The blobs, proofs, and commitments in the block.
    pub(crate) sidecars: BlobSidecars,
    /// The requests of the payload
    pub(crate) requests: Option<Requests>,
    /// The block access list of the payload
    pub(crate) block_access_list: Option<Bytes>,
}

impl BscBuiltPayload {
    /// Initializes the payload with the given initial block.
    pub const fn new(
        block: Arc<RecoveredBlock<<EthPrimitives as NodePrimitives>::Block>>,
        fees: U256,
        requests: Option<Requests>,
        block_access_list: Option<Bytes>,
    ) -> Self {
        Self { block, fees, requests, sidecars: BlobSidecars::Empty, block_access_list }
    }

    /// Returns the built block (sealed).
    pub fn block(&self) -> &SealedBlock<Block> {
        self.block.sealed_block()
    }

    /// Returns the built block with recovered senders.
    pub fn recovered_block(&self) -> &RecoveredBlock<<EthPrimitives as NodePrimitives>::Block> {
        &self.block
    }

    /// Fees of the block
    pub const fn fees(&self) -> U256 {
        self.fees
    }

    /// Sets blob transactions sidecars on the payload.
    pub fn with_sidecars(mut self, sidecars: impl Into<BlobSidecars>) -> Self {
        self.sidecars = sidecars.into();
        self
    }

    /// Try converting built payload into [`ExecutionPayloadEnvelopeV3`].
    pub fn try_into_v3(
        self,
    ) -> Result<ExecutionPayloadEnvelopeV3, BscBuiltPayloadConversionError> {
        let Self { block, fees, sidecars, .. } = self;

        let blobs_bundle = match sidecars {
            BlobSidecars::Empty => BlobsBundleV1::empty(),
            BlobSidecars::Eip4844(sidecars) => BlobsBundleV1::from(sidecars),
            BlobSidecars::Eip7594(_) => {
                return Err(BscBuiltPayloadConversionError::UnexpectedEip7594Sidecars)
            }
        };

        Ok(ExecutionPayloadEnvelopeV3 {
            execution_payload: alloy_rpc_types_engine::ExecutionPayloadV3::from_block_unchecked(
                block.hash(),
                &Arc::unwrap_or_clone(block).into_block(),
            ),
            block_value: fees,
            should_override_builder: false,
            blobs_bundle,
        })
    }

    /// Try converting built payload into [`ExecutionPayloadEnvelopeV4`].
    pub fn try_into_v4(
        mut self,
    ) -> Result<ExecutionPayloadEnvelopeV4, BscBuiltPayloadConversionError> {
        let execution_requests = self.requests.take().unwrap_or_default();
        Ok(ExecutionPayloadEnvelopeV4 { execution_requests, envelope_inner: self.try_into_v3()? })
    }

    /// Try converting built payload into [`ExecutionPayloadEnvelopeV5`].
    pub fn try_into_v5(self) -> Result<ExecutionPayloadEnvelopeV5, BscBuiltPayloadConversionError> {
        let Self { block, fees, sidecars, requests, .. } = self;

        let blobs_bundle = match sidecars {
            BlobSidecars::Empty => BlobsBundleV2::empty(),
            BlobSidecars::Eip7594(sidecars) => BlobsBundleV2::from(sidecars),
            BlobSidecars::Eip4844(_) => {
                return Err(BscBuiltPayloadConversionError::UnexpectedEip4844Sidecars)
            }
        };

        Ok(ExecutionPayloadEnvelopeV5 {
            execution_payload: ExecutionPayloadV3::from_block_unchecked(
                block.hash(),
                &Arc::unwrap_or_clone(block).into_block(),
            ),
            block_value: fees,
            should_override_builder: false,
            blobs_bundle,
            execution_requests: requests.unwrap_or_default(),
        })
    }

    /// Try converting built payload into [`ExecutionPayloadEnvelopeV6`].
    pub fn try_into_v6(self) -> Result<ExecutionPayloadEnvelopeV6, BscBuiltPayloadConversionError> {
        let Self { block, fees, sidecars, requests, block_access_list, .. } = self;

        let block_access_list =
            block_access_list.ok_or(BscBuiltPayloadConversionError::MissingBlockAccessList)?;

        let blobs_bundle = match sidecars {
            BlobSidecars::Empty => BlobsBundleV2::empty(),
            BlobSidecars::Eip7594(sidecars) => BlobsBundleV2::from(sidecars),
            BlobSidecars::Eip4844(_) => {
                return Err(BscBuiltPayloadConversionError::UnexpectedEip4844Sidecars)
            }
        };

        Ok(ExecutionPayloadEnvelopeV6 {
            execution_payload: ExecutionPayloadV4::from_block_unchecked_with_bal(
                block.hash(),
                &Arc::unwrap_or_clone(block).into_block(),
                block_access_list,
            ),
            block_value: fees,
            should_override_builder: false,
            blobs_bundle,
            execution_requests: requests.unwrap_or_default(),
        })
    }
}

/// Error during [`BscBuiltPayload`] to execution payload envelope conversion.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BscBuiltPayloadConversionError {
    /// Unexpected EIP-7594 sidecars in payload.
    #[error("unexpected EIP-7594 sidecars")]
    UnexpectedEip7594Sidecars,
    /// Unexpected EIP-4844 sidecars in payload.
    #[error("unexpected EIP-4844 sidecars")]
    UnexpectedEip4844Sidecars,
    /// Missing block access list required for V6 envelopes.
    #[error("missing block access list")]
    MissingBlockAccessList,
}

impl BuiltPayload for BscBuiltPayload {
    type Primitives = EthPrimitives;

    fn block(&self) -> &SealedBlock<Block> {
        self.block.sealed_block()
    }

    fn fees(&self) -> U256 {
        self.fees
    }

    fn block_access_list(&self) -> Option<&Bytes> {
        self.block_access_list.as_ref()
    }

    fn requests(&self) -> Option<Requests> {
        self.requests.clone()
    }
}

impl From<BscBuiltPayload> for ExecutionPayloadV1 {
    fn from(value: BscBuiltPayload) -> Self {
        Self::from_block_unchecked(
            value.block().hash(),
            &Arc::unwrap_or_clone(value.block).into_block(),
        )
    }
}

impl From<BscBuiltPayload> for ExecutionPayloadEnvelopeV2 {
    fn from(value: BscBuiltPayload) -> Self {
        let BscBuiltPayload { block, fees, .. } = value;

        Self {
            block_value: fees,
            execution_payload: ExecutionPayloadFieldV2::from_block_unchecked(
                block.hash(),
                &Arc::unwrap_or_clone(block).into_block(),
            ),
        }
    }
}

impl TryFrom<BscBuiltPayload> for ExecutionPayloadEnvelopeV3 {
    type Error = BscBuiltPayloadConversionError;

    fn try_from(value: BscBuiltPayload) -> Result<Self, Self::Error> {
        value.try_into_v3()
    }
}

impl TryFrom<BscBuiltPayload> for ExecutionPayloadEnvelopeV4 {
    type Error = BscBuiltPayloadConversionError;

    fn try_from(value: BscBuiltPayload) -> Result<Self, Self::Error> {
        value.try_into_v4()
    }
}

impl TryFrom<BscBuiltPayload> for ExecutionPayloadEnvelopeV5 {
    type Error = BscBuiltPayloadConversionError;

    fn try_from(value: BscBuiltPayload) -> Result<Self, Self::Error> {
        value.try_into_v5()
    }
}

impl TryFrom<BscBuiltPayload> for ExecutionPayloadEnvelopeV6 {
    type Error = BscBuiltPayloadConversionError;

    fn try_from(value: BscBuiltPayload) -> Result<Self, Self::Error> {
        value.try_into_v6()
    }
}

impl From<BscBuiltPayload> for ExecutionData {
    fn from(value: BscBuiltPayload) -> Self {
        let (payload, sidecar) = ExecutionPayload::from_block_unchecked_with_extras(
            value.block().hash(),
            &value.recovered_block().clone().into_block(),
            value.block_access_list().cloned(),
        );
        Self { payload, sidecar }
    }
}
