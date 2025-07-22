<<<<<<< HEAD
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
=======
use sc_service::ChainType;
use cbc_runtime::WASM_BINARY;

pub type ChainSpec = sc_service::GenericChainSpec;

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None,
        )
        .with_name("CBC-CHAIN")
        .with_id("CBC")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
        .build()
    )
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None,
        )
        .with_name("Local Testnet")
        .with_id("local_testnet")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
        .build()
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
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
<<<<<<< HEAD
        .with_genesis_config_preset_name("bob_sudo") // Use custom preset
        .build()
    )
}






// use sc_service::ChainType;
// use cbc_runtime::{
//     WASM_BINARY,
//     AccountId, 
//     BalancesConfig, 
//     SudoConfig,
// };
// use sp_core::{sr25519, Pair, Public};
// use sp_runtime::traits::{IdentifyAccount, Verify};
// use sp_genesis_builder::Result as GenesisResult;
// use sp_genesis_builder::GenesisBuilder;
// use sp_runtime::MultiSignature;
// use serde_json::{self, Value as JsonValue};
// use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, GenericChainSpec};
// use serde::{Serialize, Deserialize};
// use sp_runtime::generic::{Block, Header};
// use sp_runtime::OpaqueExtrinsic;
// use sc_cli::ChainSpec as ChainSpecTrait;
// use std::path::PathBuf;
// use sc_telemetry::TelemetryEndpoints;
// use sp_runtime::{BuildStorage, Storage};
// use std::collections::BTreeMap;
// use sc_network_types::multiaddr::Multiaddr;

// /// The extensions for the [`ChainSpec`].
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
// #[serde(deny_unknown_fields)]
// pub struct Extensions {
//     /// The relay chain of the parachain.
//     pub relay_chain: String,
//     /// The id of the parachain.
//     pub para_id: u32,
// }

// impl Extensions {
//     /// Try to get the extension from the given `ChainSpec`.
//     pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
//         sc_chain_spec::get_extension(chain_spec.extensions())
//     }
// }

// /// Specialized `ChainSpec`.
// #[derive(Clone)]
// pub struct ChainSpec(GenericChainSpec<Extensions>);

// impl BuildStorage for ChainSpec {
//     fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
//         self.0.assimilate_storage(storage)
//     }
// }

// impl ChainSpec {
//     /// Create a new chain spec builder.
//     pub fn builder(
//         wasm_binary: &[u8],
//         extensions: Extensions,
//     ) -> sc_chain_spec::ChainSpecBuilder<Extensions> {
//         GenericChainSpec::builder(wasm_binary, extensions)
//     }

//     /// Load chain spec from JSON file.
//     pub fn from_json_file(path: PathBuf) -> Result<Self, String> {
//         Ok(ChainSpec(GenericChainSpec::from_json_file(path)?))
//     }
// }

// impl ChainSpecTrait for ChainSpec {
//     fn name(&self) -> &str {
//         self.0.name()
//     }

//     fn id(&self) -> &str {
//         self.0.id()
//     }

//     fn chain_type(&self) -> ChainType {
//         self.0.chain_type()
//     }

//     fn boot_nodes(&self) -> &[Multiaddr] {
//         self.0.boot_nodes()
//     }

//     fn telemetry_endpoints(&self) -> &Option<TelemetryEndpoints> {
//         self.0.telemetry_endpoints()
//     }

//     fn protocol_id(&self) -> Option<&str> {
//         self.0.protocol_id()
//     }

//     fn properties(&self) -> serde_json::Map<String, JsonValue> {
//         self.0.properties()
//     }

//     fn extensions(&self) -> &dyn sc_chain_spec::GetExtension {
//         self.0.extensions()
//     }

//     fn extensions_mut(&mut self) -> &mut dyn sc_chain_spec::GetExtension {
//         self.0.extensions_mut()
//     }

//     fn add_boot_node(&mut self, addr: Multiaddr) {
//         self.0.add_boot_node(addr)
//     }

//     fn as_json(&self, raw: bool) -> Result<String, String> {
//         self.0.as_json(raw)
//     }

