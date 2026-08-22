#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cbc_dvf.
pub trait WeightInfo {
    fn submit_dvf_vote() -> Weight;
    fn submit_justification() -> Weight;
    fn join_validators() -> Weight;
    fn leave_validators() -> Weight;
    fn cancel_leave_request() -> Weight;
}

/// Weights for pallet_cbc_dvf using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn submit_dvf_vote() -> Weight {
        Weight::from_parts(25_000, 1000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn submit_justification() -> Weight {
        Weight::from_parts(100_000, 4000)
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().writes(10))
    }

    fn join_validators() -> Weight {
        Weight::from_parts(80_000, 2500)
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(6))
    }

    fn leave_validators() -> Weight {
        Weight::from_parts(60_000, 2000)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn cancel_leave_request() -> Weight {
        Weight::from_parts(30_000, 1000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

// Default implementation for tests and dev
impl WeightInfo for () {
    fn submit_dvf_vote() -> Weight {
        Weight::from_parts(25_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn submit_justification() -> Weight {
        Weight::from_parts(100_000, 4000)
            .saturating_add(RocksDbWeight::get().reads(10))
            .saturating_add(RocksDbWeight::get().writes(10))
    }

    fn join_validators() -> Weight {
        Weight::from_parts(80_000, 2500)
            .saturating_add(RocksDbWeight::get().reads(7))
            .saturating_add(RocksDbWeight::get().writes(6))
    }

    fn leave_validators() -> Weight {
        Weight::from_parts(60_000, 2000)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn cancel_leave_request() -> Weight {
        Weight::from_parts(30_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}
