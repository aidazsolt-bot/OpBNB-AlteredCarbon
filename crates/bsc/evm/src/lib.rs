//! EVM config for BSC.

#![allow(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(feature = "bsc")]

extern crate alloc;

use std::{convert::Infallible, sync::Arc};

use alloy_consensus::Header;
use alloy_evm::{
    eth::{EthBlockExecutionCtx, EthBlockExecutorFactory},
    EthEvmFactory, FromRecoveredTx, FromTxWithEncoded,
};
use alloy_primitives::U256;
use reth_bsc_chainspec::BscChainSpec;
use reth_chainspec::ChainSpec;
use reth_bsc_forks::BscHardforks;
use reth_chainspec::{EthChainSpec, EthereumHardforks, Hardforks};
use reth_evm::{
    eth::NextEvmEnvAttributes, precompiles::PrecompilesMap, ConfigureEngineEvm, ConfigureEvm,
    EvmEnv, EvmFactory, NextBlockEnvAttributes, TransactionEnvMut,
};
use reth_evm_ethereum::RethReceiptBuilder;
use reth_ethereum_primitives::{Block, BlockBody, EthPrimitives, Receipt, TransactionSigned};
use reth_primitives_traits::{NodePrimitives, SealedBlock, SealedHeader};
use reth_ethereum_forks::Head;
use revm::{
    context::BlockEnv,
    primitives::hardfork::SpecId,
};

mod bsc_evm;
mod config;
pub use config::{revm_spec, revm_spec_by_timestamp_after_shanghai};
mod error;
pub use error::BscBlockExecutionError;
mod execute;
pub use execute::*;
mod factory;
pub use factory::BscEvmFactory;
mod patch_hertz;
mod post_execution;
mod pre_execution;
mod precompiles;
mod transaction;
pub use transaction::BscTxEnv;

/// BSC node primitives used by [`ConfigureEvm`].
pub type BscNodePrimitives = EthPrimitives;

/// BSC-related EVM configuration.
#[derive(Debug, Clone)]
pub struct BscEvmConfig {
    executor_factory: BscBlockExecutorFactory,
    block_assembler: BscBlockAssembler,
}

/// BSC block executor factory.
#[derive(Debug, Clone)]
pub struct BscBlockExecutorFactory {
    inner: EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, BscEvmFactory>,
    chain_spec: Arc<BscChainSpec>,
}

/// BSC block assembler.
#[derive(Debug, Clone)]
pub struct BscBlockAssembler {
    chain_spec: Arc<BscChainSpec>,
}

impl BscBlockExecutorFactory {
    fn spec(&self) -> &Arc<BscChainSpec> {
        &self.chain_spec
    }
}

impl BscBlockAssembler {
    const fn new(chain_spec: Arc<BscChainSpec>) -> Self {
        Self { chain_spec }
    }
}

impl BscEvmConfig {
    /// Creates a new BSC EVM configuration with the given chain spec.
    pub fn new(chain_spec: Arc<BscChainSpec>) -> Self {
        Self {
            block_assembler: BscBlockAssembler::new(chain_spec.clone()),
            executor_factory: BscBlockExecutorFactory {
                inner: EthBlockExecutorFactory::new(
                    RethReceiptBuilder::default(),
                    chain_spec.inner.clone().into(),
                    BscEvmFactory,
                ),
                chain_spec,
            },
        }
    }

    /// Returns the chain spec associated with this configuration.
    pub fn chain_spec(&self) -> &Arc<BscChainSpec> {
        self.executor_factory.spec()
    }
}

