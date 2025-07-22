<<<<<<< HEAD
//! Command-line interface definition for CBC Chain Node.

use sc_cli::RunCmd;
use clap::Parser;
use clap:: Args;
/// The main CLI struct for CBC Chain Node.
///
/// Defines all command-line arguments and subcommands available to users.
/// Uses the `clap` crate for parsing and documentation.
=======
use sc_cli::RunCmd;
use clap::Parser;
use clap::Args;

>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
#[derive(Debug, Parser)]
#[command(
    name = "cbc-node",
    about = "CBC Chain Node - A Substrate-based blockchain node.",
    author = "Caerulean ByteChains Private Limited",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
<<<<<<< HEAD
    /// The subcommand to execute (e.g., build-spec, check-block, etc.).
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// The main command for running the node.
    #[clap(flatten)]
    pub run: RunCmd,

    /// CBC custom mode (e.g., "testing", "production").
    #[clap(long, default_value = "production", help = "CBC custom mode (e.g., testing, production)")]
    pub cbc_mode: String,

    /// Enable CBC custom RPC extensions
=======
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,

    #[clap(long, default_value = "production", help = "CBC custom mode (e.g., testing, production)")]
    pub cbc_mode: String,

>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    #[clap(long, help = "Enable CBC custom RPC extensions")]
    pub enable_cbc_extensions: bool,
    
    #[clap(long, help = "Write logs to the given file instead of stdout")]
    pub log_file: Option<String>,

<<<<<<< HEAD
    /// Expose unsafe RPC methods
    #[clap(long, help = "Enable unsafe RPC methods (use with caution)")]
    pub unsafe_rpc_expose: bool,

    /// RPC rate limiting window in seconds
    #[clap(long, default_value = "60", help = "RPC rate limiting window in seconds")]
    pub rpc_rate_limit_window: u64,

    /// Maximum RPC requests per window
=======
    #[clap(long, help = "Enable unsafe RPC methods (use with caution)")]
    pub unsafe_rpc_expose: bool,

    #[clap(long, default_value = "60", help = "RPC rate limiting window in seconds")]
    pub rpc_rate_limit_window: u64,

>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    #[clap(long, default_value = "100", help = "Maximum RPC requests per rate limit window")]
    pub rpc_rate_limit_requests: u32,
}

<<<<<<< HEAD
/// All available subcommands for the CBC Chain Node CLI.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Key management CLI utilities (generate, inspect, sign, etc.).
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks in the blockchain.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks from the blockchain to a file.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain specification.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks from a file into the blockchain.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain (all blocks, state, and database files).
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state by removing recent blocks.
=======
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    BuildSpec(sc_cli::BuildSpecCmd),

    CheckBlock(sc_cli::CheckBlockCmd),

    ExportBlocks(sc_cli::ExportBlocksCmd),

    ExportState(sc_cli::ExportStateCmd),

    ImportBlocks(sc_cli::ImportBlocksCmd),

    PurgeChain(sc_cli::PurgeChainCmd),

>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    Revert(sc_cli::RevertCmd),

    Faucet(FaucetCmd),

<<<<<<< HEAD
    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Display database meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
#[derive(Debug, Args)]
pub struct FaucetCmd {
    /// Destination address to receive tokens
    #[clap(long)]
    pub to: String,

    /// Amount of tokens to send
    #[clap(long)]
    pub amount: u128,
}
=======
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    ChainInfo(sc_cli::ChainInfoCmd),
}

#[derive(Debug, Args)]
pub struct FaucetCmd {
    #[clap(long)]
    pub to: String,

    #[clap(long)]
    pub amount: u128,
}
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
