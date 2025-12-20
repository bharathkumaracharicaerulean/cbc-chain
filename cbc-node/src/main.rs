

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

    // Initialize CBC logging with log file support
    let enable_colors = atty::is(atty::Stream::Stdout);
    let (_deduplicator, _file_writer) = match logging::init_cbc_logging(
        cli.cbc_log_only, 
        enable_colors, 
        cli.log_file.clone()
    ) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to initialize CBC logging: {}", e);
            // Continue without custom logging if initialization fails
            return command::run();
        }
    };

    // Display startup information
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