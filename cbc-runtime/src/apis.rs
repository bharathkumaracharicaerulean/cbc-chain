//! Runtime API implementations for Substrate-based blockchain

// External Crates
use alloc::vec::Vec;
use frame_support::{
	genesis_builder_helper::{build_state},
	weights::Weight,
};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	traits::{Block as BlockT},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_version::RuntimeVersion;

// Local Imports
use super::{
	AccountId, Balance, Block, Executive, InherentDataExt, Nonce, Runtime,
	RuntimeCall, RuntimeGenesisConfig, SessionKeys, System, TransactionPayment, VERSION,
};

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

	// Session Keys API
	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
		fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
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

	// POS API
	impl pallet_cbc_pos::PosApi<Block, AccountId, Balance> for Runtime {
		fn get_validator_stake(validator: AccountId) -> Balance {
			pallet_cbc_pos::Pallet::<Runtime>::stake(&validator)
		}

		fn get_validator_score(validator: AccountId) -> u32 {
			pallet_cbc_pos::Pallet::<Runtime>::validator_scores(&validator)
				.unwrap_or_default()
		}

		fn get_active_validators() -> Vec<AccountId> {
			// Use ValidatorSet from pallet_cbc_pos if available, otherwise fallback to an empty vec
			// If not available, you may need to maintain such a list in storage
			// For now, let's assume pallet_cbc_pos::Pallet::<Runtime>::validator_set() exists
			#[cfg(feature = "std")] {
				// For std builds, you might want to use all accounts, but that's not efficient
			}
			#[cfg(not(feature = "std"))] {
				// Try to use a storage value if available
				// If not, return empty
			}
			// Try to use the DCF pallet's ValidatorSet if available
			pallet_cbc_dcf::Pallet::<Runtime>::validator_set().to_vec()
		}

		fn get_slashing_count(validator: AccountId) -> u32 {
			pallet_cbc_pos::Pallet::<Runtime>::slashing_count(&validator)
				.unwrap_or_default()
		}
	}

	// POI API
	impl pallet_cbc_poi::PoiApi<Block, AccountId> for Runtime {
		fn get_inference_result(validator: AccountId) -> Option<(u32, u32)> {
			pallet_cbc_poi::Pallet::<Runtime>::inference_results(&validator)
		}

		fn get_challenge(validator: AccountId) -> Option<(AccountId, u32, u32)> {
			pallet_cbc_poi::Pallet::<Runtime>::challenges(&validator)
		}

		fn get_current_epoch() -> u32 {
			pallet_cbc_poi::Pallet::<Runtime>::current_epoch()
		}
	}

	// DCF API
	impl pallet_cbc_dcf::DcfApi<Block, AccountId> for Runtime {
		fn get_validator_scores() -> Vec<(AccountId, u64)> {
			let validators = pallet_cbc_dcf::Pallet::<Runtime>::validator_set();
			validators
				.iter()
				.map(|validator| {
					let state = pallet_cbc_dcf::Pallet::<Runtime>::validator_states(validator);
					match state {
						Some(state) => (validator.clone(), state.current.final_score),
						None => (validator.clone(), 0),
					}
				})
				.collect()
		}

		fn get_current_epoch() -> u32 {
			pallet_cbc_dcf::Pallet::<Runtime>::current_epoch()
		}

		fn get_validator_stake_score(validator: AccountId) -> u64 {
			pallet_cbc_dcf::Pallet::<Runtime>::validator_states(&validator)
				.map(|state| state.current.stake_score)
				.unwrap_or_default()
		}

		fn get_validator_inference_score(validator: AccountId) -> u64 {
			pallet_cbc_dcf::Pallet::<Runtime>::validator_states(&validator)
				.map(|state| state.current.inference_score)
				.unwrap_or_default()
		}

		fn get_consensus_weights() -> (u64, u64) {
			(
				pallet_cbc_dcf::Pallet::<Runtime>::pos_weight(),
				pallet_cbc_dcf::Pallet::<Runtime>::poi_weight()
			)
		}

		fn is_validator_active(validator: AccountId) -> bool {
			pallet_cbc_dcf::Pallet::<Runtime>::is_validator_active(&validator)
		}

		fn get_expected_author(block_number: u32) -> Option<AccountId> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_expected_author(block_number)
		}

		fn get_validator_score_history(validator: AccountId) -> Vec<u64> {
			pallet_cbc_dcf::Pallet::<Runtime>::validator_states(&validator)
				.map(|state| state.history.iter().map(|stats| stats.final_score).collect())
				.unwrap_or_default()
		}

		fn get_validator_participation(validator: AccountId) -> (u32, u32) {
			pallet_cbc_dcf::Pallet::<Runtime>::validator_states(&validator)
				.map(|state| (state.current.authored_blocks, state.current.missed_blocks))
				.unwrap_or_default()
		}

		fn get_active_validators() -> Vec<AccountId> {
			pallet_cbc_dcf::Pallet::<Runtime>::active_validators().to_vec()
		}

		fn get_validator_last_active(validator: AccountId) -> u32 {
			pallet_cbc_dcf::Pallet::<Runtime>::validator_states(&validator)
				.map(|state| state.last_active_epoch)
				.unwrap_or_default()
		}

		fn validate_block_author(block_number: u32, author: AccountId) {
			pallet_cbc_dcf::Pallet::<Runtime>::validate_block_author(block_number, author)
		}

		fn get_validator_profile(validator: AccountId) -> Option<(u64, u32, u32, u32, u32)> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_validator_profile(validator)
		}

		fn get_inference_result(validator: AccountId) -> Option<u64> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_inference_result(validator)
		}

		fn get_epoch_history(epoch_number: u32) -> Option<pallet_cbc_dcf::RuntimeEpochHistory<AccountId>> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_epoch_history_api(epoch_number)
		}

		fn get_recent_epochs(n: u32) -> Vec<pallet_cbc_dcf::RuntimeEpochHistory<AccountId>> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_recent_epochs_api(n)
		}

		fn get_governance_mode() -> bool {
			pallet_cbc_dcf::Pallet::<Runtime>::get_governance_mode()
		}

		fn get_validator_consensus_contribution(validator: AccountId) -> Option<(u64, u64, u64)> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_validator_consensus_contribution(&validator)
		}

		fn get_epoch_config() -> pallet_cbc_dcf::EpochConfig {
			pallet_cbc_dcf::Pallet::<Runtime>::get_epoch_config()
		}

		fn get_validator_epoch_stats(validator: AccountId, epoch: u32) -> Option<pallet_cbc_dcf::EpochStats> {
			pallet_cbc_dcf::Pallet::<Runtime>::get_validator_epoch_stats(&validator, epoch)
		}

		fn get_total_validators_count() -> u32 {
			pallet_cbc_dcf::Pallet::<Runtime>::get_total_validators_count()
		}

		fn get_validator_set_info() -> (u32, u32, u32) {
			pallet_cbc_dcf::Pallet::<Runtime>::get_validator_set_info()
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
			crate::genesis_config_presets::get_preset(id)
		}
		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			crate::genesis_config_presets::preset_names()
		}
	}
}