//     fn fork_id(&self) -> Option<&str> {
//         self.0.fork_id()
//     }

//     fn as_storage_builder(&self) -> &dyn BuildStorage {
//         self
//     }

//     fn cloned_box(&self) -> Box<dyn ChainSpecTrait> {
//         Box::new(self.clone())
//     }

//     fn set_storage(&mut self, storage: Storage) {
//         self.0.set_storage(storage)
//     }

//     fn code_substitutes(&self) -> BTreeMap<String, Vec<u8>> {
//         self.0.code_substitutes()
//     }
// }

// /// Generate a crypto pair from seed.
// pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
//     TPublic::Pair::from_string(&format!("//{}", seed), None)
//         .expect("static values are valid; qed")
//         .public()
// }

// type AccountPublic = <MultiSignature as Verify>::Signer;

// /// Generate an account ID from seed.
// pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
// where
//     AccountPublic: From<<TPublic::Pair as Pair>::Public>,
// {
//     AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
// }

// /// Configure initial storage state for the development chain.
// fn development_genesis() -> GenesisResult {
//     let root = get_account_id_from_seed::<sr25519::Public>("Alice");
    
//     let mut balances = vec![
//         (root.clone(), 1_000_000_000_000_000),
//         (get_account_id_from_seed::<sr25519::Public>("Bob"), 500_000_000_000_000),
//         (get_account_id_from_seed::<sr25519::Public>("Charlie"), 500_000_000_000_000),
//     ];

//     let mut builder = <dyn GenesisBuilder<Block<Header<u32, sp_runtime::traits::BlakeTwo256>, OpaqueExtrinsic>>>::default();
//     builder
//         .with_wasm_binary(WASM_BINARY.unwrap())
//         .with_balances_config(|config: &mut BalancesConfig| {
//             config.balances = balances;
//         })
//         .with_sudo_config(|config: &mut SudoConfig| {
//             config.key = Some(root);
//         })
//         .build()
// }

// /// Configure initial storage state for the local testnet.
// fn local_testnet_genesis() -> GenesisResult {
//     let root = get_account_id_from_seed::<sr25519::Public>("Alice");
    
//     let mut balances = vec![
//         (root.clone(), 1_000_000_000_000_000),
//         (get_account_id_from_seed::<sr25519::Public>("Bob"), 500_000_000_000_000),
//         (get_account_id_from_seed::<sr25519::Public>("Charlie"), 500_000_000_000_000),
//         (get_account_id_from_seed::<sr25519::Public>("Dave"), 500_000_000_000_000),
//     ];

//     let mut builder = <dyn GenesisBuilder<Block<Header<u32, sp_runtime::traits::BlakeTwo256>, OpaqueExtrinsic>>>::default();
//     builder
//         .with_wasm_binary(WASM_BINARY.unwrap())
//         .with_balances_config(|config: &mut BalancesConfig| {
//             config.balances = balances;
//         })
//         .with_sudo_config(|config: &mut SudoConfig| {
//             config.key = Some(root);
//         })
//         .build()
// }

// /// Generates the chain specification for a development chain.
// pub fn development_chain_spec() -> Result<ChainSpec, String> {
//     let mut properties = sc_service::Properties::new();
//     properties.insert("tokenSymbol".into(), "CBC".into());
//     properties.insert("tokenDecimals".into(), 12.into());
//     properties.insert("ss58Format".into(), 42.into());
    
//     let genesis = development_genesis().map_err(|e| format!("{:?}", e))?;
    
//     Ok(ChainSpec(ChainSpec::builder(
//         WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
//         Extensions {
//             relay_chain: "rococo-local".into(),
//             para_id: 1000,
//         },
//     )
//     .with_name("Development")
//     .with_id("dev")
//     .with_chain_type(ChainType::Development)
//     .with_genesis_config_patch(serde_json::to_value(genesis).map_err(|e| format!("{:?}", e))?)
//     .with_properties(properties)
//     .build()))
// }

