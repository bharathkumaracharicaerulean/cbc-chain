// This file is part of the CBC Runtime.
// It defines functions for generating JSON-based genesis configurations used by the blockchain node
// when starting a development or local testnet chain.

use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig, SudoConfig}; // Runtime-specific types
use alloc::{vec, vec::Vec}; // Alloc crate for dynamic arrays
use frame_support::build_struct_json_patch; // Macro to build partial JSON patches for genesis config
use serde_json::Value; // JSON value type
use sp_genesis_builder::{self, PresetId}; // Genesis builder utilities and PresetId for pre-defined configs
use sp_keyring::Sr25519Keyring; // Keyring to easily access dev accounts

/// Returns a genesis configuration in JSON format with the specified authorities,
/// endowed accounts, and sudo (root) key.
fn testnet_genesis(
	initial_validators: Vec<AccountId>, // Validator accounts
	endowed_accounts: Vec<AccountId>,   // Accounts pre-funded with balance
	root: AccountId,                    // Root (sudo) key
) -> Value {
	// Create initial validator scores
	let validator_scores = vec![100; initial_validators.len()];

	// Create initial inference results
	let inference_results = initial_validators
		.iter()
		.map(|acc| (acc.clone(), 42))
		.collect::<Vec<_>>();

	build_struct_json_patch!(RuntimeGenesisConfig {
		// Configure initial balances for all endowed accounts with large amounts of tokens
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1u128 << 61)) // Each gets 2^61 units
				.collect::<Vec<_>>(),
		},
		// Assign the sudo (root) key to the provided account
		sudo: SudoConfig { key: Some(root) },
		system: frame_system::GenesisConfig::default(),
		transaction_payment: pallet_transaction_payment::GenesisConfig::default(),
		dcf: pallet_cbc_dcf::GenesisConfig {
			validators: initial_validators.clone(),
			validator_scores: validator_scores.clone(),
			current_epoch: 0,
			epoch_config: pallet_cbc_dcf::EpochConfig {
				blocks_per_epoch: 10,      
				min_stake: 1,              
				max_validators: 24,        
			},
		},
		// Configure initial validators
		pallet_cbc_pos: pallet_cbc_pos::GenesisConfig {
			validators: initial_validators.clone(),
			validator_scores,
			current_epoch: 0,
			slashing_count: vec![],
		},
		// Configure initial inference results
		pallet_cbc_poi: pallet_cbc_poi::GenesisConfig {
			inference_results,
			challenges: vec![],
			current_epoch: 0,
		},
	})
}

/// Returns a basic development configuration suitable for running a single-node dev chain.
/// - Uses Alice as the sole validator and sudo.
/// - Endows Alice, Bob, and their stash accounts with tokens.
pub fn development_config_genesis() -> Value {
	let initial_validators = vec![Sr25519Keyring::Alice.to_account_id()];
	let endowed_accounts = vec![
		Sr25519Keyring::Alice.to_account_id(),
		Sr25519Keyring::Bob.to_account_id(),
	];
	testnet_genesis(
		initial_validators,
		endowed_accounts,
		Sr25519Keyring::Alice.to_account_id(),
	)
}

/// Returns a local testnet configuration:
/// - Alice and Bob are set as validators.
/// - Endows all keyring accounts (except One and Two) with tokens.
/// - Alice is the sudo key.
pub fn local_config_genesis() -> Value {
	let initial_validators = vec![
		Sr25519Keyring::Alice.to_account_id(),
		Sr25519Keyring::Bob.to_account_id(),
	];
	let endowed_accounts = Sr25519Keyring::iter()
		.filter(|v| v != &Sr25519Keyring::One && v != &Sr25519Keyring::Two)
		.map(|v| v.to_account_id())
		.collect::<Vec<_>>();
	testnet_genesis(
		initial_validators,
		endowed_accounts,
		Sr25519Keyring::Alice.to_account_id(),
	)
}

/// Fetches the JSON representation of the genesis config for the given `PresetId`.
/// - Supports "dev" and "local" presets.
/// - Returns None for unknown presets.
pub fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
    match id.as_deref() {
        None => Some(serde_json::to_vec(&development_config_genesis()).unwrap()), // Default to dev
        Some("development") => Some(serde_json::to_vec(&development_config_genesis()).unwrap()),
        Some("local") => Some(serde_json::to_vec(&local_config_genesis()).unwrap()),
        Some("bob_sudo") => Some(serde_json::to_vec(&local_config_genesis()).unwrap()),
        Some("local_testnet") => Some(serde_json::to_vec(&local_config_genesis()).unwrap()),
        _ => None,
    }
}

/// Returns the list of preset names that are supported by this runtime.
/// These identifiers can be used when launching the chain with a specific genesis preset.
pub fn preset_names() -> Vec<PresetId> {
	vec!["default".into()]
}
