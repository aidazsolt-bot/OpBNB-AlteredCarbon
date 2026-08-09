//! Simplified BSC precompiles provider using stock revm precompiles.
//!
//! BSC-specific precompile implementations live in submodules and are wired in
//! incrementally; this wrapper keeps compilation working on stock revm 41.

use revm::{
    handler::EthPrecompiles,
    precompile::Precompiles,
    primitives::hardfork::SpecId,
};

/// BSC precompile provider.
#[derive(Debug, Clone)]
pub struct BscPrecompiles {
    inner: EthPrecompiles,
}

impl BscPrecompiles {
    /// Create a new precompile provider for the given EVM spec.
    #[inline]
    pub fn new(spec: SpecId) -> Self {
        Self { inner: EthPrecompiles::new(spec) }
    }

    #[inline]
    pub fn precompiles(&self) -> &'static Precompiles {
        self.inner.precompiles
    }
}

/// No-op trace context hooks used by the BSC handler.
pub struct PrecompileTraceContext;

impl PrecompileTraceContext {
    pub fn from_parts(
        _block_number: u64,
        _spec: SpecId,
        _is_system_tx: bool,
        _tx_hash: Option<alloy_primitives::B256>,
        _tx_to: Option<alloy_primitives::Address>,
        _tx_selector: Option<String>,
        _tx_input_len: usize,
    ) -> Self {
        Self
    }
}

pub fn push_precompile_trace_context(_ctx: PrecompileTraceContext) {}

pub fn pop_precompile_trace_context() {}
