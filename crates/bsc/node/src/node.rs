//! BSC Node types config.

use crate::BscEngineValidator;
use std::sync::Arc;

use alloy_network::Ethereum;
use alloy_rpc_types_engine::{ExecutionData, PayloadAttributes};
use reth_bsc_chainspec::BscChainSpec;
use reth_bsc_consensus::Parlia;
use reth_bsc_engine::BscEngineTypes;
use reth_bsc_evm::BscEvmConfig;
use reth_bsc_payload_builder::{BscBuilderConfig, BscPayloadBuilder};
use reth_chainspec::{EthChainSpec, EthereumHardforks, Hardforks};
use reth_engine_primitives::EngineTypes;
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::{
    eth::spec::EthExecutorSpec, ConfigureEvm, NextBlockEnvAttributes,
};
use reth_network::{primitives::BasicNetworkPrimitives, NetworkHandle, PeersInfo};
use reth_node_api::{
    AddOnsContext, FullNodeComponents, HeaderTy, NodeAddOns,
    PrimitivesTy, TxTy,
};
use reth_node_builder::{
    components::{
        BasicPayloadServiceBuilder, ComponentsBuilder, ConsensusBuilder, ExecutorBuilder,
        NetworkBuilder, PayloadBuilderBuilder, PoolBuilder, TxPoolBuilder,
    },
    node::{FullNodeTypes, NodeTypes},
    rpc::{
        BasicEngineApiBuilder, BasicEngineValidatorBuilder, EngineApiBuilder, EngineValidatorAddOn,
        EngineValidatorBuilder, EthApiBuilder, EthApiCtx, Identity, PayloadValidatorBuilder,
        RethRpcAddOns, RpcAddOns, RpcHandle,
    },
    BuilderContext, Node, NodeAdapter, PayloadBuilderConfig,
};
use reth_payload_primitives::PayloadTypes;
use reth_primitives::parlia::ParliaConfig;
use reth_provider::EthStorage;
use reth_rpc::eth::core::{EthApiFor, EthRpcConverterFor};
use reth_rpc_builder::middleware::RethRpcMiddleware;
use reth_rpc_eth_api::{
    helpers::pending_block::BuildPendingEnv,
    RpcConvert, RpcTypes, SignableTxRequest,
};
use reth_rpc_eth_types::{error::FromEvmError, EthApiError};
use reth_tracing::tracing::info;
use reth_transaction_pool::{
    blobstore::DiskFileBlobStore, EthTransactionValidator,
    PoolPooledTx, PoolTransaction, TransactionPool, TransactionValidationTaskExecutor,
};
use std::marker::PhantomData;

/// Type configuration for a regular BSC node.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct BscNode;

impl BscNode {
    /// Returns a [`ComponentsBuilder`] configured for a regular BSC node.
    pub fn components<Node>() -> ComponentsBuilder<
        Node,
        BscPoolBuilder,
        BasicPayloadServiceBuilder<BscPayloadServiceBuilder>,
        BscNetworkBuilder,
        BscExecutorBuilder,
        BscConsensusBuilder,
    >
    where
        Node: FullNodeTypes<
            Types: NodeTypes<
                ChainSpec: Hardforks + EthereumHardforks,
                ChainSpec = BscChainSpec,
                Primitives = EthPrimitives,
                Payload = BscEngineTypes,
            >,
        >,
        <Node::Types as NodeTypes>::Payload: PayloadTypes<
            BuiltPayload = reth_bsc_payload_builder::BscBuiltPayload,
            PayloadAttributes = PayloadAttributes,
            PayloadBuilderAttributes = reth_bsc_payload_builder::BscPayloadBuilderAttributes,
        >,
    {
        ComponentsBuilder::default()
            .node_types::<Node>()
            .pool(BscPoolBuilder::default())
            .executor(BscExecutorBuilder::default())
            .payload(BasicPayloadServiceBuilder::default())
            .network(BscNetworkBuilder::default())
            .consensus(BscConsensusBuilder::default())
    }
}

impl NodeTypes for BscNode {
    type Primitives = EthPrimitives;
    type ChainSpec = BscChainSpec;
    type Storage = EthStorage;
    type Payload = BscEngineTypes;
}

/// Builds [`EthApi`](reth_rpc::EthApi) for BSC.
#[derive(Debug)]
pub struct BscEthApiBuilder<NetworkT = Ethereum>(PhantomData<NetworkT>);

