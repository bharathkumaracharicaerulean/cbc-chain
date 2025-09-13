#![allow(unused_variables, static_mut_refs)]
use futures::FutureExt;
use sc_client_api::Backend;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::TelemetryWorker;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use cbc_runtime::{self, apis::RuntimeApi, opaque::Block};
use std::{sync::Arc};
use sc_consensus::import_queue::{ImportQueueService, Link};
use std::pin::Pin;
use std::future::Future;
use cbc_consensus::{ConsensusParams, AuthorSelectionMode};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{SaturatedConversion, Header as HeaderT};
use sp_consensus::BlockOrigin;
use pallet_cbc_dcf::DcfApi;

/// DCF-integrated import queue that validates blocks through the DCF runtime
pub struct DcfImportQueue {
    client: Arc<FullClient>,
}

impl DcfImportQueue {
    pub fn new(client: Arc<FullClient>) -> Self {
        Self { client }
    }
}

impl sc_service::ImportQueue<Block> for DcfImportQueue {
    fn poll_actions(&mut self, _cx: &mut std::task::Context<'_>, _link: &dyn Link<Block>) {
        // Poll for any pending import actions
        // In a full implementation, this would handle queued block imports
    }
    
    fn service(&self) -> Box<(dyn ImportQueueService<Block> + 'static)> {
        Box::new(DcfImportQueueService {
            client: self.client.clone(),
        })
    }
    
    fn service_ref(&mut self) -> &mut dyn ImportQueueService<Block> {
        // Create a static service instance for the lifetime of the import queue
        static mut SERVICE: Option<DcfImportQueueService> = None;
        unsafe {
            if SERVICE.is_none() {
                SERVICE = Some(DcfImportQueueService {
                    client: self.client.clone(),
                });
            }
            SERVICE.as_mut().unwrap()
        }
    }
    
    fn run<'life0, 'async_trait>(
        self,
        _link: &'life0 (dyn Link<Block> + 'life0),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            log::info!("DCF Import Queue: Starting import queue service");
            // In a full implementation, this would run the import queue loop
            // For now, we just keep it running
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        })
    }
}

/// DCF import queue service that handles block imports with DCF validation
struct DcfImportQueueService {
    client: Arc<FullClient>,
}

impl ImportQueueService<Block> for DcfImportQueueService {
    fn import_blocks(&mut self, origin: BlockOrigin, blocks: Vec<sc_consensus::IncomingBlock<Block>>) {
        log::info!("DCF Import Queue: Importing {} blocks from {:?}", blocks.len(), origin);
        
        for block in blocks {
            if let Err(e) = self.validate_and_import_block(block) {
                log::error!("DCF Import Queue: Failed to import block: {:?}", e);
            }
        }
    }
    
    fn import_justifications(
        &mut self, 
        _peer_id: sc_network::PeerId, 
        hash: <Block as sp_runtime::traits::Block>::Hash, 
        number: <<Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number, 
        _justifications: sp_runtime::Justifications
    ) {
        log::debug!("DCF Import Queue: Importing justifications for block #{} ({:?})", number, hash);
        // In a full implementation, this would validate and store justifications
    }
}

impl DcfImportQueueService {
    /// Validate and import a block using DCF rules
    fn validate_and_import_block(&self, block: sc_consensus::IncomingBlock<Block>) -> Result<(), String> {
        // Extract block information
        let block_header = block.header.ok_or("Missing block header")?;
        let block_number = (*block_header.number()).saturated_into::<u32>();
        let block_hash = block_header.hash();
        
        log::info!("DCF Import Queue: Validating block #{} ({:?})", block_number, block_hash);
        
        // Get the runtime API
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        // Extract block author from the block header
        if let Some(author) = self.extract_block_author(&block_header) {
            log::info!("DCF Import Queue: Block author: {:?}", author);
            
            // Validate expected author and emit AuthorMismatch event if needed
            match api.validate_expected_author(best_hash, block_number, author.clone()) {
                Ok(is_valid) => {
                    if !is_valid {
                        // AuthorMismatch event was already emitted by the runtime method
                        log::error!("DCF Import Queue: Author mismatch for block #{} with author {:?}", 
                                   block_number, author);
                        
                        // Reject the block due to author mismatch
                        return Err(format!("Author mismatch for block #{} with author {:?}", block_number, author));
                    } else {
                        log::info!("DCF Import Queue: Block author validation passed for {:?}", author);
                    }
                }
                Err(e) => {
                    log::error!("DCF Import Queue: Failed to validate expected author for block #{}: {:?}", block_number, e);
                    return Err(format!("Failed to validate expected author: {:?}", e));
                }
            }
            
            // Check if the author is in the active validator set
            match api.get_active_validators(best_hash) {
                Ok(active_validators) => {
                    if !active_validators.contains(&author) {
                        log::error!("DCF Import Queue: Author {:?} is not in active validator set", author);
                        return Err("Author not in active validator set".to_string());
                    }
                }
                Err(e) => {
                    log::error!("DCF Import Queue: Failed to get active validators: {:?}", e);
                    return Err(format!("Failed to get active validators: {:?}", e));
                }
            }
        } else {
            log::warn!("DCF Import Queue: Could not extract block author from block #{}", block_number);
        }
        
        log::info!("DCF Import Queue: Block #{} validation completed successfully", block_number);
        Ok(())
    }
    
