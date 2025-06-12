//! Service and ServiceFactory implementation.
//!
//! This file defines how the CBC node's services are built and launched,
//! including consensus setup (DCF), transaction pool, networking,
//! and RPC interfaces. It's a critical part of the node runtime.

use futures::FutureExt; // Needed for handling async functions that return futures.
use sc_client_api::Backend;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager}; // Core service types.
use sc_telemetry::{Telemetry, TelemetryWorker}; // Telemetry for monitoring nodes.
use sc_transaction_pool_api::OffchainTransactionPoolFactory; // For submitting transactions via offchain workers.
use cbc_runtime::{opaque::Block};
use cbc_runtime::apis::RuntimeApi;
use std::{sync::Arc, time::Duration}; // Standard concurrency and time utilities.
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

// Import DCF consensus components
use cbc_consensus::{
	DcfConsensus,
	DcfBlockImport,
	ValidatorSet,
	AuthorSelection,
	EpochManager,
	ProposerFactory,
	ImportQueue,
	FinalityEngine,
	AuthorSelectionMode,
	ConsensusParams,
	EpochConfig,
};

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
	DcfBlockImport<Block, FullClient, sp_core::ed25519::Pair>,
	sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
	Option<Telemetry>,
>;

/// Builds the partial components of a node (used for both full and light nodes).
/// Returns the essential pieces to create the full node later.
pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {
	// Setup telemetry
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

	// Build the core node components (client, backend, keystore, task_manager)
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client);

	// Spawn telemetry worker if enabled
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// Longest chain fork choice rule
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	// Create the transaction pool
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

	// Create DCF import queue
	let dcf_config = DcfConfig {
		author_selection: AuthorSelection::new(AuthorSelectionMode::RoundRobin),
		params: ConsensusParams {
			author_selection_mode: AuthorSelectionMode::RoundRobin,
			finality_threshold: 2,
			block_time: std::time::Duration::from_secs(6),
			max_block_size: 1024 * 1024,
			max_transactions_per_block: 1000,
		},
	};
	let dcf_consensus = DcfConsensus::new(
		client.clone(),
		dcf_config.author_selection,
		dcf_config.params,
	);
	let import_queue = DcfBlockImport::new(Arc::new(dcf_consensus));

	// Return all the components as a tuple for building the full node.
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
pub async fn new_full(config: Configuration) -> Result<NewFull<Block, FullClient, FullBackend>, ServiceError> {
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			None,
		)?;
	let client = Arc::new(client);

	let dcf_config = DcfConfig {
		author_selection: AuthorSelection::new(AuthorSelectionMode::RoundRobin),
		params: ConsensusParams {
			author_selection_mode: AuthorSelectionMode::RoundRobin,
			finality_threshold: 2,
			block_time: std::time::Duration::from_secs(6),
			max_block_size: 1024 * 1024,
			max_transactions_per_block: 1000,
		},
	};

	let dcf_consensus = DcfConsensus::new(
		client.clone(),
		dcf_config.author_selection,
		dcf_config.params,
	);

	let import_queue = DcfBlockImport::new(Arc::new(dcf_consensus));

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
			import_queue,
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

	// === DCF Consensus Setup ===
	let dcf_config = DcfConfig {
		author_selection: AuthorSelection::new(AuthorSelectionMode::RoundRobin),
		params: ConsensusParams {
			author_selection_mode: AuthorSelectionMode::RoundRobin,
			finality_threshold: 2,
			block_time: std::time::Duration::from_secs(6),
			max_block_size: 1024 * 1024,
			max_transactions_per_block: 1000,
		},
	};
	let dcf_consensus = DcfConsensus::new(
		client.clone(),
		dcf_config.author_selection,
		dcf_config.params,
	);
	
	// Setup validator set
	let validator_set_config = ValidatorSetConfig {
		max_validators: 100,
		min_stake: 1000,
	};
	let validator_set = ValidatorSet::new(
		validator_set_config.max_validators,
		validator_set_config.min_stake,
	);

	// Setup epoch manager
	let epoch_config = EpochConfig {
		epoch_length: 100,
		min_validators: 4,
		max_validators: 100,
		min_stake: 1000,
	};
	let epoch_manager = EpochManager::new(epoch_config);

	// Setup proposer factory
	let proposer_factory = ProposerFactory::new(
		dcf_config.author_selection,
		dcf_config.params.block_time,
	);

	// Setup finality engine
	let finality_engine = FinalityEngine::new(dcf_config.params.finality_threshold);

	// Setup import queue
	let import_queue = ImportQueue::new(validator_set);
	
	// Start consensus tasks
	task_manager.spawn_essential_handle().spawn(
		"consensus",
		None,
		Box::pin(async move {
			epoch_manager.run().await;
			finality_engine.run().await;
		}),
	);

	// === RPC Setup ===
	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				deny_unsafe,
			};
			crate::rpc::create_full(deps)
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
		telemetry: telemetry.as_ref().map(|x| x.handle()),
	})?;

	Ok(task_manager)
}
