// use sc_service::ChainType; // Import the ChainType enum to specify the type of blockchain (e.g., Development, Local, etc.)
// use cbc_runtime::WASM_BINARY; // Import the WASM binary for the runtime, which is required to build the chain specification.

// /// Specialized `ChainSpec`. 
// /// This is a type alias for the CBC `GenericChainSpec`, which is used to define the configuration of a blockchain.
// /// The `ChainSpec` contains information such as the chain name, ID, type, and genesis configuration.
// pub type ChainSpec = sc_service::GenericChainSpec;


// /// Generates the chain specification for a development chain.
// ///
// /// This function creates a `ChainSpec` for a development environment, which is typically used for testing purposes.
// /// The development chain is a single-node blockchain with a predefined genesis configuration.
// ///
// /// # Returns
// /// A `Result` containing the `ChainSpec` for the development chain or an error message if the WASM binary is unavailable.
// pub fn development_chain_spec() -> Result<ChainSpec, String> {
//     Ok(
//         ChainSpec::builder(
//             // Use the WASM binary for the runtime. If it's not available, return an error.
//             WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
//             None, // No additional properties are provided for the chain spec.
//         )
//         .with_name("Development") // Set the name of the chain to "Development".
//         .with_id("dev") // Set the unique identifier for the chain to "dev".
//         .with_chain_type(ChainType::Development) // Specify that this is a development chain.
//         .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET) // Use the development runtime preset for the genesis configuration.
//         .build() // Build and return the chain specification.
//     )
// }

// /// Generates the chain specification for a local testnet.
// ///
// /// This function creates a `ChainSpec` for a local testnet, which is typically used for testing with multiple nodes.
// /// The local testnet allows developers to simulate a real blockchain environment on their local machines.
// ///
// /// # Returns
// /// A `Result` containing the `ChainSpec` for the local testnet or an error message if the WASM binary is unavailable.
// pub fn local_chain_spec() -> Result<ChainSpec, String> {
//     Ok(
//         ChainSpec::builder(
//             // Use the WASM binary for the runtime. If it's not available, return an error.
//             WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
//             None, // No additional properties are provided for the chain spec.
//         )
//         .with_name("Local Testnet") // Set the name of the chain to "Local Testnet".
//         .with_id("local_testnet") // Set the unique identifier for the chain to "local_testnet".
//         .with_chain_type(ChainType::Local) // Specify that this is a local testnet chain.
//         .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET) // Use the local testnet runtime preset for the genesis configuration.
//         .build() // Build and return the chain specification.
//     )
// }
use sc_service::ChainType; // Import the ChainType enum to specify the type of blockchain
use cbc_runtime::{
    WASM_BINARY, // Import the WASM binary for the runtime
    AccountId, 
    BalancesConfig, 
    GenesisConfig, 
    SystemConfig,
    SudoConfig,
};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_genesis_builder::Result as GenesisResult; // Import Result type for genesis builder
use sp_genesis_builder::GenesisBuilder; // Import GenesisBuilder for custom genesis configuration

/// Specialized `ChainSpec`.
/// This is a type alias for the CBC `GenericChainSpec`, which is used to define the configuration of a blockchain.
/// The `ChainSpec` contains information such as the chain name, ID, type, and genesis configuration.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Configure initial storage state for the development chain.
///
/// This function creates a custom genesis configuration for the development chain 
/// with multiple predefined accounts and their initial token balances.
fn development_genesis() -> GenesisResult<GenesisConfig> {
    let root = get_account_id_from_seed::<sr25519::Public>("Alice");
    
    // Define multiple accounts with their initial balances
    // This is the BalancesConfig that you need to extend
    let mut balances = vec![
        (root.clone(), 1_000_000_000_000_000), // Alice (sudo account) with 1,000,000 tokens
        (get_account_id_from_seed::<sr25519::Public>("Rishit"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Charlie"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Dave"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Eve"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Ferdie"), 500_000_000_000_000),
        // Add more accounts as needed
    ];
    
    // You can also add real-world accounts if needed
    // Example of adding a specific account using SS58 address format:
    // let account1 = AccountId::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
    // balances.push((account1, 1_000_000_000_000_000));

    // Build the genesis configuration
    GenesisBuilder::default()
        .with_wasm_binary(WASM_BINARY.unwrap())
        .with_balances_config(|config: &mut BalancesConfig| {
            config.balances = balances;
        })
        .with_sudo_config(|config: &mut SudoConfig| {
            config.key = Some(root);
        })
        .build()
}

