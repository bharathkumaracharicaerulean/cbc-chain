#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cbc_poi.
pub trait WeightInfo {
	fn submit_inference() -> Weight;
	fn challenge_inference() -> Weight;
	fn resolve_challenge() -> Weight;
	fn epoch_cleanup() -> Weight;
	fn simulate_inference() -> Weight;
	fn apply_offchain_poi_scores() -> Weight;
	fn update_validator_inference_score() -> Weight;
}

/// Weights for pallet_cbc_poi using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Weight for `submit_inference`.
	fn submit_inference() -> Weight {
		Weight::from_parts(25_000, 1000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}

	/// Weight for `challenge_inference`.
	fn challenge_inference() -> Weight {
		Weight::from_parts(35_000, 1500)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}

	/// Weight for `resolve_challenge`.
	fn resolve_challenge() -> Weight {
		Weight::from_parts(45_000, 2000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}

	/// Weight for `epoch_cleanup`.
	fn epoch_cleanup() -> Weight {
		Weight::from_parts(50_000, 2500)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(5))
	}

	/// Weight for `simulate_inference`.
	fn simulate_inference() -> Weight {
		Weight::from_parts(30_000, 1000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}

	/// Weight for `apply_offchain_poi_scores`.
	fn apply_offchain_poi_scores() -> Weight {
		Weight::from_parts(60_000, 3000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}

	/// Weight for `update_validator_inference_score`.
	fn update_validator_inference_score() -> Weight {
		Weight::from_parts(20_000, 1000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn submit_inference() -> Weight {
		Weight::from_parts(25_000, 1000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}

	fn challenge_inference() -> Weight {
		Weight::from_parts(35_000, 1500)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}

	fn resolve_challenge() -> Weight {
		Weight::from_parts(45_000, 2000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(3))
	}

	fn epoch_cleanup() -> Weight {
		Weight::from_parts(50_000, 2500)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(5))
	}

	fn simulate_inference() -> Weight {
		Weight::from_parts(30_000, 1000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}

	fn apply_offchain_poi_scores() -> Weight {
		Weight::from_parts(60_000, 3000)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(5))
	}

	fn update_validator_inference_score() -> Weight {
		Weight::from_parts(20_000, 1000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
