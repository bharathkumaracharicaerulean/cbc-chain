//! Service and ServiceFactory implementation.
//!
//! This file defines how the CBC node's services are built and launched,
//! including consensus setup (DCF), transaction pool, networking,
//! and RPC interfaces. It's a critical part of the node runtime.

use futures::FutureExt; // Needed for handling async functions that return futures.
use sc_client_api::Backend;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager}; // Core service types.
use cbc_runtime::{opaque::Block};
use cbc_runtime::apis::RuntimeApi;
use std::{sync::Arc}; // Standard concurrency and time utilities.
use sp_core::traits::SpawnNamed;

// === Type Aliases for Readability ===

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
	sc_consensus::BasicQueue<Block>,
	sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
	Option<sc_telemetry::Telemetry>,
>;

/// Builds the partial components of a node (used for both full and light nodes).
/// Returns the essential pieces to create the full node later.
pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = sc_telemetry::TelemetryWorker::new(16)?;
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
	let telemetry = telemetry.map(|(worker, telemetry)| {
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
	let block_import = cbc_consensus::DcfBlockImport::new(client.clone());
	let import_queue = sc_consensus::BasicQueue::new(
		sc_consensus::import_queue::BasicVerifier::new(client.clone()),
		Box::new(block_import),
		None,
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
	);
	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: telemetry,
	})
}

/// Builds and starts a full CBC service node.
/// This includes the network layer, DCF consensus engine,
/// offchain workers, and RPC server.
///
/// `N` is the type of network backend (e.g., Libp2p or Litep2p).
pub async fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
	let Service {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain: _,
		transaction_pool,
		other: telemetry,
	} = new_partial(&config)?;

	// === Network Setup ===
	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_config: None,
			block_relay: None,
			net_config: sc_network::config::FullNetworkConfiguration::new(&config.network, config.prometheus_registry().cloned()),
			metrics: sc_network::config::NotificationMetrics::new(None),
		})?;

	// === Offchain Workers ===
	if config.offchain_worker.enabled {
		let offchain_workers =
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(sc_transaction_pool_api::OffchainTransactionPoolFactory::new(
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

	// === RPC Setup ===
	let rpc_builder = {
		let client = client.clone();
		Box::new(move |_spawner: Arc<dyn SpawnNamed>| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				deny_unsafe: sc_rpc_api::DenyUnsafe::No,
			};
			Ok(crate::rpc::create_full(deps))
		})
	};

	// === Spawn all async services ===
	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder,
		backend,
		system_rpc_tx,
		tx_handler_controller,
		sync_service,
		config,
		telemetry: telemetry.as_mut(),
	})?;

	Ok(task_manager)
}
