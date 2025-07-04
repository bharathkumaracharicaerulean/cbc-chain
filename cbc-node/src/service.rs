//! Service and ServiceFactory implementation for CBC Node with DCF Consensus only.
//!
//! This file defines how the CBC node's services are built and launched,
//! including DCF consensus setup, transaction pool, networking,


use futures::FutureExt;
use sc_client_api::Backend;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::TelemetryWorker;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use cbc_runtime::{self, apis::RuntimeApi, opaque::Block};
use std::{sync::Arc};
// DCF Consensus imports
use cbc_consensus::{DcfConsensus, ConsensusParams, AuthorSelectionMode};
use sc_service::ImportQueue;
use sc_consensus::import_queue::{ImportQueueService, Link};
use std::pin::Pin;
use std::future::Future;
use sp_core::sr25519::Pair;
use sc_consensus::IncomingBlock;
use sc_network::PeerId;
use sp_runtime::Justifications;
use sp_consensus::BlockOrigin;

// Minimal dummy import queue for DCF-only node
pub struct DummyImportQueue;
impl<B: sp_runtime::traits::Block> sc_service::ImportQueue<B> for DummyImportQueue {
    fn poll_actions(&mut self, _cx: &mut std::task::Context<'_>, _link: &dyn Link<B>) {}
    fn service(&self) -> Box<(dyn ImportQueueService<B> + 'static)> {
        struct DummyService;
        impl<B: sp_runtime::traits::Block> ImportQueueService<B> for DummyService {
            fn import_blocks(&mut self, _origin: BlockOrigin, _blocks: Vec<IncomingBlock<B>>) {}
            fn import_justifications(&mut self, _peer_id: PeerId, _hash: <B as sp_runtime::traits::Block>::Hash, _number: <<B as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number, _justifications: Justifications) {}
        }
        Box::new(DummyService)
    }
    fn service_ref(&mut self) -> &mut dyn ImportQueueService<B> {
        struct DummyService;
        impl<B: sp_runtime::traits::Block> ImportQueueService<B> for DummyService {
            fn import_blocks(&mut self, _origin: BlockOrigin, _blocks: Vec<IncomingBlock<B>>) {}
            fn import_justifications(&mut self, _peer_id: PeerId, _hash: <B as sp_runtime::traits::Block>::Hash, _number: <<B as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number, _justifications: Justifications) {}
        }
        static mut DUMMY: DummyService = DummyService;
        unsafe { &mut DUMMY }
    }
    fn run<'life0, 'async_trait>(
        self,
        _link: &'life0 (dyn Link<B> + 'life0),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {})
    }
}

/// Full CBC client type (using the runtime's `Block` and `RuntimeApi`)
pub(crate) type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;

/// Full backend type used for storage operations.
type FullBackend = sc_service::TFullBackend<Block>;

/// Longest-chain selection strategy for forks.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Alias for the output of `new_partial()` — the essential building blocks of a node.
pub type Service = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    DummyImportQueue, // Use dummy import queue
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
    (), 
>;

/// Extra parameters for configuring services
#[derive(Clone)]
pub struct NodeConfig {
    /// RPC configuration from CLI
    pub rpc_config: crate::rpc::RpcSecurityConfig,
}

/// Builds the partial components of a node (used for both full and light nodes).
/// Returns the essential pieces to create the full node later.
pub fn new_partial(
    config: &Configuration,
    node_config: NodeConfig,
) -> Result<Service, ServiceError> {
    // Setup optional telemetry (for Prometheus/Grafana dashboards).
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    // Create WASM executor for executing runtime logic.
    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);

    // Build the core node components (client, backend, keystore, task manager).
    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;

    let client = Arc::new(client);

    // If telemetry was enabled, spawn its background worker.
    let _telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let import_queue = DummyImportQueue;

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue, // Use dummy import queue
        keystore_container,
        select_chain,
        transaction_pool,
        other: (), 
    })
}

/// Builds and starts a full CBC service node with DCF consensus only.
pub fn new_full<
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
    config: Configuration,
    node_config: NodeConfig,
) -> Result<TaskManager, ServiceError>
where
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
{
    // Start with building partial components (client, pool, backend, etc.)
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        keystore_container,
        select_chain,
        transaction_pool,
        import_queue,
        ..
    } = new_partial(&config, node_config.clone())?;

    // === Network Setup ===
    let mut net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());

    let metrics = N::register_notification_metrics(config.prometheus_registry());
    let peer_store_handle = net_config.peer_store_handle();

    // === Build the network ===
    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue, // Use dummy import queue
            block_announce_validator_builder: None,
            warp_sync_config: None,
            block_relay: None,
            metrics,
        })?;

    // === Offchain Workers ===
    if config.offchain_worker.enabled {
        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                enable_http_requests: true,
                custom_extensions: |_| vec![],
            })?;

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
        );
    }

    // === DCF Consensus Integration ===
    // Spawn the DCF consensus loop as a background task.
    let consensus_params = ConsensusParams {
        author_selection_mode: AuthorSelectionMode::RoundRobin,
        finality_threshold: 10,
        block_time: 6, // seconds
        max_block_size: 2 * 1024 * 1024,
        max_transactions_per_block: 1000,
    };
    let client_for_consensus = client.clone();
    task_manager.spawn_essential_handle().spawn_blocking(
        "dcf-consensus",
        None,
        async move {
            let mut dcf = DcfConsensus::<_, _, Pair>::new(client_for_consensus, consensus_params);
            dcf.run().await;
        },
    );

    // === RPC Setup ===
    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let rpc_config = node_config.rpc_config.clone();

        Box::new(move |_| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                rpc_config: rpc_config.clone(),
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    // === Spawn all async services (RPC, networking, etc.) ===
    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::new(network.clone()),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: None, // No telemetry for now
    })?;

    // All services have started successfully.
    Ok(task_manager)
}

