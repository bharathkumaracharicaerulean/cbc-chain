#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::Get;
use sp_std::prelude::*;

benchmarks! {
    register_validator {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller.clone()))
    verify {
        assert!(Validators::<T>::contains_key(&caller));
    }

    submit_score {
        let caller: T::AccountId = whitelisted_caller();
        let validator: T::AccountId = account("validator", 0, 0);
        
        // Register the validator first
        Validators::<T>::insert(&validator, true);
        
        let score = T::MinValidatorScore::get() + 10;
    }: _(RawOrigin::Signed(caller), validator.clone(), score)
    verify {
        assert_eq!(ValidatorScores::<T>::get(&validator), Some(score));
    }

    slash_validator {
        let caller: T::AccountId = whitelisted_caller();
        let validator: T::AccountId = account("validator", 0, 0);
        
        // Register the validator first
        Validators::<T>::insert(&validator, true);
        
        // Set initial slashing count
        let initial_count = T::MaxSlashingCount::get() - 1;
        SlashingCount::<T>::insert(&validator, initial_count);
    }: _(RawOrigin::Signed(caller), validator.clone())
    verify {
        // Verify validator was removed after reaching max slashing count
        assert!(!Validators::<T>::contains_key(&validator));
        assert_eq!(ValidatorScores::<T>::get(&validator), None);
        assert_eq!(SlashingCount::<T>::get(&validator), None);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
} 