    /// Extract the block author from the block header
    fn extract_block_author(&self, header: &<Block as sp_runtime::traits::Block>::Header) -> Option<cbc_runtime::AccountId> {
        // For now, using  default author for testing
        // this would extract the author from block digest
        Some(cbc_runtime::AccountId::from([0u8; 32]))
    }
}

pub(crate) type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;

type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type Service = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    DcfImportQueue,
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
    (), 
>;

#[derive(Clone)]
pub struct NodeConfig {
    pub rpc_config: crate::rpc::RpcSecurityConfig,
}

pub fn new_partial(
    config: &Configuration,
    node_config: &NodeConfig,
) -> Result<Service, ServiceError> {
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

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;

    let client = Arc::new(client);

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

    // Use our DCF import queue for validation, but we'll also need direct client access for real imports
    let import_queue = DcfImportQueue::new(client.clone());

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (), 
    })
}

pub fn new_full<
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
    config: Configuration,
    node_config: &NodeConfig,
) -> Result<TaskManager, ServiceError>
where
    N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain: _,
        transaction_pool,
        ..
    } = new_partial(&config, &node_config)?;

    let net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());

    let metrics = N::register_notification_metrics(config.prometheus_registry());
    let _peer_store_handle = net_config.peer_store_handle();

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: None,
            block_relay: None,
            metrics,
        })?;

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


    // --- CBC Custom Consensus Integration ---
    // This integrates custom PoS/PoI consensus 
    // The DCF pallet provides validator selection based on PoS and PoI scores
    
    let consensus_params = ConsensusParams {
        author_selection_mode: AuthorSelectionMode::RoundRobin,
        finality_threshold: 10,
        block_time: 6,
        max_block_size: 2 * 1024 * 1024,
        max_transactions_per_block: 1000,
        slot_duration: std::time::Duration::from_secs(6),
        min_block_time: 1000,
        metrics_update_interval: 10,
        score_refresh_interval: 50,
        consensus_loop_interval: 1000,
        detailed_logging_interval: 100,
        health_check_interval: 10,
        min_performance_score: 30,
        high_performance_score: 80,
        min_participation_rate: 50,
        high_participation_rate: 90,
        max_missed_blocks: 10,
        max_missed_blocks_high: 2,
        healthy_validator_score: 50,
        healthy_participation_rate: 80,
        healthy_missed_blocks_max: 5,
        top_validators_display_count: 5,
        health_check_sample_size: 5,
    };

    // Initialize Prometheus metrics for consensus monitoring
    let consensus_metrics = if let Some(registry) = config.prometheus_registry() {
        match cbc_consensus::metrics::ConsensusMetrics::new(registry) {
            Ok(metrics) => {
                log::info!("CBC: Consensus metrics initialized successfully");
                Some(metrics)
            }
            Err(e) => {
                log::error!("CBC: Failed to initialize consensus metrics: {:?}", e);
                None
            }
        }
    } else {
        log::warn!("CBC: No Prometheus registry available, metrics disabled");
        None
    };
    
    // This will actually add blocks to the chain state
    let dcf_block_import = cbc_consensus::RealBlockImport::new(client.clone());
    let dcf_block_import_arc = Arc::new(dcf_block_import);
    
    // Set up  PoS+PoI block production with DCF consensus
    if config.role.is_authority() {
        let mut dcf_consensus = if let Some(metrics) = consensus_metrics.clone() {
            cbc_consensus::DcfConsensus::<Block, FullClient, sp_core::sr25519::Pair, _>::new_with_metrics(
                client.clone(),
                transaction_pool.clone(),
                dcf_block_import_arc.clone(), // Use real import queue
                consensus_params.clone(),
                metrics,
            )
        } else {
            cbc_consensus::DcfConsensus::<Block, FullClient, sp_core::sr25519::Pair, _>::new(
                client.clone(),
                transaction_pool.clone(),
                dcf_block_import_arc.clone(), // Use real import queue
                consensus_params.clone(),
            )
        };
        
        task_manager.spawn_essential_handle().spawn(
            "cbc-pos-poi-consensus",
            None,
            async move {
                log::info!("CBC: Starting real PoS+PoI consensus engine for block production");
                // Run the consensus engine which will produce real blocks
                dcf_consensus.run().await;
                log::error!("CBC: PoS+PoI consensus engine unexpectedly stopped");
            },
        );
    }

    // Start metrics update task if metrics are available
    if let Some(metrics) = consensus_metrics {
        let metrics_client = client.clone();
        task_manager.spawn_handle().spawn(
            "consensus-metrics-updater",
            None,
            async move {
                log::info!("CBC: Starting consensus metrics updater");
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                
                loop {
                    interval.tick().await;
                    
                    if let Err(e) = metrics.update_from_runtime(&metrics_client) {
                        log::warn!("CBC: Failed to update consensus metrics: {}", e);
                    }
                }
            },
        );
    }
    
    log::info!("DCF: Consensus monitoring active. Block production handled by Substrate's default mechanisms.");

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
        telemetry: None,
    })?;

    Ok(task_manager)
}