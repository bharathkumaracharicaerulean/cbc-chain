// This file is part of the CBC Runtime.
// It defines functions for generating JSON-based genesis configurations used by the blockchain node
// when starting a development or local testnet chain.

use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig, SudoConfig}; // Runtime-specific types
use alloc::{vec, vec::Vec}; // Alloc crate for dynamic arrays
use frame_support::build_struct_json_patch; // Macro to build partial JSON patches for genesis config
use serde_json::Value; // JSON value type
use sp_consensus_aura::sr25519::AuthorityId as AuraId; // Aura consensus authority ID
use sp_consensus_grandpa::AuthorityId as GrandpaId; // Grandpa finality authority ID
use sp_genesis_builder::{self, PresetId}; // Genesis builder utilities and PresetId for pre-defined configs
use sp_keyring::Sr25519Keyring; // Keyring to easily access dev accounts
use sp_core::{sr25519, Pair};
use sp_runtime::traits::IdentifyAccount;

/// Helper function to get an account ID from a seed
fn get_account_id_from_seed<TPublic: Pair>(seed: &str) -> AccountId
where
	<TPublic::Public as IdentifyAccount>::AccountId: From<TPublic::Public>,
	TPublic::Public: IdentifyAccount,
	<TPublic::Public as IdentifyAccount>::AccountId: Into<AccountId>,
{
	let pair = TPublic::from_string(seed, None).expect("static seed is valid; qed");
	TPublic::Public::from(pair.public())
		.into_account()
		.into()
}

/// Returns a genesis configuration in JSON format with the specified authorities,
/// endowed accounts, and sudo (root) key.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>, // Validator authority pairs
	endowed_accounts: Vec<AccountId>,              // Accounts pre-funded with balance
	root: AccountId,                               // Root (sudo) key
) -> Value {
	// Create initial validators list (Alice and Bob)
	let _initial_validators = vec![
		Sr25519Keyring::Alice.to_account_id(),
		Sr25519Keyring::Bob.to_account_id(),
	];

	build_struct_json_patch!(RuntimeGenesisConfig {
		// Configure initial balances for all endowed accounts with large amounts of tokens
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1u128 << 61)) // Each gets 2^61 units
				.collect::<Vec<_>>(),
		},
		// Set Aura authorities from the provided initial authorities
		aura: pallet_aura::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		// Set Grandpa authorities with weight (1 in this case)
		grandpa: pallet_grandpa::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		// Assign the sudo (root) key to the provided account
		sudo: SudoConfig { key: Some(root) },
		// Configure initial validators
		pallet_cbc_pos: pallet_cbc_pos::GenesisConfig {
			validators: vec![
				get_account_id_from_seed::<sr25519::Pair>("Alice"),
				get_account_id_from_seed::<sr25519::Pair>("Bob"),
				get_account_id_from_seed::<sr25519::Pair>("Charlie"),
			],
			validator_scores: vec![100, 90, 80],
			slashing_count: vec![],
		},
		// Configure initial inference results
		pallet_cbc_poi: pallet_cbc_poi::GenesisConfig {
			inference_results: vec![],
			challenges: vec![],
		},
	})
}

/// Returns a basic development configuration suitable for running a single-node dev chain.
/// - Uses Alice as the sole authority and sudo.
/// - Endows Alice, Bob, and their stash accounts with tokens.
pub fn development_config_genesis() -> Value {
	testnet_genesis(
		vec![(
			sp_keyring::Sr25519Keyring::Alice.public().into(),
			sp_keyring::Ed25519Keyring::Alice.public().into(),
		)],
		vec![
			Sr25519Keyring::Alice.to_account_id(),
			Sr25519Keyring::Bob.to_account_id(),
		],
		sp_keyring::Sr25519Keyring::Alice.to_account_id(),
	)
}

/// Returns a local testnet configuration:
/// - Alice and Bob are set as authorities.
/// - Endows all keyring accounts (except One and Two) with tokens.
/// - Alice is the sudo key.
pub fn local_config_genesis() -> Value {
	testnet_genesis(
		vec![
			(
				sp_keyring::Sr25519Keyring::Alice.public().into(),
				sp_keyring::Ed25519Keyring::Alice.public().into(),
			),
			(
				sp_keyring::Sr25519Keyring::Bob.public().into(),
				sp_keyring::Ed25519Keyring::Bob.public().into(),
			),
		],
		// Filter out keys "One" and "Two" which are not intended to be endowed
		Sr25519Keyring::iter()
			.filter(|v| v != &Sr25519Keyring::One && v != &Sr25519Keyring::Two)
			.map(|v| v.to_account_id())
			.collect::<Vec<_>>(),
		Sr25519Keyring::Alice.to_account_id(),
	)
}

/// Fetches the JSON representation of the genesis config for the given `PresetId`.
/// - Supports "dev" and "local" presets.
/// - Returns None for unknown presets.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_ref() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
		_ => return None, // Unknown preset
	};
	// Serialize the generated config into JSON and return as byte array
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// Returns the list of preset names that are supported by this runtime.
/// These identifiers can be used when launching the chain with a specific genesis preset.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
	]
}
