#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_todo.
pub trait WeightInfo {
    fn create_todo() -> Weight;
    fn complete_todo() -> Weight;
    fn remove_todo() -> Weight;
    fn update_todo() -> Weight;
}

/// Weights for pallet_todo using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_todo() -> Weight {
        Weight::from_parts(25_000, 1000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn complete_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn remove_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn update_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

// Default implementation for tests and dev
impl WeightInfo for () {
    fn create_todo() -> Weight {
        Weight::from_parts(25_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn complete_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn remove_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn update_todo() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
