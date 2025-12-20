use sc_service::ChainType;
use cbc_runtime::WASM_BINARY;
use sc_chain_spec::Properties;
use serde::{Deserialize, Serialize};

// Chain constants
pub const TOKEN_SYMBOL: &str = "CBC";
pub const TOKEN_DECIMALS: u8 = 12;
pub const PROTOCOL_ID: &str = "cbc";

pub type ChainSpec = sc_service::GenericChainSpec;

/// Chain metadata structure containing CBC-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    pub cbc_epoch_length: u32,
    pub cbc_validator_count: u32,
}

/// Returns chain properties with token information
pub fn chain_properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    properties.insert("ss58Format".into(), 42.into());
    properties
}

/// Returns chain metadata with CBC-specific parameters
#[allow(dead_code)]
pub fn chain_metadata() -> ChainMetadata {
    ChainMetadata {
        cbc_epoch_length: 100,
        cbc_validator_count: 10,
    }
}

/// Example bootnode configurations for different networks
#[allow(dead_code)]
pub fn example_bootnodes() -> Vec<String> {
    vec![
        // Example bootnode addresses - these would be replaced with actual bootnode addresses
        "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".to_string(),
        "/ip4/127.0.0.1/tcp/30334/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD".to_string(),
        "/dns/bootnode1.cbc-chain.io/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".to_string(),
        "/dns/bootnode2.cbc-chain.io/tcp/30333/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD".to_string(),
    ]
}

/// Example telemetry endpoints
#[allow(dead_code)]
pub fn example_telemetry_endpoints() -> Vec<(String, u8)> {
    vec![
        ("wss://telemetry.polkadot.io/submit/".to_string(), 0),
        ("wss://telemetry.cbc-chain.io/submit/".to_string(), 0),
    ]
}

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(
            WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
            None,
        )
        .with_name("CBC Development Chain")
        .with_id("cbc_dev")
        .with_chain_type(ChainType::Development)
        .with_protocol_id(PROTOCOL_ID)
        .with_properties(chain_properties())
        .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
        .build()
    )
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    
    // Created a minimal chain spec with properties and metadata
    let chain_spec = ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("CBC Local Testnet")
    .with_id("cbc_local")
    .with_chain_type(ChainType::Local)
    .with_protocol_id(PROTOCOL_ID)
    .with_properties(chain_properties())
    .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
    .build();
    
    Ok(chain_spec)
}

pub fn multi_validator_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    
    let chain_spec = ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("CBC Multi-Validator Testnet")
    .with_id("cbc_multi_validator")
    .with_chain_type(ChainType::Local)
    .with_protocol_id(PROTOCOL_ID)
    .with_properties(chain_properties())
    .with_genesis_config_preset_name("multi_validator")
    .build();
    
    Ok(chain_spec)
}

pub fn high_stake_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    
    let chain_spec = ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("CBC High-Stake Testnet")
    .with_id("cbc_high_stake")
    .with_chain_type(ChainType::Local)
    .with_protocol_id(PROTOCOL_ID)
    .with_properties(chain_properties())
    .with_genesis_config_preset_name("high_stake")
    .build();
    
    Ok(chain_spec)
}