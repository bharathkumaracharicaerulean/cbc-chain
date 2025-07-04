use sc_service::ChainType; // Import the ChainType enum to specify the type of blockchain (e.g., Development, Local, etc.)
use cbc_runtime::WASM_BINARY; // Import the WASM binary for the runtime, which is required to build the chain specification.

/// Specialized `ChainSpec`. 
/// This is a type alias for the CBC `GenericChainSpec`, which is used to define the configuration of a blockchain.
/// The `ChainSpec` contains information such as the chain name, ID, type, and genesis configuration.
pub type ChainSpec = sc_service::GenericChainSpec;


/// Generates the chain specification for a development chain.
///
/// This function creates a `ChainSpec` for a development environment, which is typically used for testing purposes.
/// The development chain is a single-node blockchain with a predefined genesis configuration.
///
/// # Returns
/// A `Result` containing the `ChainSpec` for the development chain or an error message if the WASM binary is unavailable.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            // Use the WASM binary for the runtime. If it's not available, return an error.
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None, // No additional properties are provided for the chain spec.
        )
        .with_name("CBC-CHAIN") // Set the name of the chain to "Development".
        .with_id("CBC") // Set the unique identifier for the chain to "dev".
        .with_chain_type(ChainType::Development) // Specify that this is a development chain.
        .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET) // Use the development runtime preset for the genesis configuration.
        .build() // Build and return the chain specification.
    )
}

/// Generates the chain specification for a local testnet.
///
/// This function creates a `ChainSpec` for a local testnet, which is typically used for testing with multiple nodes.
/// The local testnet allows developers to simulate a real blockchain environment on their local machines.
///
/// # Returns
/// A `Result` containing the `ChainSpec` for the local testnet or an error message if the WASM binary is unavailable.
pub fn local_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            // Use the WASM binary for the runtime. If it's not available, return an error.
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None, // No additional properties are provided for the chain spec.
        )
        .with_name("Local Testnet") // Set the name of the chain to "Local Testnet".
        .with_id("local_testnet") // Set the unique identifier for the chain to "local_testnet".
        .with_chain_type(ChainType::Local) // Specify that this is a local testnet chain.
        .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET) // Use the local testnet runtime preset for the genesis configuration.
        .build() // Build and return the chain specification.
    )
}

pub fn bob_sudo_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None,
        )
        .with_name("Bob Sudo Chain")
        .with_id("bob_sudo")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_preset_name("bob_sudo") // Use custom preset
        .build()
    )
}






