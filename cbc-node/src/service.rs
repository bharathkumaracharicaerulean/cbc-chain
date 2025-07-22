<<<<<<< HEAD
//! Service and ServiceFactory implementation.
//!
//! This file defines how the CBC node's services are built and launched,
//! including consensus setup (AURA/GRANDPA), transaction pool, networking,
//! and RPC interfaces. It's a critical part of the node runtime.

use futures::FutureExt; // Needed for handling async functions that return futures.
use sc_client_api::{Backend, BlockBackend}; // Traits for interacting with blockchain backends.
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams}; // AURA consensus building blocks.
use sc_consensus_grandpa::SharedVoterState; // Used for GRANDPA consensus participation.
use sc_service::{error::Error as ServiceError, Configuration, TaskManager, WarpSyncConfig}; // Core service types.
use sc_telemetry::{Telemetry, TelemetryWorker}; // Telemetry for monitoring nodes.
use sc_transaction_pool_api::OffchainTransactionPoolFactory; // For submitting transactions via offchain workers.
use cbc_runtime::{self, apis::RuntimeApi, opaque::Block}; // Use CBC runtime types and APIs.
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair; // AURA authority type.
use std::{sync::Arc, time::Duration}; // Standard concurrency and time utilities.

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
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Alias for the output of `new_partial()` — the essential building blocks of a node.
pub type Service = sc_service::PartialComponents<
	FullClient,
	FullBackend,
	FullSelectChain,
	sc_consensus::DefaultImportQueue<Block>,
	sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
	(
		sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
		sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
		Option<Telemetry>,
	),
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
			let worker = TelemetryWorker::new(16)?; // buffer size
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?; // Flatten nested Result<Option<_>>

	// Create WASM executor for executing runtime logic.
	let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);

	// Build the core node components (client, backend, keystore, task manager).
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client); // Wrap the client in Arc for shared use

	// If telemetry was enabled, spawn its background worker.
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// Longest chain fork choice rule (used by consensus).
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	// Create the transaction pool, responsible for managing pending transactions.
	let transaction_pool = Arc::from(
		sc_transaction_pool::Builder::new(
			task_manager.spawn_essential_handle(),
			client.clone(),
			config.role.is_authority().into(), // Enable pool if this node is an authority
		)
		.with_options(config.transaction_pool.clone())
		.with_prometheus(config.prometheus_registry())
		.build(),
	);

	// Set up GRANDPA consensus logic (used for finality).
	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	// Set up AURA (block production) import queue and inherent data providers.
	let cidp_client = client.clone(); // Clone used inside async closure
	let import_queue =
		sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(ImportQueueParams {
			block_import: grandpa_block_import.clone(),
			justification_import: Some(Box::new(grandpa_block_import.clone())),
			client: client.clone(),
			create_inherent_data_providers: move |parent_hash, _| {
				let cidp_client = cidp_client.clone();
				async move {
					let slot_duration = sc_consensus_aura::standalone::slot_duration_at(
						&*cidp_client,
						parent_hash,
					)?;
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

					Ok((slot, timestamp))
				}
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			check_for_equivocation: Default::default(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			compatibility_mode: Default::default(),
		})?;

	// Return all the components as a tuple for building the full node.
	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (grandpa_block_import, grandpa_link, telemetry),
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
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, grandpa_link, mut telemetry),
	} = new_partial(&config, node_config.clone())?;

	// === Network Setup ===

	// Generate a full network configuration from the node's base config.
	let mut net_config = sc_network::config::FullNetworkConfiguration::<
		Block,
		<Block as sp_runtime::traits::Block>::Hash,
		N,
	>::new(&config.network, config.prometheus_registry().cloned());

	let metrics = N::register_notification_metrics(config.prometheus_registry());

	let peer_store_handle = net_config.peer_store_handle();

	// GRANDPA protocol setup: used for finality synchronization across peers.
	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);

	let (grandpa_protocol_config, grandpa_notification_service) =
		sc_consensus_grandpa::grandpa_peers_set_config::<_, N>(
			grandpa_protocol_name.clone(),
			metrics.clone(),
			peer_store_handle,
		);

	net_config.add_notification_protocol(grandpa_protocol_config);

	// Set up warp sync (fast sync mechanism for GRANDPA finality).
	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));

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
			warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
			block_relay: None,
			metrics,
		})?;

	// === Offchain Workers ===

	// If offchain workers are enabled, start them.
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
				custom_extensions: |_| vec![], // Can inject custom extensions here.
			})?;

		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
		);
	}

	// === Runtime Configuration and Consensus ===

	let role = config.role;
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks: Option<()> = None;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

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
		telemetry: telemetry.as_mut(),
	})?;

	// === AURA Block Authoring Setup ===

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;

		let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(
			StartAuraParams {
				slot_duration,
				client,
				select_chain,
				block_import,
				proposer_factory,
				create_inherent_data_providers: move |_, ()| async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

					Ok((slot, timestamp))
				},
				force_authoring,
				backoff_authoring_blocks,
				keystore: keystore_container.keystore(),
				sync_oracle: sync_service.clone(),
				justification_sync_link: sync_service.clone(),
				block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
				max_block_proposal_slot_portion: None,
				telemetry: telemetry.as_ref().map(|x| x.handle()),
				compatibility_mode: Default::default(),
			},
		)?;

		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	// === GRANDPA Finality Gadget Setup ===

	if enable_grandpa {
		let keystore = if role.is_authority() {
			Some(keystore_container.keystore())
		} else {
			None
		};

		let grandpa_config = sc_consensus_grandpa::Config {
			gossip_duration: Duration::from_millis(333),
			justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
			name: Some(name),
			observer_enabled: false,
			keystore,
			local_role: role,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			protocol_name: grandpa_protocol_name,
		};

		let grandpa_config = sc_consensus_grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network,
			sync: Arc::new(sync_service),
			notification_service: grandpa_notification_service,
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	// All services have started successfully.
	Ok(task_manager)
}
=======
#![allow(unused_variables, static_mut_refs)]
use futures::FutureExt;
use sc_client_api::Backend;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::TelemetryWorker;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use cbc_runtime::{self, apis::RuntimeApi, opaque::Block};
use std::{sync::Arc};
use cbc_consensus::{DcfConsensus, ConsensusParams, AuthorSelectionMode};
use sc_consensus::import_queue::{ImportQueueService, Link};
use std::pin::Pin;
use std::future::Future;
use sp_core::sr25519::Pair;
use sp_consensus::BlockOrigin;

