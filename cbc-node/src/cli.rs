use sc_cli::RunCmd;
use clap::Parser;
use clap::Args;

#[derive(Debug, Parser)]
#[command(
    name = "cbc-node",
    about = "CBC Chain Node - A Substrate-based blockchain node.",
    author = "Caerulean ByteChains Private Limited",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,

    #[clap(long, default_value = "production", help = "CBC custom mode (e.g., testing, production)")]
    pub cbc_mode: String,

    #[clap(long, help = "Enable CBC custom RPC extensions")]
    pub enable_cbc_extensions: bool,
    
    #[clap(long, help = "Write logs to the given file instead of stdout")]
    pub log_file: Option<String>,

    #[clap(long, help = "Show only CBC-related logs")]
    pub cbc_log_only: bool,

    #[clap(long, help = "Enable unsafe RPC methods (use with caution)")]
    pub unsafe_rpc_expose: bool,

    #[clap(long, default_value = "60", help = "RPC rate limiting window in seconds")]
    pub rpc_rate_limit_window: u64,

    #[clap(long, default_value = "100", help = "Maximum RPC requests per rate limit window")]
    pub rpc_rate_limit_requests: u32,
}

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

    Revert(sc_cli::RevertCmd),

    Faucet(FaucetCmd),

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