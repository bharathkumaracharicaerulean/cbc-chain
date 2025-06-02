#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::Get;
use sp_std::prelude::*;

benchmarks! {
    submit_inference {
        let caller: T::AccountId = whitelisted_caller();
        let result: u32 = 42;
        let confidence = T::MinInferenceConfidence::get() + 10;
    }: _(RawOrigin::Signed(caller.clone()), result, confidence)
    verify {
        assert!(InferenceResults::<T>::contains_key(&caller));
        let (stored_result, _) = InferenceResults::<T>::get(&caller).unwrap();
        assert_eq!(stored_result, result);
    }

    challenge_inference {
        let caller: T::AccountId = whitelisted_caller();
        let challenged: T::AccountId = account("challenged", 0, 0);
        let result: u32 = 42;
        let confidence = T::MinInferenceConfidence::get() + 10;

        // First submit an inference
        InferenceResults::<T>::insert(&challenged, (result, 0));
    }: _(RawOrigin::Signed(caller.clone()), challenged.clone(), result)
    verify {
        let (challenged_account, stored_result, _) = Challenges::<T>::get(&caller).unwrap();
        assert_eq!(challenged_account, challenged);
        assert_eq!(stored_result, result);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
} 