// Import necessary components and traits used in this CLI entry point for the CBC Chain node.
use crate::{
	benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder}, // Benchmarking tools
	chain_spec,      // Chain spec definitions
	cli::{Cli, Subcommand}, // CLI definitions and subcommands
	service,         // Node service creation utilities
	rpc,             // RPC configuration
};

use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE}; // Benchmarking CLI tools
use sc_cli::SubstrateCli; // Trait for defining CLI-related metadata and behavior
use sc_service::PartialComponents; // Struct to work with partially built service components
use cbc_runtime::{Block, EXISTENTIAL_DEPOSIT}; // Custom runtime types
use sp_keyring::Sr25519Keyring; // Useful dev/test keyring accounts

/// Implement the `SubstrateCli` trait for the `Cli` struct, which allows Substrate to understand
/// how to interpret and respond to CLI arguments for this specific node.
impl SubstrateCli for Cli {
	fn impl_name() -> String {
		// Name of the implementation, shows up in help output and logs
		"CBC CHAIN".into()
	}

	fn impl_version() -> String {
		// Uses a compile-time environment variable (set in `build.rs`) to show version info
		// env!("CBC_CLI_IMPL_VERSION").into()
		env!("CARGO_PKG_VERSION").into()

	}

	fn description() -> String {
		// User-friendly description of what this node does
		"A CBC Chain runtime node built with Substrate.".into()
	}

	fn author() -> String {
		// Name of the authors or maintaining organization
		"Caerulean ByteChains Private Limited".into()
	}

	fn support_url() -> String {
		// URL for community or customer support
		"https://support.cbytechains.com".into()
	}

	fn copyright_start_year() -> i32 {
		// Year the project officially started
		2025
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		// Load the correct chain specification (network config) based on the `--chain` argument
		Ok(match id {
			"dev" => Box::new(chain_spec::development_chain_spec()?), // Development config
			"" | "local" => Box::new(chain_spec::local_chain_spec()?), // Local node config
			path => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?), // Load from a JSON file
		})
	}
}

