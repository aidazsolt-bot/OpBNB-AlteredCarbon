//! OP-Reth hard forks.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

extern crate alloc;

pub mod hardfork;

mod dev;

pub use alloy_op_hardforks::{OpHardfork, OpHardforks};
pub use dev::DEV_HARDFORKS;
pub use hardfork::OptimismHardfork;
pub use reth_ethereum_forks::{EthereumHardforks, Hardforks};

/// Extends [`EthereumHardforks`] with optimism helper methods.
pub trait OptimismHardforks: EthereumHardforks + Hardforks {
    /// Convenience method to check if [`OptimismHardfork::Bedrock`] is active at a given block
    /// number.
    fn is_bedrock_active_at_block(&self, block_number: u64) -> bool {
        self.fork(OptimismHardfork::Bedrock).active_at_block(block_number)
    }

    /// Returns `true` if [`Canyon`](OptimismHardfork::Canyon) is active at given block timestamp.
    fn is_canyon_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Canyon).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Ecotone`](OptimismHardfork::Ecotone) is active at given block timestamp.
    fn is_ecotone_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Ecotone).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Fjord`](OptimismHardfork::Fjord) is active at given block timestamp.
    fn is_fjord_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Fjord).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Granite`](OptimismHardfork::Granite) is active at given block timestamp.
    fn is_granite_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Granite).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Holocene`](OptimismHardfork::Holocene) is active at given block
    /// timestamp.
    fn is_holocene_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Holocene).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Regolith`](OptimismHardfork::Regolith) is active at given block
    /// timestamp.
    fn is_regolith_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Regolith).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Fermat`](OptimismHardfork::Fermat) is active at the given block number.
    ///
    /// Fermat is a block fork in bnb-chain/op-geth (`Fermat *big.Int`), not a timestamp fork.
    fn is_fermat_active_at_block(&self, block_number: u64) -> bool {
        self.fork(OptimismHardfork::Fermat).active_at_block(block_number)
    }

    /// Returns `true` if [`Haber`](OptimismHardfork::Haber) is active at given block timestamp.
    fn is_haber_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Haber).active_at_timestamp(timestamp)
    }

    /// Convenience method to check if [`OptimismHardfork::Wright`] is active at a given timestamp.
    fn is_wright_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Wright).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Snow`](OptimismHardfork::Snow) is active at given block timestamp.
    fn is_snow_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Snow).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Volta`](OptimismHardfork::Volta) is active at given block timestamp.
    fn is_volta_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Volta).active_at_timestamp(timestamp)
    }

    /// Returns `true` if [`Fourier`](OptimismHardfork::Fourier) is active at given block timestamp.
    fn is_fourier_active_at_timestamp(&self, timestamp: u64) -> bool {
        self.fork(OptimismHardfork::Fourier).active_at_timestamp(timestamp)
    }

    /// opBNB L2 block interval in milliseconds for the active hardfork at `timestamp`.
    ///
    /// Pre-Volta: 1000ms, Volta: 500ms, Fourier: 250ms (bnb-chain/opbnb rollup config).
    fn opbnb_block_interval_ms_at_timestamp(&self, timestamp: u64) -> u64 {
        if self.is_fourier_active_at_timestamp(timestamp) {
            250
        } else if self.is_volta_active_at_timestamp(timestamp) {
            500
        } else {
            1000
        }
    }
}
