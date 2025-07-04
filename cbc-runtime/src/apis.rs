//! Runtime API implementations for Substrate-based blockchain

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
	// Core Runtime API
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}
		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}
		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	// Metadata API
	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}
		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	// View Function API
	impl frame_support::view_functions::runtime_api::RuntimeViewFunction<Block> for Runtime {
		fn execute_view_function(id: frame_support::view_functions::ViewFunctionId, input: Vec<u8>) -> Result<Vec<u8>, frame_support::view_functions::ViewFunctionDispatchError> {
			Runtime::execute_view_function(id, input)
		}
	}

	// Block Builder API
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}
		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}
		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	// Transaction Validation API
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic, block_hash: <Block as BlockT>::Hash) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	// Offchain Worker API
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	// Aura Consensus API
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}
		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	// Session Keys API
	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
		fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	// Grandpa Finality API
	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}
		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}
		fn submit_report_equivocation_unsigned_extrinsic(
			_: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>
			>,
			_: sp_consensus_grandpa::OpaqueKeyOwnershipProof
		) -> Option<()> {
			None
		}
		fn generate_key_ownership_proof(_: sp_consensus_grandpa::SetId, _: GrandpaId) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			None
		}
	}

	// Account Nonce API
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	// Transaction Payment Info API
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	// Transaction Call Payment API
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall> for Runtime {
		fn query_call_info(call: RuntimeCall, len: u32) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(call: RuntimeCall, len: u32) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	// Runtime Benchmarking API
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
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

	// Try-Runtime Testing API
	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, super::configs::RuntimeBlockWeights::get().max_block)
		}
		fn execute_block(block: Block, state_root_check: bool, signature_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	// Genesis Builder API
	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}
		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, crate::genesis_config_presets::get_preset)
		}
		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			crate::genesis_config_presets::preset_names()
		}
	}

	impl crate::apis::CbcCustomApi<Block> for Runtime {
		fn get_validator_profile(account: AccountId) -> ValidatorProfile {
			ValidatorProfile {
				score: PalletCbcPos::validator_scores(&account),
				slashing_count: PalletCbcPos::slashing_count(&account),
			}
		}
		fn get_inference_result(account: AccountId) -> Option<u32> {
			PalletCbcPoi::inference_results(&account)
		}
		fn get_current_epoch() -> u32 {
			PalletCbcPos::current_epoch()
		}
		fn get_expected_block_author() -> Option<AccountId> {
			// Placeholder: No logic for expected block author in current pallets
			None
		}
	}
}

// --- Custom Runtime APIs ---

/// Validator profile information for get_validator_profile.
#[derive(codec::Encode, codec::Decode, scale_info::TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ValidatorProfile {
	pub score: Option<u32>,
	pub slashing_count: Option<u32>,
}

sp_api::decl_runtime_apis! {
	pub trait CbcCustomApi {
		/// Get the validator profile for a given account.
		fn get_validator_profile(account: AccountId) -> ValidatorProfile;
		/// Get the inference result for a given account.
		fn get_inference_result(account: AccountId) -> Option<u32>;
		/// Get the current epoch (from POS or POI pallet).
		fn get_current_epoch() -> u32;
		/// Get the expected block author (if available).
		fn get_expected_block_author() -> Option<AccountId>;
	}
}
