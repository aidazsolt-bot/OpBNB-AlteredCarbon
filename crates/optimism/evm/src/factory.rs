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
    /// Inject Fermat/Haber precompiles and Wright `skip_l1_data_fee` when the chainspec says so.
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
    if chain_spec.fork(OptimismHardfork::Wright).active_at_timestamp(timestamp) {
        evm.ctx_mut().chain.skip_l1_data_fee = true;
    }
}

fn u256_to_u64(value: U256) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
