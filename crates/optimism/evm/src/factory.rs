//! opBNB Fermat/Haber/Wright overlays applied after stock [`OpEvmFactory`] creates an EVM.
//!
//! Stock `alloy-op-evm` only wires [`OpEvmFactory`] / [`PostExecEvmFactoryAdapter`] into
//! `OpBlockExecutorFactory`. Overlays therefore run as a post-create hook via
//! [`OpBnbOverlayFactory`], not a separate factory type parameter.

use alloy_evm::{Database, Evm, EvmEnv, EvmFactory, IntoTxEnv};
use alloy_op_evm::{
    post_exec::{PostExecEvmFactoryAdapter, PostExecEvmFactoryHooks},
    OpEvm, OpEvmContext, OpEvmFactory,
};
use alloy_primitives::U256;
use core::fmt::Debug;
use op_revm::{OpSpecId, OpTransaction};
use reth_evm::precompiles::PrecompilesMap;
use reth_optimism_forks::{Hardforks, OpHardforks, OptimismHardfork};
use revm::{
    context::{BlockEnv, TxEnv},
    inspector::NoOpInspector,
    Inspector,
};

use crate::{config::opbnb_precompile_flags, opbnb_precompiles::opbnb_precompiles};

/// Factory that can apply opBNB precompile / Wright L1-fee overlays onto EVMs it creates.
pub trait OpBnbOverlayFactory: EvmFactory<Spec = OpSpecId, BlockEnv = BlockEnv> {
    /// Inject Fermat/Haber precompiles and Wright L1-fee mode when the chainspec says so.
    ///
    /// Wright sets `L1BlockInfo::skip_l1_data_fee = true`. In `op-revm` that flag means
    /// **skip L1 data fee only when `tx.gas_price() == 0`** (gasless), matching
    /// `bnb-chain/op-geth` `state_transition.go` (`GasPrice==0 && IsWright`). Paid txs still
    /// pay L1 cost.
    fn apply_opbnb_overlays<DB, I>(
        chain_spec: &(impl Hardforks + OpHardforks),
        evm: &mut Self::Evm<DB, I>,
        input: &EvmEnv<OpSpecId, BlockEnv>,
    ) where
        DB: Database,
        I: Inspector<Self::Context<DB>>;

    /// [`EvmFactory::create_evm`] then [`Self::apply_opbnb_overlays`].
    fn create_evm_with_opbnb_overlays<DB: Database>(
        &self,
        chain_spec: &(impl Hardforks + OpHardforks),
        db: DB,
        input: EvmEnv<OpSpecId, BlockEnv>,
    ) -> Self::Evm<DB, NoOpInspector> {
        let mut evm = self.create_evm(db, input.clone());
        Self::apply_opbnb_overlays(chain_spec, &mut evm, &input);
        evm
    }

    /// [`EvmFactory::create_evm_with_inspector`] then [`Self::apply_opbnb_overlays`].
    fn create_evm_with_inspector_and_opbnb_overlays<DB, I>(
        &self,
        chain_spec: &(impl Hardforks + OpHardforks),
        db: DB,
        input: EvmEnv<OpSpecId, BlockEnv>,
        inspector: I,
    ) -> Self::Evm<DB, I>
    where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        let mut evm = self.create_evm_with_inspector(db, input.clone(), inspector);
        Self::apply_opbnb_overlays(chain_spec, &mut evm, &input);
        evm
    }
}

impl<Tx> OpBnbOverlayFactory for OpEvmFactory<Tx>
where
    Tx: IntoTxEnv<Tx> + Into<OpTransaction<TxEnv>> + Default + Clone + Debug,
{
    fn apply_opbnb_overlays<DB, I>(
        chain_spec: &(impl Hardforks + OpHardforks),
        evm: &mut Self::Evm<DB, I>,
        input: &EvmEnv<OpSpecId, BlockEnv>,
    ) where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        apply_opbnb_overlays_to_op_evm(chain_spec, evm, input);
    }
}

impl<F> OpBnbOverlayFactory for PostExecEvmFactoryAdapter<F>
where
    F: PostExecEvmFactoryHooks + OpBnbOverlayFactory,
{
    fn apply_opbnb_overlays<DB, I>(
        chain_spec: &(impl Hardforks + OpHardforks),
        evm: &mut Self::Evm<DB, I>,
        input: &EvmEnv<OpSpecId, BlockEnv>,
    ) where
        DB: Database,
        I: Inspector<Self::Context<DB>>,
    {
        // Adapter derefs to the inner factory EVM (`OpEvm` for stock `OpEvmFactory`).
        F::apply_opbnb_overlays(chain_spec, &mut **evm, input);
    }
}

fn apply_opbnb_overlays_to_op_evm<DB, I, Tx>(
    chain_spec: &(impl Hardforks + OpHardforks),
    evm: &mut OpEvm<DB, I, PrecompilesMap, Tx>,
    input: &EvmEnv<OpSpecId, BlockEnv>,
) where
    DB: Database,
    I: Inspector<OpEvmContext<DB>>,
    Tx: IntoTxEnv<Tx> + Into<OpTransaction<TxEnv>>,
{
    let block_number = u256_to_u64(input.block_env.number);
    let timestamp = u256_to_u64(input.block_env.timestamp);
    let flags = opbnb_precompile_flags(chain_spec, block_number, timestamp);
    if !flags.is_empty() {
        *Evm::components_mut(evm).2 = opbnb_precompiles(input.cfg_env.spec, flags);
    }
    // Enable Wright gasless-L1-fee mode (op-revm still requires gas_price==0 to skip).
    if chain_spec.fork(OptimismHardfork::Wright).active_at_timestamp(timestamp) {
        evm.ctx_mut().chain.skip_l1_data_fee = true;
    }
}

fn u256_to_u64(value: U256) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_evm::EvmEnv;
    use alloy_op_evm::OpTx;
    use reth_optimism_chainspec::OPBNB_MAINNET;
    use revm::{context::BlockEnv, database::EmptyDB};

    #[test]
    fn wright_sets_skip_l1_data_fee_flag_only() {
        // Mainnet Wright timestamp / first block (~32984677 @ 1724738400).
        let wright_ts = 1_724_738_400_u64;
        let chain_spec = OPBNB_MAINNET.clone();
        assert!(chain_spec.fork(OptimismHardfork::Wright).active_at_timestamp(wright_ts));
        assert!(!chain_spec.fork(OptimismHardfork::Wright).active_at_timestamp(wright_ts - 1));

        let factory = OpEvmFactory::<OpTx>::default();
        let mut pre = factory.create_evm_with_opbnb_overlays(
            chain_spec.as_ref(),
            EmptyDB::default(),
            EvmEnv {
                cfg_env: Default::default(),
                block_env: BlockEnv {
                    number: U256::from(32_984_677_u64),
                    timestamp: U256::from(wright_ts - 1),
                    ..Default::default()
                },
            },
        );
        assert!(
            !pre.ctx_mut().chain.skip_l1_data_fee,
            "pre-Wright must leave skip_l1_data_fee false"
        );

        let mut post = factory.create_evm_with_opbnb_overlays(
            chain_spec.as_ref(),
            EmptyDB::default(),
            EvmEnv {
                cfg_env: Default::default(),
                block_env: BlockEnv {
                    number: U256::from(32_984_677_u64),
                    timestamp: U256::from(wright_ts),
                    ..Default::default()
                },
            },
        );
        assert!(
            post.ctx_mut().chain.skip_l1_data_fee,
            "Wright must enable skip_l1_data_fee (gas_price==0 gate lives in op-revm)"
        );
    }
}