pub struct DummyImportQueue;
impl<B: sp_runtime::traits::Block> sc_service::ImportQueue<B> for DummyImportQueue {
    fn poll_actions(&mut self, _cx: &mut std::task::Context<'_>, _link: &dyn Link<B>) {}
    fn service(&self) -> Box<(dyn ImportQueueService<B> + 'static)> {
        struct DummyService;
        impl<B: sp_runtime::traits::Block> ImportQueueService<B> for DummyService {
            fn import_blocks(&mut self, _origin: BlockOrigin, _blocks: Vec<sc_consensus::IncomingBlock<B>>) {}
            fn import_justifications(&mut self, _peer_id: sc_network::PeerId, _hash: <B as sp_runtime::traits::Block>::Hash, _number: <<B as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number, _justifications: sp_runtime::Justifications) {}
        }
        Box::new(DummyService)
    }
    fn service_ref(&mut self) -> &mut dyn ImportQueueService<B> {
        struct DummyService;
        impl<B: sp_runtime::traits::Block> ImportQueueService<B> for DummyService {
            fn import_blocks(&mut self, _origin: BlockOrigin, _blocks: Vec<sc_consensus::IncomingBlock<B>>) {}
            fn import_justifications(&mut self, _peer_id: sc_network::PeerId, _hash: <B as sp_runtime::traits::Block>::Hash, _number: <<B as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number, _justifications: sp_runtime::Justifications) {}
        }
        static mut SERVICE: DummyService = DummyService;
        unsafe { &mut SERVICE }
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
    DummyImportQueue,
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

    let import_queue = DummyImportQueue;

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

    let consensus_params = ConsensusParams {
        author_selection_mode: AuthorSelectionMode::RoundRobin,
        finality_threshold: 10,
        block_time: 6,
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
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
