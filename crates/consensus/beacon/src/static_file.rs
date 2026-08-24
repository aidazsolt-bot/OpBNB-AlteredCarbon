//! Static file hook stub.

use reth_provider::{
    BlockReader, ChainStateBlockReader, DatabaseProviderFactory, StageCheckpointReader,
    StaticFileProviderFactory,
};
use reth_static_file::StaticFileProducer;
use reth_tasks::TaskSpawner;

/// Manages producing static files under the control of the engine.
#[derive(Debug)]
pub struct StaticFileHook<Provider> {
    producer: StaticFileProducer<Provider>,
    task_spawner: Box<dyn TaskSpawner>,
}

impl<Provider> StaticFileHook<Provider>
where
    Provider: StaticFileProviderFactory
        + DatabaseProviderFactory<
            Provider: StageCheckpointReader + BlockReader + ChainStateBlockReader,
        > + 'static,
{
    /// Create a new instance.
    pub fn new(producer: StaticFileProducer<Provider>, task_spawner: Box<dyn TaskSpawner>) -> Self {
        Self { producer, task_spawner }
    }
}
