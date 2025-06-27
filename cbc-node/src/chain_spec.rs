use sc_service::ChainType;
use cbc_runtime::WASM_BINARY;
use cbc_runtime::genesis_config_presets::{development_config_genesis, local_config_genesis};

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Generates the chain specification for a development chain.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "CBC-CHAIN",
        "cbc",
        ChainType::Development,
        move || development_config_genesis(),
        wasm_binary,
        vec![],
        None,
        None,
        None,
    ))
}

/// Generates the chain specification for a local testnet.
pub fn local_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        move || local_config_genesis(),
        wasm_binary,
        vec![],
        None,
        None,
        None,
    ))
}