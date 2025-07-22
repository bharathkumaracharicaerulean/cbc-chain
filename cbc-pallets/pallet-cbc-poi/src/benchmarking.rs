#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking file for pallet-cbc-poi.
//! Provides benchmarks for the main extrinsics: submit_inference and challenge_inference.

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::Get;
use sp_std::prelude::*;

// Define benchmarks for the PoI pallet.
benchmarks! {
    // Benchmark for the `submit_inference` extrinsic.
    submit_inference {
        // Set up a whitelisted caller (benchmarking utility).
        let caller: T::AccountId = whitelisted_caller();
        // Example inference result value.
        let result: u32 = 42;
        // Confidence above the minimum threshold.
        let confidence = T::MinInferenceConfidence::get() + 10;
    }: _(RawOrigin::Signed(caller.clone()), result, confidence)
    verify {
        // Verify that the inference result was stored for the caller.
        assert!(InferenceResults::<T>::contains_key(&caller));
        let (stored_result, _) = InferenceResults::<T>::get(&caller).unwrap();
        assert_eq!(stored_result, result);
    }

    // Benchmark for the `challenge_inference` extrinsic.
    challenge_inference {
        // Set up a whitelisted caller and a challenged account.
        let caller: T::AccountId = whitelisted_caller();
        let challenged: T::AccountId = account("challenged", 0, 0);
        let result: u32 = 42;
        let confidence = T::MinInferenceConfidence::get() + 10;

        // Pre-insert an inference result for the challenged account.
        InferenceResults::<T>::insert(&challenged, (result, 0));
    }: _(RawOrigin::Signed(caller.clone()), challenged.clone(), result)
    verify {
        // Verify that the challenge was stored for the caller.
        let (challenged_account, stored_result, _) = Challenges::<T>::get(&caller).unwrap();
        assert_eq!(challenged_account, challenged);
        assert_eq!(stored_result, result);
    }

    // Macro to generate the full benchmark test suite for this pallet.
    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}