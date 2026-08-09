use futures::{Stream, StreamExt};
use pin_project::pin_project;
use reth_chain_state::StateTrieOverlayManager;
use reth_chainspec::EthChainSpec;
use reth_consensus::FullConsensus;
use reth_engine_primitives::{BeaconEngineMessage, ConsensusEngineEvent};
use reth_engine_tree::{
    backfill::PipelineSync,
    download::BasicBlockDownloader,
    engine::{EngineApiKind, EngineApiRequest, EngineApiRequestHandler, EngineHandler},
    persistence::PersistenceHandle,
    tree::{EngineApiTreeHandler, EngineValidator, TreeConfig},
};
pub use reth_engine_tree::{
    chain::{ChainEvent, ChainOrchestrator},
    engine::EngineApiEvent,
};
use reth_evm::ConfigureEvm;
use reth_network_p2p::BlockClient;
use reth_node_types::{BlockTy, NodeTypes};
use reth_payload_builder::PayloadBuilderHandle;
use reth_provider::{
    providers::{BlockchainProvider, ProviderNodeTypes},
    ProviderFactory,
};
use reth_prune::PrunerWithFactory;
use reth_stages_api::{MetricEventsSender, Pipeline};
use reth_tasks::Runtime;
use reth_trie_db::ChangesetCache;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Alias for consensus engine stream.
pub type EngineMessageStream<T> = Pin<Box<dyn Stream<Item = BeaconEngineMessage<T>> + Send + Sync>>;

/// Alias for chain orchestrator.
type EngineServiceType<N, Client> = ChainOrchestrator<
    EngineHandler<
        EngineApiRequestHandler<
            EngineApiRequest<<N as NodeTypes>::Payload, <N as NodeTypes>::Primitives>,
            <N as NodeTypes>::Primitives,
        >,
        EngineMessageStream<<N as NodeTypes>::Payload>,
        BasicBlockDownloader<Client, BlockTy<N>>,
    >,
    PipelineSync<N>,
>;

/// The type that drives the chain forward and communicates progress.
#[pin_project]
#[expect(missing_debug_implementations)]
// TODO(mattsse): remove hidden once fixed : <https://github.com/rust-lang/rust/issues/135363>
//  otherwise rustdoc fails to resolve the alias
#[doc(hidden)]
pub struct EngineService<N, Client>
where
    N: ProviderNodeTypes,
    Client: BlockClient<Block = BlockTy<N>> + 'static,
{
    orchestrator: EngineServiceType<N, Client>,
}

impl<N, Client> EngineService<N, Client>
where
    N: ProviderNodeTypes,
    Client: BlockClient<Block = BlockTy<N>> + 'static,
{
    /// Constructor for `EngineService`.
    #[expect(clippy::too_many_arguments)]
    pub fn new<V, Evm>(
        consensus: Arc<dyn FullConsensus<N::Primitives>>,
        chain_spec: Arc<N::ChainSpec>,
        client: Client,
        incoming_requests: EngineMessageStream<N::Payload>,
        pipeline: Pipeline<N>,
        pipeline_task_spawner: Runtime,
        provider: ProviderFactory<N>,
        blockchain_db: BlockchainProvider<N>,
        pruner: PrunerWithFactory<ProviderFactory<N>>,
        payload_builder: PayloadBuilderHandle<N::Payload>,
        payload_validator: V,
        state_trie_overlays: StateTrieOverlayManager<N::Primitives>,
        tree_config: TreeConfig,
        sync_metrics_tx: MetricEventsSender,
        evm_config: Evm,
        changeset_cache: ChangesetCache,
        runtime: Runtime,
    ) -> Self
    where
        V: EngineValidator<N::Payload>,
        Evm: ConfigureEvm<Primitives = N::Primitives> + Send + Sync + 'static,
    {
        let engine_kind =
            if chain_spec.is_optimism() { EngineApiKind::OpStack } else { EngineApiKind::Ethereum };

        let downloader = BasicBlockDownloader::new(client, consensus.clone());

        let persistence_handle =
            PersistenceHandle::<N::Primitives>::spawn_service(provider, pruner, sync_metrics_tx);

        let canonical_in_memory_state = blockchain_db.canonical_in_memory_state();

        let (to_tree_tx, from_tree) = EngineApiTreeHandler::spawn_new(
            blockchain_db,
            consensus,
            payload_validator,
            persistence_handle,
            payload_builder,
            canonical_in_memory_state,
            state_trie_overlays,
            tree_config,
            engine_kind,
            evm_config,
            changeset_cache,
            runtime,
        );

        let engine_handler = EngineApiRequestHandler::new(to_tree_tx, from_tree);
        let handler = EngineHandler::new(engine_handler, downloader, incoming_requests);

        let backfill_sync = PipelineSync::new(pipeline, pipeline_task_spawner);

        Self { orchestrator: ChainOrchestrator::new(handler, backfill_sync) }
    }

    /// Returns a mutable reference to the orchestrator.
    pub fn orchestrator_mut(&mut self) -> &mut EngineServiceType<N, Client> {
        &mut self.orchestrator
    }
}

impl<N, Client> Stream for EngineService<N, Client>
where
    N: ProviderNodeTypes,
    Client: BlockClient<Block = BlockTy<N>> + 'static,
{
    type Item = ChainEvent<ConsensusEngineEvent<N::Primitives>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut orchestrator = self.project().orchestrator;
        StreamExt::poll_next_unpin(&mut orchestrator, cx)
    }
}
