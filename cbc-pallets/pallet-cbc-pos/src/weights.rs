use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};

/// Weight functions needed for pallet_cbc_pos.
pub trait WeightInfo {
    fn register_validator() -> Weight;
    fn submit_score() -> Weight;
    fn slash_validator() -> Weight;
    fn bond_stake() -> Weight;
    fn unbond_stake() -> Weight;
    fn boost_score() -> Weight;
    fn slash_score() -> Weight;
    fn increase_validator_stake() -> Weight;
    fn decrease_validator_stake() -> Weight;
    fn slash_validator_percentage() -> Weight;
    fn slash_multiple_validators() -> Weight;
    fn reward_validator_call() -> Weight;
    fn reward_multiple_validators() -> Weight;
    fn reward_all_active_validators() -> Weight;
    fn distribute_epoch_rewards() -> Weight;
}

/// Weights for pallet_cbc_pos using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_validator() -> Weight {
        Weight::from_parts(30_000, 1000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn submit_score() -> Weight {
        Weight::from_parts(15_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn slash_validator() -> Weight {
        Weight::from_parts(40_000, 1500)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn bond_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn unbond_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn boost_score() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn slash_score() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn increase_validator_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn decrease_validator_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn slash_validator_percentage() -> Weight {
        Weight::from_parts(40_000, 1500)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn slash_multiple_validators() -> Weight {
        Weight::from_parts(100_000, 4000)
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().writes(10))
    }

    fn reward_validator_call() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn reward_multiple_validators() -> Weight {
        Weight::from_parts(80_000, 3000)
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().writes(8))
    }

    fn reward_all_active_validators() -> Weight {
        Weight::from_parts(120_000, 5000)
            .saturating_add(T::DbWeight::get().reads(12))
            .saturating_add(T::DbWeight::get().writes(12))
    }

    fn distribute_epoch_rewards() -> Weight {
        Weight::from_parts(150_000, 6000)
            .saturating_add(T::DbWeight::get().reads(15))
            .saturating_add(T::DbWeight::get().writes(15))
    }
}

/// Default implementation for testing and development.
impl WeightInfo for () {
    fn register_validator() -> Weight {
        Weight::from_parts(30_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn submit_score() -> Weight {
        Weight::from_parts(15_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn slash_validator() -> Weight {
        Weight::from_parts(40_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn bond_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn unbond_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn boost_score() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn slash_score() -> Weight {
        Weight::from_parts(20_000, 1000)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn increase_validator_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn decrease_validator_stake() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn slash_validator_percentage() -> Weight {
        Weight::from_parts(40_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn slash_multiple_validators() -> Weight {
        Weight::from_parts(100_000, 4000)
            .saturating_add(RocksDbWeight::get().reads(10))
            .saturating_add(RocksDbWeight::get().writes(10))
    }

    fn reward_validator_call() -> Weight {
        Weight::from_parts(35_000, 1500)
            .saturating_add(RocksDbWeight::get().reads(2))
            .saturating_add(RocksDbWeight::get().writes(2))
    }

    fn reward_multiple_validators() -> Weight {
        Weight::from_parts(80_000, 3000)
            .saturating_add(RocksDbWeight::get().reads(8))
            .saturating_add(RocksDbWeight::get().writes(8))
    }

    fn reward_all_active_validators() -> Weight {
        Weight::from_parts(120_000, 5000)
            .saturating_add(RocksDbWeight::get().reads(12))
            .saturating_add(RocksDbWeight::get().writes(12))
    }

    fn distribute_epoch_rewards() -> Weight {
        Weight::from_parts(150_000, 6000)
            .saturating_add(RocksDbWeight::get().reads(15))
            .saturating_add(RocksDbWeight::get().writes(15))
    }
}