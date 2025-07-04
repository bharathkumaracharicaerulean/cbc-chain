// Import necessary components and traits used in this CLI entry point for the CBC Chain node.
use crate::{
    benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder},
    chain_spec,
    cli::{Cli, Subcommand},

};

use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use sc_cli::SubstrateCli;
use cbc_runtime::{Block, EXISTENTIAL_DEPOSIT};
use sp_keyring::Sr25519Keyring;

/// Implement the `SubstrateCli` trait for the `Cli` struct, which allows Substrate to understand
/// how to interpret and respond to CLI arguments for this specific node.
impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "CBC CHAIN".into()
    }

    fn impl_version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn description() -> String {
        "A CBC Chain runtime node built with Substrate.".into()
    }

    fn author() -> String {
        "Caerulean ByteChains Private Limited".into()
    }

    fn support_url() -> String {
        "https://support.cbytechains.com".into()
    }

    fn copyright_start_year() -> i32 {
        2025
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_chain_spec()?),
            "" | "local" => Box::new(chain_spec::local_chain_spec()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
        })
    }
}

/// Entry point for CLI command execution. This function is typically called from `main.rs`.
/// It handles dispatching each CLI subcommand to the correct logic.
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),

        Some(Subcommand::Faucet(cmd)) => {
            println!(
                "[FAUCET] Would send {} tokens to address {} (dummy, no on-chain transaction).",
                cmd.amount, cmd.to
            );
            Ok(())
        },

        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        },

        // DCF-only: CheckBlock, ImportBlocks, ExportBlocks, ExportState, Revert are not supported
        Some(Subcommand::CheckBlock(_))
        | Some(Subcommand::ImportBlocks(_))
        | Some(Subcommand::ExportBlocks(_))
        | Some(Subcommand::ExportState(_))
        | Some(Subcommand::Revert(_)) => {
            Err("This subcommand is not supported in DCF-only mode (no import queue).".into())
        },

        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        },

        Some(Subcommand::Benchmark(cmd)) => {
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
                        let sc_service::PartialComponents { client, .. } =
                            crate::service::new_partial(&config, &node_config)?;
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
                        let sc_service::PartialComponents { client, backend, .. } =
                            crate::service::new_partial(&config, &node_config)?;
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
                        let sc_service::PartialComponents { client, .. } =
                            crate::service::new_partial(&config, &node_config)?;
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
                        let sc_service::PartialComponents { client, .. } =
                            crate::service::new_partial(&config, &node_config)?;
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
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        },

        None => {
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
                    sc_network::config::NetworkBackendType::Libp2p =>
                        crate::service::new_full::<
                            sc_network::NetworkWorker<
                                cbc_runtime::opaque::Block,
                                <cbc_runtime::opaque::Block as sp_runtime::traits::Block>::Hash,
                            >,
                        >(config, &node_config).map_err(sc_cli::Error::Service),
                    sc_network::config::NetworkBackendType::Litep2p =>
                        crate::service::new_full::<sc_network::Litep2pNetworkBackend>(config, &node_config)
                            .map_err(sc_cli::Error::Service),
                }
            })
        },
    }
}