impl<NetworkT> Default for BscEthApiBuilder<NetworkT> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<N, NetworkT> EthApiBuilder<N> for BscEthApiBuilder<NetworkT>
where
    N: FullNodeComponents<
        Types: NodeTypes<ChainSpec: Hardforks + EthereumHardforks>,
        Evm: ConfigureEvm<NextBlockEnvCtx: BuildPendingEnv<HeaderTy<N::Types>>>,
    >,
    NetworkT: RpcTypes<TransactionRequest: SignableTxRequest<TxTy<N::Types>>>,
    EthRpcConverterFor<N, NetworkT>: RpcConvert<
        Primitives = PrimitivesTy<N::Types>,
        Error = EthApiError,
        Network = NetworkT,
        Evm = N::Evm,
    >,
    EthApiError: FromEvmError<N::Evm>,
{
    type EthApi = EthApiFor<N, NetworkT>;

    async fn build_eth_api(self, ctx: EthApiCtx<'_, N>) -> eyre::Result<Self::EthApi> {
        Ok(ctx.eth_api_builder().map_converter(|r| r.with_network()).build())
    }
}

/// Add-ons w.r.t. BSC.
#[derive(Debug)]
pub struct BscAddOns<
    N: FullNodeComponents,
    EthB: EthApiBuilder<N>,
    PVB,
    EB = BasicEngineApiBuilder<PVB>,
    EVB = BasicEngineValidatorBuilder<PVB>,
    RpcMiddleware = Identity,
> {
    inner: RpcAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>,
}

impl<N, EthB, PVB, EB, EVB, RpcMiddleware> BscAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>
where
    N: FullNodeComponents,
    EthB: EthApiBuilder<N>,
{
    pub const fn new(inner: RpcAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>) -> Self {
        Self { inner }
    }
}

impl<N> Default for BscAddOns<N, BscEthApiBuilder, BscEngineValidatorBuilder>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: EthereumHardforks + Clone + 'static,
            Payload: EngineTypes<ExecutionData = ExecutionData>
                         + PayloadTypes<PayloadAttributes = PayloadAttributes>,
            Primitives = EthPrimitives,
        >,
    >,
    BscEthApiBuilder: EthApiBuilder<N>,
{
    fn default() -> Self {
        Self::new(RpcAddOns::new(
            BscEthApiBuilder::default(),
            BscEngineValidatorBuilder::default(),
            BasicEngineApiBuilder::default(),
            BasicEngineValidatorBuilder::default(),
            Default::default(),
        ))
    }
}

impl<N, EthB, PVB, EB, EVB, RpcMiddleware> NodeAddOns<N>
    for BscAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: Hardforks + EthereumHardforks,
            Primitives = EthPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Evm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    EthB: EthApiBuilder<N>,
    PVB: PayloadValidatorBuilder<N>,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    EthApiError: FromEvmError<N::Evm>,
    RpcMiddleware: RethRpcMiddleware,
{
    type Handle = RpcHandle<N, EthB::EthApi>;

    async fn launch_add_ons(
        self,
        ctx: reth_node_api::AddOnsContext<'_, N>,
    ) -> eyre::Result<Self::Handle> {
        self.inner.launch_add_ons(ctx).await
    }
}

impl<N, EthB, PVB, EB, EVB, RpcMiddleware> RethRpcAddOns<N>
    for BscAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: Hardforks + EthereumHardforks,
            Primitives = EthPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Evm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    EthB: EthApiBuilder<N>,
    PVB: PayloadValidatorBuilder<N>,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    EthApiError: FromEvmError<N::Evm>,
    RpcMiddleware: RethRpcMiddleware,
{
    type EthApi = EthB::EthApi;

    fn hooks_mut(&mut self) -> &mut reth_node_builder::rpc::RpcHooks<N, Self::EthApi> {
        self.inner.hooks_mut()
    }
}

impl<N, EthB, PVB, EB, EVB, RpcMiddleware> EngineValidatorAddOn<N>
    for BscAddOns<N, EthB, PVB, EB, EVB, RpcMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,
            Primitives = EthPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Evm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    EthB: EthApiBuilder<N>,
    PVB: Send,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    EthApiError: FromEvmError<N::Evm>,
    RpcMiddleware: Send,
{
    type ValidatorBuilder = EVB;

    fn engine_validator_builder(&self) -> Self::ValidatorBuilder {
        self.inner.engine_validator_builder()
    }
}

impl<N> Node<N> for BscNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        BscPoolBuilder,
        BasicPayloadServiceBuilder<BscPayloadServiceBuilder>,
        BscNetworkBuilder,
        BscExecutorBuilder,
        BscConsensusBuilder,
    >;

    type AddOns = BscAddOns<NodeAdapter<N>, BscEthApiBuilder, BscEngineValidatorBuilder>;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        Self::components()
    }

    fn add_ons(&self) -> Self::AddOns {
        BscAddOns::default()
    }
}

/// A regular BSC evm builder.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct BscExecutorBuilder;

