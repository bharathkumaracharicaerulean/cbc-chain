//! Runtime API implementations for Substrate-based blockchain
//!
//! This module contains all the runtime API implementations for the CBC Chain. Runtime APIs
//! are the interface between the runtime (on-chain logic) and the client (off-chain logic).
//! They allow the client to query runtime state and submit transactions.
//!
//! ## Core APIs
//! - **Core API**: Basic runtime operations like version, block execution, and initialization
//! - **Metadata API**: Provides runtime metadata for client-side type information
//! - **View Function API**: Executes view functions (read-only operations) on the runtime
//!
//! ## Block Building & Transaction APIs
//! - **BlockBuilder API**: Handles extrinsic application, block finalization, and inherents
//! - **TaggedTransactionQueue API**: Validates transactions before inclusion in blocks
//!
//! ## Consensus APIs
//! - **Aura API**: Authority Round consensus mechanism for block production
//! - **Grandpa API**: Finality gadget for block finalization
//! - **Session Keys API**: Manages session key generation and decoding
//!
//! ## Account & Payment APIs
//! - **AccountNonce API**: Retrieves account nonces for transaction ordering
//! - **TransactionPayment API**: Calculates transaction fees and payment information
//! - **TransactionPaymentCall API**: Fee calculation for specific runtime calls
//!
//! ## Development & Testing APIs
//! - **Benchmark API**: Runtime benchmarking for performance testing (feature-gated)
//! - **TryRuntime API**: Testing and migration utilities (feature-gated)
//! - **GenesisBuilder API**: Genesis state construction and preset management
//!
//! ## Custom CBC APIs
//! - **CbcCustomApi**: Custom APIs specific to CBC Chain functionality including:
//!   - Validator profile information (scores, slashing history)
//!   - Proof of Inference results
//!   - Epoch management
//!   - Block author prediction

// External Crates
use alloc::vec::Vec;
use frame_support::{
	genesis_builder_helper::{build_state, get_preset},
	weights::Weight,
};
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	traits::{Block as BlockT, NumberFor},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_version::RuntimeVersion;

// Local Imports (You should replace these with actual paths if your layout is different)
use super::{
	AccountId, Aura, Balance, Block, Executive, Grandpa, InherentDataExt, Nonce, Runtime,
	RuntimeCall, RuntimeGenesisConfig, SessionKeys, System, TransactionPayment, VERSION,
};

// Add these lines for pallet aliases
use crate::{PalletCbcPos, PalletCbcPoi};

