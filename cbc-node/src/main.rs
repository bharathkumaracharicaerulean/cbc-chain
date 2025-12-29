

mod benchmarking;
mod block_tracker;
mod chain_spec;
mod cli;
mod command;
mod fork_detection;
mod logging;
mod rpc;
mod service;

use clap::Parser;

fn main() -> sc_cli::Result<()> {
    let cli = cli::Cli::parse();

    // Initialize CBC logging system
    if let Err(e) = logging::init_cbc_logging(true, cli.cbc_log_only, cli.log_file.clone()) {
        eprintln!("Warning: Failed to initialize CBC logging: {}", e);
        // Fall back to basic configuration
        let _log_config = logging::init_cbc_logging_config(cli.cbc_log_only);
    }
    
    // Display startup information before Substrate takes over
    logging::display_startup_info(
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
        "CBC Chain",
        "Node",
        None, // Peer ID will be set later in service startup
        None, // Network latency will be measured later
    );

    command::run()
}