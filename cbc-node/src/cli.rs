use sc_cli::RunCmd; // Import the `RunCmd` struct from the Substrate CLI library, which provides the main command for running a node.

/// The `Cli` struct represents the command-line interface (CLI) for the node.
///
/// This struct uses the `clap` crate to define and parse command-line arguments.
/// It includes a `subcommand` field for handling specific subcommands and a `run` field for the main node execution command.
#[derive(Debug, clap::Parser)] // Derive the `Parser` trait to enable command-line argument parsing.
pub struct Cli {
    /// The subcommand to execute.
    ///
    /// This field allows the user to specify a subcommand (e.g., `build-spec`, `check-block`, etc.).
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// The main command for running the node.
    ///
    /// This field uses the `RunCmd` struct from the CBC CLI library to handle node execution.
    #[clap(flatten)] // Flatten this field into the top-level CLI structure.
    pub run: RunCmd,
}

/// The `Subcommand` enum defines the various subcommands available in the CLI.
///
/// Each variant corresponds to a specific functionality, such as key management, chain specification, block validation, etc.
/// The `clap::Subcommand` derive macro is used to enable parsing of subcommands.
#[derive(Debug, clap::Subcommand)] // Derive the `Subcommand` trait to enable subcommand parsing.
#[allow(clippy::large_enum_variant)] // Suppress the Clippy lint for large enum variants.
pub enum Subcommand {
    /// Key management CLI utilities.
    ///
    /// This subcommand provides tools for managing keys, such as generating, inspecting, and signing keys.
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    ///
    /// This subcommand generates a chain specification file, which defines the configuration of the blockchain.
    /// Chain specs are used to initialize the blockchain with specific parameters, such as the genesis state.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    ///
    /// This subcommand validates blocks in the blockchain to ensure they are well-formed and adhere to the consensus rules.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ///
    /// This subcommand exports blocks from the blockchain to a file. It is useful for debugging or sharing blockchain data.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain specification.
    ///
    /// This subcommand exports the state of a specific block and converts it into a chain specification file.
    /// This is useful for creating a new chain based on the state of an existing chain.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ///
    /// This subcommand imports blocks from a file into the blockchain. It is useful for syncing or restoring the chain.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    ///
    /// This subcommand deletes all blockchain data, including blocks, state, and database files.
    /// It is useful for resetting the node or starting fresh.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    ///
    /// This subcommand reverts the blockchain to a previous state by removing recent blocks.
    /// It is useful for recovering from errors or testing specific scenarios.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    ///
    /// This subcommand provides tools for benchmarking the runtime and measuring the performance of specific operations.
    /// Benchmarking is essential for optimizing the runtime and ensuring it meets performance requirements.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Display database meta columns information.
    ///
    /// This subcommand provides information about the database's meta columns, which store auxiliary data for the blockchain.
    /// It is useful for debugging and understanding the structure of the blockchain database.
    ChainInfo(sc_cli::ChainInfoCmd),
}