// /// Generates the chain specification for a local testnet.
// pub fn local_chain_spec() -> Result<ChainSpec, String> {
//     let mut properties = sc_service::Properties::new();
//     properties.insert("tokenSymbol".into(), "CBC".into());
//     properties.insert("tokenDecimals".into(), 12.into());
//     properties.insert("ss58Format".into(), 42.into());
    
//     let genesis = local_testnet_genesis().map_err(|e| format!("{:?}", e))?;
    
//     Ok(ChainSpec(ChainSpec::builder(
//         WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
//         Extensions {
//             relay_chain: "rococo-local".into(),
//             para_id: 1000,
//         },
//     )
//     .with_name("Local Testnet")
//     .with_id("local_testnet")
//     .with_chain_type(ChainType::Local)
//     .with_genesis_config_patch(serde_json::to_value(genesis).map_err(|e| format!("{:?}", e))?)
//     .with_properties(properties)
//     .build()))
// }

// /// Generates a JSON chain specification file from a ChainSpec.
// ///
// /// This function converts a ChainSpec to a JSON string, which can be saved to a file
// /// and used to start a node with a custom chain specification.
// ///
// /// # Arguments
// /// * `chain_spec` - The ChainSpec to convert to JSON
// ///
// /// # Returns
// /// A Result containing the JSON string or an error message
// pub fn chain_spec_to_json(chain_spec: &ChainSpec) -> Result<String, String> {
//     serde_json::to_string_pretty(&chain_spec).map_err(|e| format!("{:?}", e))
// }

// /* 
//  * HOW TO GENERATE AND USE A JSON CHAIN SPEC:
//  * ------------------------------------------
//  *
//  * 1. Generate the plain chain spec:
//  *    $ ./target/release/cbc-node build-spec --chain=dev > chain-spec-plain.json
//  *
//  * 2. Convert to raw format:
//  *    $ ./target/release/cbc-node build-spec --chain=chain-spec-plain.json --raw > chain-spec-raw.json
//  *
//  * 3. Start a node with the generated chain spec:
//  *    $ ./target/release/cbc-node --chain=chain-spec-raw.json
//  *
//  * OVERRIDING CHAIN PROPERTIES:
//  * ---------------------------
//  * You can override various properties when creating or using a chain spec:
//  * 
//  * - Chain Name: Defined using .with_name() in ChainSpec builder
//  * - Chain ID: Defined using .with_id() in ChainSpec builder
//  * 
//  * - Boot Nodes: Can be specified in three ways:
//  *   a) In the chain spec JSON under "bootNodes" array
//  *   b) Programmatically using chain_spec.add_boot_node()
//  *   c) When starting a node: --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/NODE_PEER_ID
//  *
//  * - Chain Properties:
//  *   - Token Symbol: Set with properties.insert("tokenSymbol".into(), "CBC".into())
//  *   - Token Decimals: Set with properties.insert("tokenDecimals".into(), 12.into())
//  *   - SS58 Format: Set with properties.insert("ss58Format".into(), 42.into())
//  *
//  * TESTING INSTRUCTIONS:
//  * -------------------
//  * 1. Build your node:
//  *    $ cargo build --release
//  *
//  * 2. Generate the chain spec JSON as described above
//  *
//  * 3. Verify accounts and balances:
//  *    - Start your node: ./target/release/cbc-node --chain=chain-spec-raw.json
//  *    - Connect using Polkadot JS Apps: https://polkadot.js.org/apps/
//  *    - Navigate to Accounts tab to verify balances
//  *
//  * 4. Multi-node test network:
//  *    - Start the first node (bootnode):
//  *      $ ./target/release/cbc-node --chain=chain-spec-raw.json --port 30333 --ws-port 9944
//  *    - Get its peer ID from the logs
//  *    - Start additional nodes:
//  *      $ ./target/release/cbc-node --chain=chain-spec-raw.json --port 30334 --ws-port 9945 \
//  *        --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/FIRST_NODE_PEER_ID
//  */
=======
        .with_genesis_config_preset_name("bob_sudo")
        .build()
    )
}
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
