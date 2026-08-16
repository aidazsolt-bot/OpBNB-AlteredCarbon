use alloy_eips::eip2718::Encodable2718;
use alloy_rpc_types_engine::{
    payload::{ExecutionPayloadBodyV1, ExecutionPayloadFieldV2, ExecutionPayloadInputV2},
    ExecutionPayload, ExecutionPayloadSidecar, ExecutionPayloadV1, ExecutionPayloadV2,
    ExecutionPayloadV3, PayloadError,
};
use reth_primitives::{Block, SealedBlock, TransactionSigned};

pub fn try_payload_v1_to_block(payload: ExecutionPayloadV1) -> Result<Block, PayloadError> {
    payload.try_into_block::<TransactionSigned>()
}

pub fn try_payload_v2_to_block(payload: ExecutionPayloadV2) -> Result<Block, PayloadError> {
    payload.try_into_block::<TransactionSigned>()
}

pub fn try_payload_v3_to_block(payload: ExecutionPayloadV3) -> Result<Block, PayloadError> {
    payload.try_into_block::<TransactionSigned>()
}

pub fn try_into_sealed_block(
    payload: ExecutionPayload,
    sidecar: &ExecutionPayloadSidecar,
) -> Result<SealedBlock, PayloadError> {
    payload.try_into_block_with_sidecar(sidecar).map(SealedBlock::seal_slow)
}

pub fn block_to_payload(value: SealedBlock) -> ExecutionPayload {
    ExecutionPayload::from_block_unchecked(value.hash(), &value.clone_block()).0
}

pub fn block_to_payload_v1(value: SealedBlock) -> ExecutionPayloadV1 {
    ExecutionPayloadV1::from_block_unchecked(value.hash(), &value.clone_block())
}

pub fn block_to_payload_v2(value: SealedBlock) -> ExecutionPayloadV2 {
    ExecutionPayloadV2::from_block_unchecked(value.hash(), &value.clone_block())
}

pub fn block_to_payload_v3(value: SealedBlock) -> ExecutionPayloadV3 {
    ExecutionPayloadV3::from_block_unchecked(value.hash(), &value.clone_block())
}

pub fn convert_block_to_payload_field_v2(value: SealedBlock) -> ExecutionPayloadFieldV2 {
    ExecutionPayloadFieldV2::from_block_unchecked(value.hash(), &value.clone_block())
}

pub fn convert_payload_field_v2_to_payload(value: ExecutionPayloadFieldV2) -> ExecutionPayload {
    value.into_payload()
}

pub fn convert_payload_v2_to_payload_input_v2(
    value: ExecutionPayloadV2,
    is_shanghai_active: bool,
) -> ExecutionPayloadInputV2 {
    ExecutionPayloadInputV2 {
        execution_payload: value.payload_inner,
        withdrawals: is_shanghai_active.then_some(value.withdrawals),
    }
}

pub fn convert_payload_input_v2_to_payload(value: ExecutionPayloadInputV2) -> ExecutionPayload {
    value.into_payload()
}

pub fn convert_to_payload_body_v1<T: Encodable2718>(block: Block<T>) -> ExecutionPayloadBodyV1 {
    ExecutionPayloadBodyV1::from_block(block)
}