impl ConfigureEvm for BscEvmConfig {
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = BscBlockExecutorFactory;
    type BlockAssembler = BscBlockAssembler;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        &self.executor_factory
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        &self.block_assembler
    }

    fn evm_env(&self, header: &Header) -> Result<EvmEnv<SpecId>, Self::Error> {
        let mut env = EvmEnv::for_eth_block(
            header,
            self.chain_spec().as_ref(),
            self.chain_spec().chain().id(),
            self.chain_spec().blob_params_at_timestamp(header.timestamp),
        );
        let spec_id = revm_spec(
            self.chain_spec(),
            &Head {
                number: header.number,
                timestamp: header.timestamp,
                difficulty: header.difficulty,
                total_difficulty: U256::ZERO,
                hash: Default::default(),
            },
        );
        env.cfg_env.spec = spec_id;
        env.cfg_env.disable_block_gas_limit = true;
        Ok(env)
    }

    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &NextBlockEnvAttributes,
    ) -> Result<EvmEnv<SpecId>, Self::Error> {
        let mut env = EvmEnv::for_eth_next_block(
            parent,
            NextEvmEnvAttributes {
                timestamp: attributes.timestamp,
                suggested_fee_recipient: attributes.suggested_fee_recipient,
                prev_randao: attributes.prev_randao,
                gas_limit: attributes.gas_limit,
                slot_number: attributes.slot_number,
            },
            self.chain_spec()
                .next_block_base_fee(parent, attributes.timestamp)
                .unwrap_or_default(),
            self.chain_spec().as_ref(),
            self.chain_spec().chain().id(),
            self.chain_spec().blob_params_at_timestamp(attributes.timestamp),
        );
        env.cfg_env.disable_block_gas_limit = true;
        Ok(env)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<Block>,
    ) -> Result<EthBlockExecutionCtx<'a>, Self::Error> {
        Ok(EthBlockExecutionCtx {
            tx_count_hint: Some(block.transaction_count()),
            parent_hash: block.header().parent_hash,
            parent_beacon_block_root: block.header().parent_beacon_block_root,
            ommers: &block.body().ommers,
            withdrawals: block.body().withdrawals.as_ref().map(|w| alloc::borrow::Cow::Borrowed(w.as_slice())),
            extra_data: block.header().extra_data.clone(),
            slot_number: block.header().slot_number,
        })
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader,
        attributes: NextBlockEnvAttributes,
    ) -> Result<EthBlockExecutionCtx<'_>, Self::Error> {
        Ok(EthBlockExecutionCtx {
            tx_count_hint: None,
            parent_hash: parent.hash(),
            parent_beacon_block_root: attributes.parent_beacon_block_root,
            ommers: &[],
            withdrawals: attributes.withdrawals.map(|w| alloc::borrow::Cow::Owned(w.into_inner())),
            extra_data: attributes.extra_data,
            slot_number: attributes.slot_number,
        })
    }
}

impl alloy_evm::block::BlockExecutorFactory for BscBlockExecutorFactory {
    type EvmFactory = BscEvmFactory;
    type TxExecutionResult = alloy_evm::eth::EthTxResult<
        <BscEvmFactory as EvmFactory>::HaltReason,
        alloy_consensus::TxType,
    >;
    type ExecutionCtx<'a> = EthBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type Executor<'a, DB, I> = alloy_evm::eth::EthBlockExecutor<
        'a,
        <BscEvmFactory as EvmFactory>::Evm<DB, I>,
        &'a Arc<ChainSpec>,
        &'a RethReceiptBuilder,
    >
    where
        DB: alloy_evm::block::StateDB,
        I: revm::Inspector<<BscEvmFactory as EvmFactory>::Context<DB>>;

    fn evm_factory(&self) -> &Self::EvmFactory {
        self.inner.evm_factory()
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        evm: <Self::EvmFactory as EvmFactory>::Evm<DB, I>,
        ctx: Self::ExecutionCtx<'a>,
    ) -> Self::Executor<'a, DB, I>
    where
        DB: alloy_evm::block::StateDB,
        I: revm::Inspector<<Self::EvmFactory as EvmFactory>::Context<DB>>,
    {
        self.inner.create_executor(evm, ctx)
    }
}

impl reth_evm::execute::BlockAssembler<BscBlockExecutorFactory> for BscBlockAssembler {
    type Block = Block;

    fn assemble_block(
        &self,
        input: reth_evm::execute::BlockAssemblerInput<'_, '_, BscBlockExecutorFactory>,
    ) -> Result<Self::Block, reth_execution_errors::BlockExecutionError> {
        use reth_evm::execute::BlockAssembler;
        BlockAssembler::assemble_block(
            &reth_evm_ethereum::EthBlockAssembler::new(self.chain_spec.inner.clone().into()),
            input,
        )
    }
}

