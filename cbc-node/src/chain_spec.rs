use sc_service::ChainType; // Import the ChainType enum to specify the type of blockchain (e.g., Development, Local, etc.)
use cbc_runtime::WASM_BINARY; // Import the WASM binary for the runtime, which is required to build the chain specification.
use sp_core::sr25519;
use cbc_consensus::types::{
    DcfConfig,
    ValidatorSetConfig,
    AuthorSelectionConfig,
    AuthorSelectionCriteria,
    ProposerConfig,
    FinalityConfig,
};
// use hex_literal::hex; // TODO: Uncomment when hex_literal is available

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

    // Configure DCF consensus parameters
    let dcf_config = DcfConfig {
        finality_blocks: 32,
        min_stake: 1000,
        min_inference_confidence: 80,
    };

    let validator_set_config = ValidatorSetConfig {
        max_validators: 100,
        min_stake: 1000,
        cooldown_period: 100,
        blocks_per_epoch: 1000,
    };

    let author_selection_config = AuthorSelectionConfig {
        criteria: AuthorSelectionCriteria::Hybrid,
        min_stake: 1000,
        cooldown_period: 10,
    };

    let proposer_config = ProposerConfig {
        max_block_size: 1024 * 1024, // 1MB
        max_block_weight: 1_000_000,
        max_transactions: 1000,
        block_time: 6000, // 6 seconds
    };

    let finality_config = FinalityConfig {
        finality_blocks: 10,
        max_finality_time: 60000, // 1 minute
    };

    Ok(
        ChainSpec::builder(wasm_binary, None)
            .with_name("CBC-CHAIN")
            .with_id("CBC")
            .with_chain_type(ChainType::Development)
            .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
            .with_properties(|properties| {
                // Add consensus configuration
                properties.insert("dcf".to_string(), serde_json::to_value(dcf_config).unwrap());
                properties.insert("validator_set".to_string(), serde_json::to_value(validator_set_config).unwrap());
                properties.insert("author_selection".to_string(), serde_json::to_value(author_selection_config).unwrap());
                properties.insert("proposer".to_string(), serde_json::to_value(proposer_config).unwrap());
                properties.insert("finality".to_string(), serde_json::to_value(finality_config).unwrap());
                
                // Add initial validator set
                let initial_validators = vec![
                    (validator1, 1000, 90), // (public_key, stake, inference_score)
                    (validator2, 1000, 85),
                ];
                properties.insert("initial_validators".to_string(), serde_json::to_value(initial_validators).unwrap());
                
                // Add consensus parameters
                let consensus_params = serde_json::json!({
                    "block_time": 6000,
                    "epoch_length": 1000,
                    "finality_threshold": 0.67,
                    "max_validators_per_epoch": 100,
                    "min_stake_requirement": 1000,
                    "inference_threshold": 80
                });
                properties.insert("consensus_params".to_string(), consensus_params);
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

    // Configure DCF consensus parameters for local testnet
    let dcf_config = DcfConfig {
        finality_blocks: 16, // Faster finality for local testing
        min_stake: 500, // Lower stake requirement for testing
        min_inference_confidence: 70, // Lower confidence requirement for testing
    };

    let validator_set_config = ValidatorSetConfig {
        max_validators: 50, // Smaller validator set for local testing
        min_stake: 500,
        cooldown_period: 50,
        blocks_per_epoch: 500,
    };

    let author_selection_config = AuthorSelectionConfig {
        criteria: AuthorSelectionCriteria::Hybrid,
        min_stake: 500,
        cooldown_period: 5,
    };

    let proposer_config = ProposerConfig {
        max_block_size: 512 * 1024, // 512KB for local testing
        max_block_weight: 500_000,
        max_transactions: 500,
        block_time: 3000, // 3 seconds for faster block production
    };

    let finality_config = FinalityConfig {
        finality_blocks: 5,
        max_finality_time: 30000, // 30 seconds for faster finality
    };

    Ok(
        ChainSpec::builder(wasm_binary, None)
            .with_name("Local Testnet")
            .with_id("local_testnet")
            .with_chain_type(ChainType::Local)
            .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
            .with_properties(|properties| {
                properties.insert("dcf".to_string(), serde_json::to_value(dcf_config).unwrap());
                properties.insert("validator_set".to_string(), serde_json::to_value(validator_set_config).unwrap());
                properties.insert("author_selection".to_string(), serde_json::to_value(author_selection_config).unwrap());
                properties.insert("proposer".to_string(), serde_json::to_value(proposer_config).unwrap());
                properties.insert("finality".to_string(), serde_json::to_value(finality_config).unwrap());
            })
            .build()
    )
}