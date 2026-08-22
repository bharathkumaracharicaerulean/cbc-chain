#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cbc_governance.
pub trait WeightInfo {
    fn set_governance_mode() -> Weight;
    fn submit_proposal() -> Weight;
    fn vote_proposal() -> Weight;
    fn execute_proposal() -> Weight;
    fn propose_slash_validator() -> Weight;
    fn propose_reward_validator() -> Weight;
    fn propose_default_reward_validator() -> Weight;
    fn propose_reward_multiple_validators() -> Weight;
    fn propose_default_reward_multiple_validators() -> Weight;
    fn propose_reward_all_active_validators() -> Weight;
    fn propose_eject_validator() -> Weight;
}

/// Weights for pallet_cbc_governance using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_governance_mode() -> Weight {
        Weight::from_parts(15_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn submit_proposal() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn vote_proposal() -> Weight {
        Weight::from_parts(30_000, 1200)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn execute_proposal() -> Weight {
        Weight::from_parts(50_000, 2000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_slash_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_reward_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_default_reward_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_reward_multiple_validators() -> Weight {
        Weight::from_parts(45_000, 2000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_default_reward_multiple_validators() -> Weight {
        Weight::from_parts(45_000, 2000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_reward_all_active_validators() -> Weight {
        Weight::from_parts(55_000, 2500)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_eject_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

// Default implementation for tests and mock environments
impl WeightInfo for () {
    fn set_governance_mode() -> Weight {
        Weight::from_parts(15_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn submit_proposal() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn vote_proposal() -> Weight {
        Weight::from_parts(30_000, 1200)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn execute_proposal() -> Weight {
        Weight::from_parts(50_000, 2000)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_slash_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_reward_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_default_reward_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_reward_multiple_validators() -> Weight {
        Weight::from_parts(45_000, 2000)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_default_reward_multiple_validators() -> Weight {
        Weight::from_parts(45_000, 2000)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_reward_all_active_validators() -> Weight {
        Weight::from_parts(55_000, 2500)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn propose_eject_validator() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(2))
    }
}
