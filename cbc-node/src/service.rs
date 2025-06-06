//! Service and ServiceFactory implementation.
//!
//! This file defines how the CBC node's services are built and launched,
//! including consensus setup (AURA/GRANDPA), transaction pool, networking,
//! and RPC interfaces. It's a critical part of the node runtime.

use futures::FutureExt; // Needed for handling async functions that return futures.
use sc_client_api::{Backend, BlockBackend}; // Traits for interacting with blockchain backends.
// use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams}; // AURA consensus building blocks.
// use sc_consensus_grandpa::SharedVoterState; // Used for GRANDPA consensus participation.
use sc_service::{error::Error as ServiceError, Configuration, TaskManager, WarpSyncConfig}; // Core service types.
use sc_telemetry::{Telemetry, TelemetryWorker}; // Telemetry for monitoring nodes.
use sc_transaction_pool_api::OffchainTransactionPoolFactory; // For submitting transactions via offchain workers.
use cbc_runtime::{self, apis::RuntimeApi, opaque::Block}; // Use CBC runtime types and APIs.
// use sp_consensus_aura::sr25519::AuthorityPair as AuraPair; // AURA authority type.
use std::{sync::Arc, time::Duration}; // Standard concurrency and time utilities.
use sc_consensus::{
	BasicQueue, BlockCheckParams, BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult,
	LongestChain,
};
use sc_consensus_pos::{PosBlockImport, PosConsensusDataProvider};
use sc_consensus_poi::{PoiBlockImport, PoiConsensusDataProvider};
use sc_consensus_dcf::{DcfBlockImport, DcfConsensusDataProvider};
use sc_executor::NativeElseWasmExecutor;
use sc_network::NetworkService;
use sc_network_sync::SyncingService;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::{BlockOrigin, Environment, Proposer, RecordProof};
use sp_consensus_pos::PosApi;
use sp_consensus_poi::PoiApi;
use sp_consensus_dcf::DcfApi;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, Zero};

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

/// The interval (in blocks) to generate GRANDPA justifications.
// const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Alias for the output of `new_partial()` — the essential building blocks of a node.
pub type Service = sc_service::PartialComponents<
	FullClient,
	FullBackend,
	FullSelectChain,
	sc_consensus::DefaultImportQueue<Block>,
	sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
	(
		// sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
		// sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
		Option<Telemetry>,
	),
>;

/// Builds a new partial node.
pub fn new_partial(
	config: &Configuration,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			PosBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
			PoiBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
			DcfBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
			GrandpaLink<Block>,
			Option<Telemetry>,
		),
	>,
	ServiceError,
> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?
		.map(|(worker, telemetry)| {
			task_manager.spawn_handle().spawn(
				"telemetry",
				None,
				worker.run(),
			);
			telemetry
		});

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
		)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let pos_block_import = PosBlockImport::new(
		grandpa_block_import.clone(),
		client.clone(),
		select_chain.clone(),
	);

	let poi_block_import = PoiBlockImport::new(
		grandpa_block_import.clone(),
		client.clone(),
		select_chain.clone(),
	);

	let dcf_block_import = DcfBlockImport::new(
		grandpa_block_import,
		client.clone(),
		select_chain.clone(),
	);

	let import_queue = sc_consensus::BasicQueue::new(
		pos_block_import.clone(),
		Box::new(pos_block_import.clone()),
		Box::new(poi_block_import.clone()),
		Box::new(dcf_block_import.clone()),
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
		other: (
			pos_block_import,
			poi_block_import,
			dcf_block_import,
			grandpa_link,
			telemetry,
		),
	})
}

/// Builds and starts a full CBC service node.
/// This includes the network layer, consensus engine (AURA and GRANDPA),
/// offchain workers, and RPC server.
///
/// `N` is the type of network backend (e.g., Libp2p or Litep2p).
pub fn new_full<
	N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>,
>(
	config: Configuration,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (pos_block_import, poi_block_import, dcf_block_import, grandpa_link, mut telemetry),
	} = new_partial(&config)?;

	let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
		backend.clone(),
		client.clone(),
	);

	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
			warp_sync: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		)?;
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_extensions_builder = {
		let client = client.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
			};
			rpc::create_full(deps)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend,
		system_rpc_tx,
		config,
		telemetry: telemetry.as_mut(),
	})?;

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let pos_client = client.clone();
		let pos_backend = backend.clone();
		let pos_consensus_data_provider = PosConsensusDataProvider::new(
			pos_client.clone(),
			pos_backend.clone(),
		);

		let poi_client = client.clone();
		let poi_backend = backend.clone();
		let poi_consensus_data_provider = PoiConsensusDataProvider::new(
			poi_client.clone(),
			poi_backend.clone(),
		);

		let dcf_client = client.clone();
		let dcf_backend = backend.clone();
		let dcf_consensus_data_provider = DcfConsensusDataProvider::new(
			dcf_client.clone(),
			dcf_backend.clone(),
		);

		let pos_consensus = sc_consensus_pos::start_pos_consensus(
			task_manager.spawn_handle(),
			client.clone(),
			pos_consensus_data_provider,
			proposer_factory.clone(),
			network.clone(),
		);

		let poi_consensus = sc_consensus_poi::start_poi_consensus(
			task_manager.spawn_handle(),
			client.clone(),
			poi_consensus_data_provider,
			proposer_factory.clone(),
			network.clone(),
		);

		let dcf_consensus = sc_consensus_dcf::start_dcf_consensus(
			task_manager.spawn_handle(),
			client.clone(),
			dcf_consensus_data_provider,
			proposer_factory,
			network.clone(),
		);

		task_manager.spawn_essential_handle().spawn_blocking(
			"pos-consensus",
			Some("consensus"),
			pos_consensus,
		);

		task_manager.spawn_essential_handle().spawn_blocking(
			"poi-consensus",
			Some("consensus"),
			poi_consensus,
		);

		task_manager.spawn_essential_handle().spawn_blocking(
			"dcf-consensus",
			Some("consensus"),
			dcf_consensus,
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}
