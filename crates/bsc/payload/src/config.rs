pub use alloy_eips::eip1559::calculate_block_gas_limit;
use alloy_eips::eip1559::ETHEREUM_BLOCK_GAS_LIMIT_30M;
use alloy_primitives::Bytes;

/// Settings for the BSC payload builder.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BscBuilderConfig {
    /// Desired gas limit.
    pub desired_gas_limit: u64,
    /// Waits for the first payload to be built if there is no payload built when the payload is
    /// being resolved.
    pub await_payload_on_missing: bool,
    /// Extra data for built blocks.
    pub extra_data: Bytes,
}

impl Default for BscBuilderConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BscBuilderConfig {
    /// Create new payload builder config.
    pub const fn new() -> Self {
        Self {
            desired_gas_limit: ETHEREUM_BLOCK_GAS_LIMIT_30M,
            await_payload_on_missing: true,
            extra_data: Bytes::new(),
        }
    }

    /// Set desired gas limit.
    pub const fn with_gas_limit(mut self, desired_gas_limit: u64) -> Self {
        self.desired_gas_limit = desired_gas_limit;
        self
    }

    /// Configures whether the initial payload should be awaited when the payload job is being
    /// resolved and no payload has been built yet.
    pub const fn with_await_payload_on_missing(mut self, await_payload_on_missing: bool) -> Self {
        self.await_payload_on_missing = await_payload_on_missing;
        self
    }

    /// Set the extra data for built blocks.
    pub fn with_extra_data(mut self, extra_data: Bytes) -> Self {
        self.extra_data = extra_data;
        self
    }

    /// Returns the gas limit for the next block based on parent and desired gas limits.
    pub fn gas_limit(&self, parent_gas_limit: u64) -> u64 {
        calculate_block_gas_limit(parent_gas_limit, self.desired_gas_limit)
    }
}