impl ConfigureEngineEvm<alloy_rpc_types_engine::ExecutionData> for BscEvmConfig {
    fn evm_env_for_payload(
        &self,
        payload: &alloy_rpc_types_engine::ExecutionData,
    ) -> Result<EvmEnv<SpecId>, Self::Error> {
        use alloy_primitives::U256;
        use reth_chainspec::EthereumHardforks;
        use reth_primitives_traits::constants::MAX_TX_GAS_LIMIT_OSAKA;
        use revm::context::CfgEnv;
        use revm::context_interface::block::BlobExcessGasAndPrice;

        let timestamp = payload.payload.timestamp();
        let block_number = payload.payload.block_number();
        let blob_params = self.chain_spec().blob_params_at_timestamp(timestamp);
        let spec = revm_spec(
            self.chain_spec(),
            &Head {
                number: block_number,
                timestamp,
                difficulty: if self.chain_spec().is_paris_active_at_block(block_number) {
                    U256::ZERO
                } else {
                    payload.payload.as_v1().prev_randao.into()
                },
                total_difficulty: U256::ZERO,
                hash: Default::default(),
            },
        );

        let mut cfg_env = CfgEnv::new()
            .with_chain_id(self.chain_spec().chain().id())
            .with_spec_and_mainnet_gas_params(spec);

        if let Some(blob_params) = &blob_params {
            cfg_env.set_max_blobs_per_tx(blob_params.max_blobs_per_tx);
        }

        if self.chain_spec().is_osaka_active_at_timestamp(timestamp) {
            cfg_env.tx_gas_limit_cap = Some(MAX_TX_GAS_LIMIT_OSAKA);
        }
        cfg_env.disable_block_gas_limit = true;

        let blob_excess_gas_and_price =
            payload.payload.excess_blob_gas().zip(blob_params).map(|(excess_blob_gas, params)| {
                let blob_gasprice = params.calc_blob_fee(excess_blob_gas);
                BlobExcessGasAndPrice { excess_blob_gas, blob_gasprice }
            });

        let block_env = BlockEnv {
            number: U256::from(block_number),
            beneficiary: payload.payload.fee_recipient(),
            timestamp: U256::from(timestamp),
            difficulty: if spec >= SpecId::MERGE {
                U256::ZERO
            } else {
                payload.payload.as_v1().prev_randao.into()
            },
            prevrandao: (spec >= SpecId::MERGE).then(|| payload.payload.as_v1().prev_randao),
            gas_limit: payload.payload.gas_limit(),
            basefee: payload.payload.saturated_base_fee_per_gas(),
            blob_excess_gas_and_price,
            slot_num: payload.payload.as_v4().map(|v4| v4.slot_number).unwrap_or_default(),
        };

        Ok(EvmEnv { cfg_env, block_env })
    }

    fn context_for_payload<'a>(
        &self,
        payload: &'a alloy_rpc_types_engine::ExecutionData,
    ) -> Result<EthBlockExecutionCtx<'a>, Self::Error> {
        Ok(EthBlockExecutionCtx {
            tx_count_hint: Some(payload.payload.transactions().len()),
            parent_hash: payload.parent_hash(),
            parent_beacon_block_root: payload.sidecar.parent_beacon_block_root(),
            ommers: &[],
            withdrawals: payload.payload.withdrawals().map(|w| alloc::borrow::Cow::Borrowed(w.as_slice())),
            extra_data: payload.payload.as_v1().extra_data.clone(),
            slot_number: payload.payload.as_v4().map(|v4| v4.slot_number),
        })
    }

    fn tx_iterator_for_payload(
        &self,
        payload: &alloy_rpc_types_engine::ExecutionData,
    ) -> Result<impl reth_evm::ExecutableTxIterator<Self>, Self::Error> {
        use alloy_eips::Decodable2718;
        use alloy_primitives::Bytes;
        use reth_primitives_traits::{SignedTransaction, TxTy};
        use reth_storage_errors::any::AnyError;

        let txs = payload.payload.transactions().clone();
        let convert = |tx: Bytes| {
            let tx =
                TxTy::<Self::Primitives>::decode_2718_exact(tx.as_ref()).map_err(AnyError::new)?;
            let signer = tx.try_recover().map_err(AnyError::new)?;
            Ok::<_, AnyError>(tx.with_signer(signer))
        };

        Ok((txs, convert))
    }
}
