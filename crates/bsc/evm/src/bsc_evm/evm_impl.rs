use crate::{bsc_evm::api::BscEvm, transaction::BscTxEnv};
use alloy_evm::{Evm, EvmEnv, precompiles::PrecompilesMap};
use alloy_primitives::{Address, Bytes};
use reth_evm::Database;
use revm::{
    context::ContextSetters,
    context::{BlockEnv, CfgEnv},
    context_interface::result::{EVMError, HaltReason, ResultAndState},
    handler::Handler,
    inspector::NoOpInspector,
    primitives::hardfork::SpecId,
    ExecuteEvm, SystemCallEvm,
};

impl<DB, I> Evm for BscEvm<DB, I>
where
    DB: Database,
    I: revm::Inspector<crate::bsc_evm::api::BscContext<DB>>,
{
    type DB = DB;
    type Tx = BscTxEnv;
    type Error = EVMError<DB::Error>;
    type HaltReason = HaltReason;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;
    type Inspector = I;

    fn cfg_env(&self) -> &CfgEnv<Self::Spec> {
        &self.inner.ctx.cfg
    }

    fn chain_id(&self) -> u64 {
        self.cfg_env().chain_id
    }

    fn block(&self) -> &BlockEnv {
        &self.inner.ctx.block
    }

    fn transact_raw(
        &mut self,
        mut tx: Self::Tx,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        self.prepare_tx_for_execution(&mut tx);
        if tx.is_system_transaction {
            self.fund_beneficiary_for_system_tx_replay(tx.base.value);
        }
        self.inner.ctx.set_tx(tx);
        let result = crate::bsc_evm::handler::BscHandler::new().run(self)?;
        let state = ExecuteEvm::finalize(self);
        Ok(ResultAndState::new(result, state))
    }

    fn transact_system_call(
        &mut self,
        caller: Address,
        contract: Address,
        data: Bytes,
    ) -> Result<ResultAndState<Self::HaltReason>, Self::Error> {
        let result = self.inner.system_call_one_with_caller(caller, contract, data)?;
        let state = ExecuteEvm::finalize(self);
        Ok(ResultAndState::new(result, state))
    }

    fn finish(self) -> (Self::DB, EvmEnv<Self::Spec, Self::BlockEnv>) {
        let revm::context::Context { block: block_env, cfg: cfg_env, journaled_state, .. } =
            self.inner.ctx;

        (journaled_state.database, EvmEnv { block_env, cfg_env })
    }

    fn set_inspector_enabled(&mut self, enabled: bool) {
        self.inspect = enabled;
    }

    fn components(&self) -> (&Self::DB, &Self::Inspector, &Self::Precompiles) {
        (
            &self.inner.ctx.journaled_state.database,
            &self.inner.inspector,
            &self.inner.precompiles,
        )
    }

    fn components_mut(&mut self) -> (&mut Self::DB, &mut Self::Inspector, &mut Self::Precompiles) {
        (
            &mut self.inner.ctx.journaled_state.database,
            &mut self.inner.inspector,
            &mut self.inner.precompiles,
        )
    }
}
