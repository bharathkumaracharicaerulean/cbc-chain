#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cbc_pos.
pub trait WeightInfo {
    fn register_validator() -> Weight;
    fn submit_score() -> Weight;
    fn slash_validator() -> Weight;
    fn boost_score() -> Weight;
    fn update_epoch() -> Weight;
}

/// Weights for pallet_cbc_pos using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_validator() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(Weight::from_parts(0, 500)) // db write
    }

    fn submit_score() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(Weight::from_parts(0, 1_000)) // db write
    }

    fn slash_validator() -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(0, 1_500)) // db write
    }

    fn boost_score() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(Weight::from_parts(0, 1_000)) // db write
    }

    fn update_epoch() -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(0, 1_000)) // db write
    }
}

/// Default implementation for testing and development.
impl WeightInfo for () {
    fn register_validator() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn submit_score() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn slash_validator() -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn boost_score() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn update_epoch() -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }
}