impl<Types, Node> ExecutorBuilder<Node> for BscExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthereumHardforks,
        ChainSpec = BscChainSpec,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = BscEvmConfig;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(BscEvmConfig::new(ctx.chain_spec()))
    }
}

/// A basic BSC transaction pool.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct BscPoolBuilder;

impl<Types, Node> PoolBuilder<Node> for BscPoolBuilder
where
    Types: NodeTypes<
        ChainSpec: EthereumHardforks + Hardforks + EthExecutorSpec,
        ChainSpec = BscChainSpec,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type Pool = reth_transaction_pool::Pool<
        TransactionValidationTaskExecutor<
            EthTransactionValidator<
                Node::Provider,
                reth_transaction_pool::EthPooledTransaction,
                BscEvmConfig,
            >,
        >,
        reth_transaction_pool::CoinbaseTipOrdering<reth_transaction_pool::EthPooledTransaction>,
        DiskFileBlobStore,
    >;

    async fn build_pool(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Pool> {
        let pool_config = ctx.pool_config();
        let blob_store =
            reth_node_builder::components::create_blob_store_with_cache(ctx, None)?;

        let evm_config = BscEvmConfig::new(ctx.chain_spec());
        let validator = TransactionValidationTaskExecutor::eth_builder(
            ctx.provider().clone(),
            evm_config,
        )
        .kzg_settings(ctx.kzg_settings()?)
        .with_additional_tasks(1)
        .build_with_tasks(ctx.task_executor().clone(), blob_store.clone());

        let transaction_pool = TxPoolBuilder::new(ctx)
            .with_validator(validator)
            .build_and_spawn_maintenance_task(blob_store, pool_config)?;

        info!(target: "reth::cli", "Transaction pool initialized");
        Ok(transaction_pool)
    }
}

/// BSC payload builder service.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub struct BscPayloadServiceBuilder;

impl<Types, Node, Pool, Evm> PayloadBuilderBuilder<Node, Pool, Evm> for BscPayloadServiceBuilder
where
    Types: NodeTypes<
        ChainSpec: EthereumHardforks,
        ChainSpec = BscChainSpec,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
    Evm: ConfigureEvm<
            Primitives = PrimitivesTy<Types>,
            NextBlockEnvCtx = NextBlockEnvAttributes,
        > + 'static,
    Types::Payload: PayloadTypes<
        BuiltPayload = reth_bsc_payload_builder::BscBuiltPayload,
        PayloadAttributes = PayloadAttributes,
        PayloadBuilderAttributes = reth_bsc_payload_builder::BscPayloadBuilderAttributes,
    >,
{
    type PayloadBuilder = BscPayloadBuilder<Pool, Node::Provider, Evm>;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: Evm,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let conf = ctx.payload_builder_config();
        Ok(BscPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            BscBuilderConfig::new()
                .with_gas_limit(conf.gas_limit_for(ctx.chain_spec().chain()))
                .with_extra_data(conf.extra_data()),
        ))
    }
}

/// A basic BSC network builder.
#[derive(Debug, Default, Clone, Copy)]
pub struct BscNetworkBuilder;

impl<Node, Pool> NetworkBuilder<Node, Pool> for BscNetworkBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec: Hardforks>>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
{
    type Network =
        NetworkHandle<BasicNetworkPrimitives<PrimitivesTy<Node::Types>, PoolPooledTx<Pool>>>;

    async fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::Network> {
        let network = ctx.network_builder().await?;
        let handle = ctx.start_network(network, pool);
        info!(target: "reth::cli", enode=%handle.local_node_record(), "P2P networking initialized");
        Ok(handle)
    }
}

/// A basic BSC consensus builder.
#[derive(Debug, Default, Clone, Copy)]
pub struct BscConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for BscConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,
            ChainSpec = BscChainSpec,
            Primitives = EthPrimitives,
        >,
    >,
{
    type Consensus = Arc<Parlia>;

    async fn build_consensus(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(Parlia::new(ctx.chain_spec(), ParliaConfig::default())))
    }
}

/// Builder for [`BscEngineValidator`].
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct BscEngineValidatorBuilder;

impl<Node, Types> PayloadValidatorBuilder<Node> for BscEngineValidatorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthereumHardforks + Clone + 'static,
        ChainSpec = BscChainSpec,
        Payload: EngineTypes<ExecutionData = ExecutionData>
                     + PayloadTypes<PayloadAttributes = PayloadAttributes>,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeComponents<Types = Types>,
{
    type Validator = BscEngineValidator;

    async fn build(self, ctx: &AddOnsContext<'_, Node>) -> eyre::Result<Self::Validator> {
        Ok(BscEngineValidator::new(ctx.config.chain.clone()))
    }
}
