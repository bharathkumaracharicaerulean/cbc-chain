//! CBC Node Template CLI library.
//!
//! This file acts as the main entry point for the CBC node's command-line interface.
//! It wires up the CLI, loads submodules, and passes execution control to the command dispatcher.

#![warn(missing_docs)]
// This directive tells the Rust compiler to issue a warning if any public items are missing documentation comments (`///`).
// It's a best practice in library and application development to keep your code self-documented.

// === Module Declarations ===
// Each `mod` statement here declares a Rust module defined in its respective file in the same directory.
// These modules are part of the CBC Chain CLI and handle specific functionalities.

mod benchmarking; // Contains benchmarking setup logic for runtime weight calculations
mod chain_spec;   // Defines different chain specifications (e.g. dev, local, custom JSON)
mod cli;          // CLI argument parsing using StructOpt or clap
mod command;      // Command execution logic — this is where the `run()` function lives
mod rpc;          // RPC configuration and endpoint setup (used if custom RPCs are defined)
mod service;      // Node service construction (e.g., partial and full service builders)

/// The entry point of the application.
/// This function is executed when you run the binary (e.g., `./cbc-node --help`)
fn main() -> sc_cli::Result<()> {
	// Delegate execution to the `run()` function defined in the `command` module.
	// This function parses CLI arguments and invokes the appropriate subcommand.
	command::run()
}
