#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{Weight, constants::RocksDbWeight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for the pallet.
pub trait WeightInfo {
    fn on_initialize() -> Weight;
    fn offchain_worker() -> Weight;
    fn update_validator_stake_score() -> Weight;
    fn update_validator_inference_score() -> Weight;
    fn update_consensus_weights() -> Weight;
}

/// Weights for the pallet using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn on_initialize() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn offchain_worker() -> Weight {
        Weight::from_parts(50_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn update_validator_stake_score() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn update_validator_inference_score() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn update_consensus_weights() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn on_initialize() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn offchain_worker() -> Weight {
        Weight::from_parts(50_000, 0)
    }

    fn update_validator_stake_score() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn update_validator_inference_score() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn update_consensus_weights() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}