/// Configure initial storage state for the local testnet.
///
/// This function creates a custom genesis configuration for the local testnet
/// with multiple predefined accounts and their initial token balances.
fn local_testnet_genesis() -> GenesisResult<GenesisConfig> {
    let root = get_account_id_from_seed::<sr25519::Public>("Alice");
    
    // Define multiple accounts with their initial balances for the local testnet
    let mut balances = vec![
        (root.clone(), 1_000_000_000_000_000), // Alice (sudo account)
        (get_account_id_from_seed::<sr25519::Public>("Rishit"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Charlie"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Dave"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Eve"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Ferdie"), 500_000_000_000_000),
        // Add additional accounts for testing
        (get_account_id_from_seed::<sr25519::Public>("George"), 500_000_000_000_000),
        (get_account_id_from_seed::<sr25519::Public>("Harry"), 500_000_000_000_000),
    ];

    // Build the genesis configuration
    GenesisBuilder::default()
        .with_wasm_binary(WASM_BINARY.unwrap())
        .with_balances_config(|config: &mut BalancesConfig| {
            config.balances = balances;
        })
        .with_sudo_config(|config: &mut SudoConfig| {
            config.key = Some(root);
        })
        .build()
}

/// Generates the chain specification for a development chain.
///
/// This function creates a `ChainSpec` for a development environment, which is typically used for testing purposes.
/// The development chain is a single-node blockchain with a predefined genesis configuration.
///
/// # Returns
/// A `Result` containing the `ChainSpec` for the development chain or an error message if the WASM binary is unavailable.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    // CUSTOMIZATION GUIDE:
    // - To override chain name: Change "Development" in .with_name()
    // - To override chain ID: Change "dev" in .with_id()
    // - To override properties: Add a sc_service::Properties object and use .with_properties()
    
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "CBC".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());
    
    // Instead of using the preset, use our custom genesis function
    let genesis = development_genesis().map_err(|e| format!("{:?}", e))?;
    
    ChainSpec::from_genesis(
        // Name of the chain
        "Development",
        // Unique identifier for the chain
        "dev",
        // Chain type (Development, Local, Live, Custom)
        ChainType::Development,
        // Genesis config builder function
        move || genesis.clone(),
        // Bootnodes - override with your own if needed
        Vec::new(),
        // Telemetry endpoints - set to None for development
        None,
        // Protocol ID - can be customized if needed
        None,
        // Fork ID - can be used to identify specific forks
        None,
        // Chain properties - token symbol, decimals, etc.
        Some(properties),
        // Extensions - additional features for the chain
        None,
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
    // CUSTOMIZATION GUIDE:
    // - To override chain name: Change "Local Testnet" in .with_name()
    // - To override chain ID: Change "local_testnet" in .with_id()
    // - To override bootnodes: Add multiaddresses to the Vec in the ChainSpec::from_genesis call
    // Example bootnode: "/ip4/192.168.1.100/tcp/30333/p2p/QmPeerID..."
    
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "CBC".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());
    
    // Instead of using the preset, use our custom genesis function
    let genesis = local_testnet_genesis().map_err(|e| format!("{:?}", e))?;
    
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || genesis.clone(),
        // Bootnodes can be specified here
        Vec::new(),
        None,
        None,
        None,
        Some(properties),
        None,
    )
}

/// Generates a JSON chain specification file from a ChainSpec.
///
/// This function converts a ChainSpec to a JSON string, which can be saved to a file
/// and used to start a node with a custom chain specification.
///
/// # Arguments
/// * `chain_spec` - The ChainSpec to convert to JSON
///
/// # Returns
/// A Result containing the JSON string or an error message
pub fn chain_spec_to_json(chain_spec: &ChainSpec) -> Result<String, String> {
    serde_json::to_string_pretty(&chain_spec).map_err(|e| format!("{:?}", e))
}

/* 
 * HOW TO GENERATE AND USE A JSON CHAIN SPEC:
 * ------------------------------------------
 *
 * 1. Generate the raw chain spec:
 *    $ ./target/release/cbc-node build-spec --chain=dev > chain-spec-plain.json
 *
 * 2. Convert to raw format:
 *    $ ./target/release/cbc-node build-spec --chain=chain-spec-plain.json --raw > chain-spec-raw.json
 *
 * 3. Start a node with the generated chain spec:
 *    $ ./target/release/cbc-node --chain=chain-spec-raw.json
 *
 * TESTING PROCESS:
 * ---------------
 * 1. Build your node:
 *    $ cargo build --release
 *
 * 2. Generate and test your chain spec with the commands above
 *
 * 3. You can verify the balances by:
 *    - Starting your node
 *    - Connecting to it with the Polkadot JS Apps (https://polkadot.js.org/apps/)
 *    - Checking the Accounts section to see if your accounts have the correct balances
 */