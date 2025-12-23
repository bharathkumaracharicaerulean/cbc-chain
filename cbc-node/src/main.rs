

mod benchmarking;
mod block_tracker;
mod chain_spec;
mod cli;
mod command;
mod logging;
mod rpc;
mod service;

use clap::Parser;

fn main() -> sc_cli::Result<()> {
    let cli = cli::Cli::parse();

    // Configure RUST_LOG environment variable for CBC logging
    let log_config = logging::init_cbc_logging_config(cli.cbc_log_only);
    
    // Display startup information before Substrate takes over
    logging::display_startup_info(
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
        "CBC Chain",
        "Node",
        None, // Peer ID will be set later in service startup
        None, // Network latency will be measured later
    );

    // If log file is specified, inform user about shell redirection
    if let Some(ref log_file) = cli.log_file {
        println!("Note: To redirect logs to {}, use shell redirection:", log_file);
        println!("  ./target/release/cbc-node --dev > {} 2>&1", log_file);
        println!("Or use the standard Substrate logging with -l flag");
        println!("Configured RUST_LOG: {}", log_config);
    }

    command::run()
}