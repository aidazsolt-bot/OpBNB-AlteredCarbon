use alloy_primitives::{Signature as PrimitiveSignature, U256};
use reth_primitives::TxType;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Signature {
    pub r: U256,
    pub s: U256,
    pub v: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_parity: Option<bool>,
}

pub fn from_legacy_primitive_signature(
    signature: PrimitiveSignature,
    chain_id: Option<u64>,
) -> Signature {
    let v = if let Some(chain_id) = chain_id {
        if signature.v() { chain_id * 2 + 36 } else { chain_id * 2 + 35 }
    } else if signature.v() {
        28
    } else {
        27
    };
    Signature { r: signature.r(), s: signature.s(), v: U256::from(v), y_parity: None }
}

pub fn from_typed_primitive_signature(signature: PrimitiveSignature) -> Signature {
    Signature {
        r: signature.r(),
        s: signature.s(),
        v: U256::from(u8::from(signature.v())),
        y_parity: Some(signature.v().into()),
    }
}

pub fn from_primitive_signature(
    signature: PrimitiveSignature,
    tx_type: TxType,
    chain_id: Option<u64>,
) -> Signature {
    match tx_type {
        TxType::Legacy => from_legacy_primitive_signature(signature, chain_id),
        _ => from_typed_primitive_signature(signature),
    }
}