// Begin API Implementations
impl_runtime_apis! {
	// Core Runtime API - Provides fundamental runtime operations
	impl sp_api::Core<Block> for Runtime {
		/// Returns the runtime version information
		fn version() -> RuntimeVersion {
			VERSION
		}
		/// Executes a block by applying all its extrinsics
		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}
		/// Initializes a new block with the given header
		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	// Metadata API - Provides runtime metadata for client-side type information
	impl sp_api::Metadata<Block> for Runtime {
		/// Returns the runtime metadata for client-side type information
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		/// Returns metadata for a specific version
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}
		/// Returns all available metadata versions
		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	// View Function API - Executes read-only operations on the runtime
	impl frame_support::view_functions::runtime_api::RuntimeViewFunction<Block> for Runtime {
		/// Executes a view function (read-only operation) on the runtime
		fn execute_view_function(id: frame_support::view_functions::ViewFunctionId, input: Vec<u8>) -> Result<Vec<u8>, frame_support::view_functions::ViewFunctionDispatchError> {
			Runtime::execute_view_function(id, input)
		}
	}

	// Block Builder API - Handles block construction and extrinsic processing
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		/// Applies an extrinsic to the runtime state
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}
		/// Finalizes the current block and returns its header
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}
		/// Creates inherent extrinsics from the provided inherent data
		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}
		/// Checks that all inherents are valid for the given block
		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	// Transaction Validation API - Validates transactions before block inclusion
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		/// Validates a transaction and returns its validity information
		fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic, block_hash: <Block as BlockT>::Hash) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	// Offchain Worker API - Handles offchain worker execution
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		/// Executes offchain worker logic for the given block header
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	// Aura Consensus API - Authority Round consensus mechanism
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		/// Returns the slot duration in milliseconds
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}
		/// Returns the list of current authorities
		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	// Session Keys API - Manages session key generation and decoding
	impl sp_session::SessionKeys<Block> for Runtime {
		/// Generates new session keys from an optional seed
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
		/// Decodes session keys into raw public keys with their type IDs
		fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	// Grandpa Finality API - Finality gadget for block finalization
	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		/// Returns the current Grandpa authorities
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}
		/// Returns the current set ID
		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}
		/// Submits an equivocation report (currently disabled)
		fn submit_report_equivocation_unsigned_extrinsic(
			_: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>
			>,
			_: sp_consensus_grandpa::OpaqueKeyOwnershipProof
		) -> Option<()> {
			None
		}
		/// Generates key ownership proof (currently disabled)
		fn generate_key_ownership_proof(_: sp_consensus_grandpa::SetId, _: GrandpaId) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			None
		}
	}

	// Account Nonce API - Retrieves account nonces for transaction ordering
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		/// Returns the current nonce for a given account
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	// Transaction Payment Info API - Calculates transaction fees and payment information
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		/// Queries dispatch info for a transaction
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		/// Queries fee details for a transaction
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		/// Converts weight to fee
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		/// Converts length to fee
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	// Transaction Call Payment API - Fee calculation for specific runtime calls
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall> for Runtime {
		/// Queries dispatch info for a specific runtime call
		fn query_call_info(call: RuntimeCall, len: u32) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		/// Queries fee details for a specific runtime call
		fn query_call_fee_details(call: RuntimeCall, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		/// Converts weight to fee for calls
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		/// Converts length to fee for calls
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	// Runtime Benchmarking API - Performance testing utilities (feature-gated)
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		/// Returns benchmark metadata and storage information
		fn benchmark_metadata(extra: bool) -> (Vec<frame_benchmarking::BenchmarkList>, Vec<frame_support::traits::StorageInfo>) {
			use frame_benchmarking::{baseline, BenchmarkList};
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
			use baseline::Pallet as BaselineBench;
			use super::*;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		/// Dispatches benchmark execution
		fn dispatch_benchmark(config: frame_benchmarking::BenchmarkConfig) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
			use frame_benchmarking::{baseline, BenchmarkBatch};
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
			use baseline::Pallet as BaselineBench;
			use super::*;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<sp_storage::TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	// Try-Runtime Testing API - Testing and migration utilities (feature-gated)
	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		/// Performs runtime upgrade checks and returns weight information
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, super::configs::RuntimeBlockWeights::get().max_block)
		}
		/// Executes a block for testing purposes
		fn execute_block(block: Block, state_root_check: bool, signature_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	// Genesis Builder API - Genesis state construction and preset management
	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		/// Builds genesis state from configuration
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}
		/// Retrieves a genesis preset by ID
		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, crate::genesis_config_presets::get_preset)
		}
		/// Returns all available preset names
		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			crate::genesis_config_presets::preset_names()
		}
	}

	// Custom CBC API - CBC Chain specific functionality
	impl crate::apis::CbcCustomApi<Block> for Runtime {
		/// Retrieves validator profile information including score and slashing history
		fn get_validator_profile(account: AccountId) -> ValidatorProfile {
			ValidatorProfile {
				score: PalletCbcPos::validator_scores(&account),
				slashing_count: PalletCbcPos::slashing_count(&account),
			}
		}
		/// Retrieves inference result for a given account from the POI pallet
		fn get_inference_result(account: AccountId) -> Option<u32> {
			PalletCbcPoi::inference_results(&account)
		}
		/// Returns the current epoch from the POS pallet
		fn get_current_epoch() -> u32 {
			PalletCbcPos::current_epoch()
		}
		/// Returns the expected block author (currently not implemented)
		fn get_expected_block_author() -> Option<AccountId> {
			// Placeholder: No logic for expected block author in current pallets
			None
		}
	}
}

// --- Custom Runtime APIs ---

/// Validator profile information returned by get_validator_profile.
/// Contains score and slashing count information for a validator.
#[derive(codec::Encode, codec::Decode, scale_info::TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ValidatorProfile {
	/// The validator's current score (if available)
	pub score: Option<u32>,
	/// The number of times the validator has been slashed
	pub slashing_count: Option<u32>,
}

/// Custom runtime API trait for CBC Chain specific functionality.
/// This trait defines the interface for CBC-specific runtime operations.
sp_api::decl_runtime_apis! {
	pub trait CbcCustomApi {
		/// Get the validator profile for a given account.
		/// Returns score and slashing count information.
		fn get_validator_profile(account: AccountId) -> ValidatorProfile;
		/// Get the inference result for a given account from the POI pallet.
		/// Returns None if no inference result is available.
		fn get_inference_result(account: AccountId) -> Option<u32>;
		/// Get the current epoch from the POS pallet.
		/// Used for epoch-based operations and time tracking.
		fn get_current_epoch() -> u32;
		/// Get the expected block author (if available).
		/// Currently returns None as this functionality is not implemented.
		fn get_expected_block_author() -> Option<AccountId>;
	}
}
