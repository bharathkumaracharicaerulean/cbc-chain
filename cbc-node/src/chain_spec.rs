use sc_service::ChainType; // Import the ChainType enum to specify the type of blockchain (e.g., Development, Local, etc.)
use cbc_runtime::WASM_BINARY; // Import the WASM binary for the runtime, which is required to build the chain specification.
use sp_core::sr25519;
use cbc_consensus::{
    ConsensusParams,
    EpochConfig,
    AuthorSelectionMode,
    ValidatorSet,
    AuthorSelection,
};
use hex_literal::hex;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_chain_spec::ChainSpecExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialized `ChainSpec`. 
/// This is a type alias for the CBC `GenericChainSpec`, which is used to define the configuration of a blockchain.
/// The `ChainSpec` contains information such as the chain name, ID, type, and genesis configuration.
pub type ChainSpec = sc_service::GenericChainSpec;

/// DCF configuration for the chain
// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct DcfConfig {
//     pub pos_weight: u32,
//     pub poi_weight: u32,
//     pub validator_scores: Vec<(sr25519::Public, u128, u32)>, // (public_key, stake, inference_score)
// }

/// Generates the chain specification for a development chain.
///
/// This function creates a `ChainSpec` for a development environment, which is typically used for testing purposes.
/// The development chain is a single-node blockchain with a predefined genesis configuration.
///
/// # Returns
/// A `Result` containing the `ChainSpec` for the development chain or an error message if the WASM binary is unavailable.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Define initial validators with their public keys
    let validator1 = sr25519::Public::from_raw(hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));
    let validator2 = sr25519::Public::from_raw(hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"));

    // Example consensus parameters
    let consensus_params = ConsensusParams {
        author_selection_mode: AuthorSelectionMode::Hybrid,
        finality_threshold: 10,
        block_time: 6000,
        max_block_size: 1024 * 1024,
        max_transactions_per_block: 1000,
    };
    let epoch_config = EpochConfig {
        epoch_length: 1000,
        min_validators: 2,
        max_validators: 100,
        min_stake: 1000,
    };

    let initial_validators = vec![
        (validator1, 1000, 90),
        (validator2, 1000, 85),
    ];

    Ok(
        ChainSpec::builder(wasm_binary, None)
            .with_name("CBC-CHAIN")
            .with_id("CBC")
            .with_chain_type(ChainType::Development)
            .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
            .with_properties(|properties| {
                properties.insert("consensus_params".to_string(), serde_json::to_value(consensus_params).unwrap());
                properties.insert("epoch_config".to_string(), serde_json::to_value(epoch_config).unwrap());
                properties.insert("initial_validators".to_string(), serde_json::to_value(initial_validators).unwrap());
            })
            .build()
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
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // Define initial validators with their public keys
    let validator1 = sr25519::Public::from_raw(hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"));
    let validator2 = sr25519::Public::from_raw(hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"));
    let validator3 = sr25519::Public::from_raw(hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"));

    let consensus_params = ConsensusParams {
        author_selection_mode: AuthorSelectionMode::Hybrid,
        finality_threshold: 5,
        block_time: 3000,
        max_block_size: 512 * 1024,
        max_transactions_per_block: 500,
    };
    let epoch_config = EpochConfig {
        epoch_length: 500,
        min_validators: 2,
        max_validators: 50,
        min_stake: 500,
    };

    let initial_validators = vec![
        (validator1, 1000, 90),
        (validator2, 1000, 85),
        (validator3, 1000, 80),
    ];

    Ok(
        ChainSpec::builder(wasm_binary, None)
            .with_name("Local Testnet")
            .with_id("local_testnet")
            .with_chain_type(ChainType::Local)
            .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
            .with_properties(|properties| {
                properties.insert("consensus_params".to_string(), serde_json::to_value(consensus_params).unwrap());
                properties.insert("epoch_config".to_string(), serde_json::to_value(epoch_config).unwrap());
                properties.insert("initial_validators".to_string(), serde_json::to_value(initial_validators).unwrap());
            })
            .build()
    )
}