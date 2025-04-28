// In your chain_spec.rs file

// Import necessary dependencies
use sp_core::{sr25519, Pair, Public};
use node_primitives::{AccountId, Balance};
use cbc_runtime::{
    BalancesConfig, GenesisConfig, SudoConfig, SystemConfig,
    WASM_BINARY, 
};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_service::ChainType;

// Helper function to derive account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
    AccountId: From<AccountPublic>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// Helper function to generate key from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

// Function to create the genesis configuration
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    // Define initial balance amount
    const ENDOWMENT: Balance = 1_000_000_000_000_000_000; // 1 token with 18 decimals

    GenesisConfig {
        system: SystemConfig {
            // Add the wasm binary in genesis
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            // Configure multiple accounts with initial balances
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        sudo: SudoConfig {
            // Assign the sudo key
            key: Some(root_key),
        },
        // Add other module configs as needed
        // ...
    }
}

// Function to create the development chain specification
pub fn development_config() -> ChainSpec {
    // Define accounts to be endowed with tokens
    let endowed_accounts = vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        // Add custom accounts using their public keys
        AccountId::from_ss58check("5GukQt4gJW2XqzFwmm3RHa7x6sYuVcGhuhz72CN7oiBsgffx").unwrap(),
        // Add more accounts as needed
    ];

    ChainSpec::from_genesis(
        // Name of the chain
        "Development",
        // ID of the chain
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                WASM_BINARY.expect("WASM binary was not built, please build it!"),
                vec![],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.clone(),
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry endpoints
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    )
}

// Function to create a custom testnet chain specification
pub fn custom_testnet_config() -> ChainSpec {
    // Define accounts to be endowed with tokens for the testnet
    let endowed_accounts = vec![
        // Add accounts that should have tokens in your testnet
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        AccountId::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap(),
        AccountId::from_ss58check("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap(),
        // Add more accounts as needed
    ];

    // Define initial authorities if using consensus like Aura/GRANDPA
    let initial_authorities = vec![
        // Add authority keys here
    ];

    ChainSpec::from_genesis(
        // Name of the chain - can be overridden with --chain-name
        "My Custom Testnet",
        // ID of the chain - can be overridden with --chain-id
        "my_custom_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                WASM_BINARY.expect("WASM binary was not built, please build it!"),
                initial_authorities.clone(),
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                endowed_accounts.clone(),
                true,
            )
        },
        // Bootnodes - can be overridden with --bootnodes
        vec![
            // Add bootnodes in the format:
            // "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp".parse().unwrap(),
        ],
        // Telemetry endpoints
        None,
        // Protocol ID
        Some("my-custom-testnet"),
        // Properties
        Some(properties()),
        // Extensions
        None,
    )
}

// Define chain properties
fn properties() -> serde_json::map::Map<String, serde_json::Value> {
    let mut properties = serde_json::map::Map::new();
    // Define token symbol - can be overridden at runtime
    properties.insert("tokenSymbol".into(), "CBC".into());
    // Define token decimals - can be overridden at runtime
    properties.insert("tokenDecimals".into(), 18.into());
    // Add other properties as needed
    properties.insert("ss58Format".into(), 42.into());
    
    properties
}