/// Entry point for CLI command execution. This function is typically called from `main.rs`.
/// It handles dispatching each CLI subcommand to the correct logic.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args(); // Parse command-line arguments into the `Cli` struct

	match &cli.subcommand {
		// === Handle various CLI subcommands ===

		Some(Subcommand::Key(cmd)) => cmd.run(&cli), // Key management commands (e.g., generate, inspect)

		Some(Subcommand::Faucet(cmd)) => {
			println!(
				"[FAUCET] Would send {} tokens to address {} (dummy, no on-chain transaction).",
				cmd.amount, cmd.to
			);
			Ok(())
		},
		
		Some(Subcommand::BuildSpec(cmd)) => {
			// Generates a genesis chain spec (useful for custom networks)
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},

		Some(Subcommand::CheckBlock(cmd)) => {
			// Checks the integrity and validity of a block file
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				let PartialComponents { client, task_manager, import_queue, .. } = 
					crate::service::new_partial(&config, node_config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::ExportBlocks(cmd)) => {
			// Export blocks from database to a file
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				let PartialComponents { client, task_manager, .. } = 
					crate::service::new_partial(&config, node_config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},

		Some(Subcommand::ExportState(cmd)) => {
			// Export the current state of the chain (can be used to bootstrap a new node)
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				let PartialComponents { client, task_manager, .. } = 
					crate::service::new_partial(&config, node_config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},

		Some(Subcommand::ImportBlocks(cmd)) => {
			// Import blocks from a file into the local database
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				let PartialComponents { client, task_manager, import_queue, .. } = 
					crate::service::new_partial(&config, node_config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::PurgeChain(cmd)) => {
			// Deletes all chain data (useful for a clean start)
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},

		Some(Subcommand::Revert(cmd)) => {
			// Reverts the chain state back by a certain number of blocks
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				let PartialComponents { client, task_manager, backend, .. } = 
					crate::service::new_partial(&config, node_config)?;
				
				// Custom logic to revert Grandpa finality info as well
				let aux_revert = Box::new(|client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},

		Some(Subcommand::Benchmark(cmd)) => {
			// Handle various benchmarking tasks, used for measuring on-chain weight/cost
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				match cmd {
					BenchmarkCmd::Pallet(cmd) => {
						if !cfg!(feature = "runtime-benchmarks") {
							return Err("Runtime benchmarking wasn't enabled. Use `--features runtime-benchmarks`.".into());
						}
						cmd.run_with_spec::<sp_runtime::traits::HashingFor<Block>, ()>(Some(config.chain_spec))
					},

					BenchmarkCmd::Block(cmd) => {
						let node_config = crate::service::NodeConfig {
							rpc_config: crate::rpc::RpcSecurityConfig {
								enable_cbc_extensions: cli.enable_cbc_extensions,
								expose_unsafe_methods: cli.unsafe_rpc_expose,
								rate_limit_window: cli.rpc_rate_limit_window,
								rate_limit_requests: cli.rpc_rate_limit_requests,
							},
						};
						let PartialComponents { client, .. } = 
							crate::service::new_partial(&config, node_config)?;
						cmd.run(client)
					},

					#[cfg(not(feature = "runtime-benchmarks"))]
					BenchmarkCmd::Storage(_) => Err("Storage benchmarking requires `runtime-benchmarks` feature.".into()),

					#[cfg(feature = "runtime-benchmarks")]
					BenchmarkCmd::Storage(cmd) => {
						let node_config = crate::service::NodeConfig {
							rpc_config: crate::rpc::RpcSecurityConfig {
								enable_cbc_extensions: cli.enable_cbc_extensions,
								expose_unsafe_methods: cli.unsafe_rpc_expose,
								rate_limit_window: cli.rpc_rate_limit_window,
								rate_limit_requests: cli.rpc_rate_limit_requests,
							},
						};
						let PartialComponents { client, backend, .. } = 
							crate::service::new_partial(&config, node_config)?;
						let db = backend.expose_db();
						let storage = backend.expose_storage();
						cmd.run(config, client, db, storage)
					},

					BenchmarkCmd::Overhead(cmd) => {
						let node_config = crate::service::NodeConfig {
							rpc_config: crate::rpc::RpcSecurityConfig {
								enable_cbc_extensions: cli.enable_cbc_extensions,
								expose_unsafe_methods: cli.unsafe_rpc_expose,
								rate_limit_window: cli.rpc_rate_limit_window,
								rate_limit_requests: cli.rpc_rate_limit_requests,
							},
						};
						let PartialComponents { client, .. } = 
							crate::service::new_partial(&config, node_config)?;
						let ext_builder = RemarkBuilder::new(client.clone());
						cmd.run(config.chain_spec.name().into(), client, inherent_benchmark_data()?, Vec::new(), &ext_builder, false)
					},

					BenchmarkCmd::Extrinsic(cmd) => {
						let node_config = crate::service::NodeConfig {
							rpc_config: crate::rpc::RpcSecurityConfig {
								enable_cbc_extensions: cli.enable_cbc_extensions,
								expose_unsafe_methods: cli.unsafe_rpc_expose,
								rate_limit_window: cli.rpc_rate_limit_window,
								rate_limit_requests: cli.rpc_rate_limit_requests,
							},
						};
						let PartialComponents { client, .. } = 
							crate::service::new_partial(&config, node_config)?;

						let ext_factory = ExtrinsicFactory(vec![
							Box::new(RemarkBuilder::new(client.clone())),
							Box::new(TransferKeepAliveBuilder::new(
								client.clone(),
								Sr25519Keyring::Alice.to_account_id(),
								EXISTENTIAL_DEPOSIT,
							)),
						]);
						cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
					},

					BenchmarkCmd::Machine(cmd) => cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()),
				}
			})
		},

		Some(Subcommand::ChainInfo(cmd)) => {
			// Outputs information about the configured chain (e.g. genesis hash)
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},

		None => {
			// === Default behavior if no subcommand is specified ===
			// Start the full node service and run until shutdown
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				let node_config = crate::service::NodeConfig {
					rpc_config: crate::rpc::RpcSecurityConfig {
						enable_cbc_extensions: cli.enable_cbc_extensions,
						expose_unsafe_methods: cli.unsafe_rpc_expose,
						rate_limit_window: cli.rpc_rate_limit_window,
						rate_limit_requests: cli.rpc_rate_limit_requests,
					},
				};
				match config.network.network_backend.unwrap_or_default() {
					// Start node with libp2p networking (most common setup)
					sc_network::config::NetworkBackendType::Libp2p => 
						crate::service::new_full::<
							sc_network::NetworkWorker<
								cbc_runtime::opaque::Block,
								<cbc_runtime::opaque::Block as sp_runtime::traits::Block>::Hash,
							>,
						>(config, node_config).map_err(sc_cli::Error::Service),

					// Start node with Litep2p (experimental/lightweight networking)
					sc_network::config::NetworkBackendType::Litep2p =>
						crate::service::new_full::<sc_network::Litep2pNetworkBackend>(config, node_config)
							.map_err(sc_cli::Error::Service),
				}
			})
		},
	}
}
