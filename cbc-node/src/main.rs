

mod benchmarking;
mod block_tracker;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

use clap::Parser;
use std::path::PathBuf;

fn main() -> sc_cli::Result<()> {
    let cli = cli::Cli::parse();

    if let Some(log_file) = &cli.log_file {
        let log_path = PathBuf::from(log_file);
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create log directory");
        }
    }

    command::run()
}