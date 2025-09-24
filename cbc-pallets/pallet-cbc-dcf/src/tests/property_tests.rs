//! Property and fuzz testing for DCF pallet
//!
//! This module implements comprehensive property-based testing to validate
//! DCF invariants under randomized conditions. It includes:
//! - Validator ordering properties (highest score always wins)
//! - Threshold filtering correctness
//! - Cooldown enforcement across all operations
//! - Randomized sequences of join/leave/reward/slash operations

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, assert_noop, traits::Get};
use sp_runtime::traits::{Zero, Saturating};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

/// Property test configuration
const MAX_VALIDATORS: u32 = 20;
const MAX_EPOCHS: u32 = 50;
const MAX_OPERATIONS_PER_EPOCH: u32 = 10;

/// Test seed for reproducible failures
pub const PROPERTY_TEST_SEED: u64 = 0x1234567890abcdef;

/// Validator operation types for randomized testing
#[derive(Debug, Clone)]
pub enum ValidatorOperation {
    Join { account: u64, stake: u128 },
    Leave { account: u64 },
    Reward { account: u64, amount: u128 },
    Slash { account: u64, amount: u128 },
    UpdateStakeScore { account: u64, score: u64 },
    UpdateInferenceScore { account: u64, score: u64 },
}

/// Generate random validator operations
fn arb_validator_operation() -> impl Strategy<Value = ValidatorOperation> {
    prop_oneof![
        (1u64..=MAX_VALIDATORS as u64, 1_000_000u128..=50_000_000u128)
            .prop_map(|(account, stake)| ValidatorOperation::Join { account, stake }),
        (1u64..=MAX_VALIDATORS as u64)
            .prop_map(|account| ValidatorOperation::Leave { account }),
        (1u64..=MAX_VALIDATORS as u64, 1_000u128..=10_000u128)
            .prop_map(|(account, amount)| ValidatorOperation::Reward { account, amount }),
        (1u64..=MAX_VALIDATORS as u64, 1_000u128..=5_000u128)
            .prop_map(|(account, amount)| ValidatorOperation::Slash { account, amount }),
        (1u64..=MAX_VALIDATORS as u64, 0u64..=10_000u64)
            .prop_map(|(account, score)| ValidatorOperation::UpdateStakeScore { account, score }),
        (1u64..=MAX_VALIDATORS as u64, 0u64..=10_000u64)
            .prop_map(|(account, score)| ValidatorOperation::UpdateInferenceScore { account, score }),
    ]
}

/// Generate sequence of validator operations
fn arb_operation_sequence() -> impl Strategy<Value = Vec<ValidatorOperation>> {
    prop::collection::vec(arb_validator_operation(), 0..MAX_OPERATIONS_PER_EPOCH as usize)
}

/// Property test: Validator ordering invariant
/// Highest scoring validators should always be selected for active set
#[test]
fn prop_validator_ordering_invariant() {
    let mut runner = proptest::test_runner::TestRunner::new(
        proptest::test_runner::Config::with_source_file(file!())
            .with_seed(PROPERTY_TEST_SEED)
    );

    runner.run(&arb_operation_sequence(), |operations| {
        new_test_ext().execute_with(|| {
            // Initialize with some validators
            let initial_validators = vec![1, 2, 3, 4, 5];
            for &validator in &initial_validators {
                let _ = Balances::make_free_balance_be(&validator, 100_000_000);
                assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(validator)));
            }

            // Apply random operations
            for operation in operations {
                apply_operation(operation);
            }

            // Check ordering invariant
            let active_validators = DcfModule::active_validators();
            let validator_scores: Vec<_> = active_validators.iter()
                .map(|v| (*v, DcfModule::get_validator_final_score(*v)))
                .collect();

            // Verify scores are in descending order
            for i in 1..validator_scores.len() {
                prop_assert!(
                    validator_scores[i-1].1 >= validator_scores[i].1,
                    "Validator ordering violated: {} (score {}) should not come before {} (score {})",
                    validator_scores[i-1].0, validator_scores[i-1].1,
                    validator_scores[i].0, validator_scores[i].1
                );
            }

            // Verify active set contains highest scoring validators
            let all_validators = DcfModule::validator_set();
            let all_scores: Vec<_> = all_validators.iter()
                .map(|v| (*v, DcfModule::get_validator_final_score(*v)))
                .collect();
            
            let mut sorted_scores = all_scores.clone();
            sorted_scores.sort_by(|a, b| b.1.cmp(&a.1));

            let max_active = <Test as Config>::MaxValidators::get() as usize;
            let expected_active_count = std::cmp::min(all_validators.len(), max_active);
            
            for i in 0..expected_active_count {
                let expected_validator = sorted_scores[i].0;
                prop_assert!(
                    active_validators.contains(&expected_validator),
                    "Highest scoring validator {} (score {}) not in active set",
                    expected_validator, sorted_scores[i].1
                );
            }

            Ok(())
        })
    }).unwrap();
}

/// Property test: Threshold filtering correctness
/// Validators below minimum score threshold should be filtered out
#[test]
fn prop_threshold_filtering_invariant() {
    let mut runner = proptest::test_runner::TestRunner::new(
        proptest::test_runner::Config::with_source_file(file!())
            .with_seed(PROPERTY_TEST_SEED)
    );

    runner.run(&arb_operation_sequence(), |operations| {
        new_test_ext().execute_with(|| {
            // Initialize validators with varying scores
            for i in 1..=10 {
                let _ = Balances::make_free_balance_be(&i, 100_000_000);
                assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(i)));
                
                // Set varying scores
                let stake_score = (i as u64) * 1000;
                let inference_score = (i as u64) * 500;
                assert_ok!(DcfModule::update_validator_stake_score(
                    RuntimeOrigin::root(), i, stake_score
                ));
                assert_ok!(DcfModule::update_validator_inference_score(
                    RuntimeOrigin::root(), i, inference_score
                ));
            }

            // Apply random operations
            for operation in operations {
                apply_operation(operation);
            }

            // Check threshold filtering
            let active_validators = DcfModule::active_validators();
            let min_score = <Test as Config>::MinValidatorScore::get();

            for &validator in &active_validators {
                let final_score = DcfModule::get_validator_final_score(validator);
                prop_assert!(
                    final_score >= min_score,
                    "Active validator {} has score {} below minimum threshold {}",
                    validator, final_score, min_score
                );
            }

            // Verify high-scoring validators are not excluded
            let all_validators = DcfModule::validator_set();
            for &validator in &all_validators {
                let final_score = DcfModule::get_validator_final_score(validator);
                if final_score >= min_score && !DcfModule::is_in_cooldown(validator) {
                    let active_count = active_validators.len();
                    let max_validators = <Test as Config>::MaxValidators::get() as usize;
                    
                    if active_count < max_validators {
                        prop_assert!(
                            active_validators.contains(&validator),
                            "Validator {} with score {} above threshold should be active",
                            validator, final_score
                        );
                    }
                }
            }

            Ok(())
        })
    }).unwrap();
}

/// Property test: Cooldown enforcement across all operations
/// Validators in cooldown should not be able to perform restricted operations
#[test]
fn prop_cooldown_enforcement_invariant() {
    let mut runner = proptest::test_runner::TestRunner::new(
        proptest::test_runner::Config::with_source_file(file!())
            .with_seed(PROPERTY_TEST_SEED)
    );

    runner.run(&arb_operation_sequence(), |operations| {
        new_test_ext().execute_with(|| {
            let mut cooldown_validators = HashSet::new();
            
            // Initialize some validators
            for i in 1..=5 {
                let _ = Balances::make_free_balance_be(&i, 100_000_000);
                assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(i)));
            }

            // Track cooldown state and verify enforcement
            for operation in operations {
                match operation {
                    ValidatorOperation::Leave { account } => {
                        if DcfModule::validator_set().contains(&account) {
                            assert_ok!(DcfModule::leave_validators(RuntimeOrigin::signed(account)));
                            cooldown_validators.insert(account);
                        }
                    },
                    ValidatorOperation::Join { account, stake } => {
                        let _ = Balances::make_free_balance_be(&account, stake);
                        
                        if cooldown_validators.contains(&account) {
                            // Should fail if in cooldown
                            assert_noop!(
                                DcfModule::join_validators(RuntimeOrigin::signed(account)),
                                Error::<Test>::ValidatorInCooldown
                            );
                        } else if !DcfModule::validator_set().contains(&account) {
                            // Should succeed if not in cooldown
                            assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(account)));
                        }
                    },
                    _ => {
                        // Apply other operations normally
                        apply_operation(operation);
                    }
                }

                // Verify cooldown validators are not in active set
                let active_validators = DcfModule::active_validators();
                for &cooldown_validator in &cooldown_validators {
                    prop_assert!(
                        !active_validators.contains(&cooldown_validator) || 
                        !DcfModule::is_in_cooldown(cooldown_validator),
                        "Validator {} in cooldown should not be in active set",
                        cooldown_validator
                    );
                }
            }

            Ok(())
        })
    }).unwrap();
}

/// Property test: Multi-epoch invariant preservation
/// System invariants should hold across multiple epochs with random operations
#[test]
fn prop_multi_epoch_invariant_preservation() {
    let mut runner = proptest::test_runner::TestRunner::new(
        proptest::test_runner::Config::with_source_file(file!())
            .with_seed(PROPERTY_TEST_SEED)
    );

    let strategy = (1u32..=10u32, prop::collection::vec(arb_operation_sequence(), 1..10));
    
    runner.run(&strategy, |(num_epochs, epoch_operations)| {
        new_test_ext().execute_with(|| {
            // Initialize system
            for i in 1..=5 {
                let _ = Balances::make_free_balance_be(&i, 100_000_000);
                assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(i)));
            }

            for epoch in 0..num_epochs {
                let operations = &epoch_operations[epoch as usize % epoch_operations.len()];
                
                // Apply operations for this epoch
                for operation in operations {
                    apply_operation(operation.clone());
                }

                // Advance epoch
                run_to_block(((epoch + 1) * 100) + 1);

                // Check invariants after epoch transition
                check_system_invariants()?;
            }

            Ok(())
        })
    }).unwrap();
}

/// Apply a validator operation with proper error handling
fn apply_operation(operation: ValidatorOperation) {
    match operation {
        ValidatorOperation::Join { account, stake } => {
            let _ = Balances::make_free_balance_be(&account, stake);
            let _ = DcfModule::join_validators(RuntimeOrigin::signed(account));
        },
        ValidatorOperation::Leave { account } => {
            if DcfModule::validator_set().contains(&account) {
                let _ = DcfModule::leave_validators(RuntimeOrigin::signed(account));
            }
        },
        ValidatorOperation::Reward { account, amount } => {
            if DcfModule::validator_set().contains(&account) {
                let _ = DcfModule::propose_reward_validator(
                    RuntimeOrigin::root(), account, amount
                );
            }
        },
        ValidatorOperation::Slash { account, amount } => {
            if DcfModule::validator_set().contains(&account) {
                let _ = DcfModule::slash_validator(
                    RuntimeOrigin::root(), account, amount
                );
            }
        },
        ValidatorOperation::UpdateStakeScore { account, score } => {
            if DcfModule::validator_set().contains(&account) {
                let _ = DcfModule::update_validator_stake_score(
                    RuntimeOrigin::root(), account, score
                );
            }
        },
        ValidatorOperation::UpdateInferenceScore { account, score } => {
            if DcfModule::validator_set().contains(&account) {
                let _ = DcfModule::update_validator_inference_score(
                    RuntimeOrigin::root(), account, score
                );
            }
        },
    }
}

/// Check system invariants and return error if any are violated
fn check_system_invariants() -> Result<(), proptest::test_runner::TestCaseError> {
    // Economic invariants
    let active_validators = DcfModule::active_validators();
    for &validator in &active_validators {
        let reserved = Balances::reserved_balance(&validator);
        let min_stake = <Test as Config>::MinStake::get();
        
        prop_assert!(
            reserved >= min_stake,
            "Validator {} has insufficient reserved balance: {} < {}",
            validator, reserved, min_stake
        );
    }

    // Validator set size invariants
    let max_validators = <Test as Config>::MaxValidators::get() as usize;
    prop_assert!(
        active_validators.len() <= max_validators,
        "Active validator set size {} exceeds maximum {}",
        active_validators.len(), max_validators
    );

    // Score invariants
    let max_score = <Test as Config>::MaxValidatorScore::get();
    for &validator in &active_validators {
        let final_score = DcfModule::get_validator_final_score(validator);
        prop_assert!(
            final_score <= max_score,
            "Validator {} score {} exceeds maximum {}",
            validator, final_score, max_score
        );
    }

    // Cooldown invariants
    for &validator in &active_validators {
        prop_assert!(
            !DcfModule::is_in_cooldown(validator),
            "Active validator {} should not be in cooldown",
            validator
        );
    }

    Ok(())
}

/// Fuzz test with committed seeds for reproducible failures
#[test]
fn fuzz_test_with_committed_seeds() {
    // Test with known problematic seeds
    let problematic_seeds = vec![
        0x1234567890abcdef,
        0xdeadbeefcafebabe,
        0x0123456789abcdef,
        0xfedcba9876543210,
    ];

    for &seed in &problematic_seeds {
        let mut runner = proptest::test_runner::TestRunner::new(
            proptest::test_runner::Config::with_source_file(file!())
                .with_seed(seed)
        );

        runner.run(&arb_operation_sequence(), |operations| {
            new_test_ext().execute_with(|| {
                // Initialize system
                for i in 1..=3 {
                    let _ = Balances::make_free_balance_be(&i, 100_000_000);
                    assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(i)));
                }

                // Apply operations
                for operation in operations {
                    apply_operation(operation);
                }

                // Check invariants
                check_system_invariants()?;

                Ok(())
            })
        }).unwrap_or_else(|e| {
            panic!("Fuzz test failed with seed {:#x}: {}", seed, e);
        });
    }
}

/// Property test: Validator score consistency
/// Scores should be computed consistently and deterministically
#[test]
fn prop_score_consistency() {
    let mut runner = proptest::test_runner::TestRunner::new(
        proptest::test_runner::Config::with_source_file(file!())
            .with_seed(PROPERTY_TEST_SEED)
    );

    let strategy = (
        1u64..=10u64,  // validator account
        0u64..=10000u64,  // stake score
        0u64..=10000u64,  // inference score
    );

    runner.run(&strategy, |(validator, stake_score, inference_score)| {
        new_test_ext().execute_with(|| {
            // Initialize validator
            let _ = Balances::make_free_balance_be(&validator, 100_000_000);
            assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(validator)));

            // Set scores
            assert_ok!(DcfModule::update_validator_stake_score(
                RuntimeOrigin::root(), validator, stake_score
            ));
            assert_ok!(DcfModule::update_validator_inference_score(
                RuntimeOrigin::root(), validator, inference_score
            ));

            // Get final score multiple times
            let score1 = DcfModule::get_validator_final_score(validator);
            let score2 = DcfModule::get_validator_final_score(validator);
            let score3 = DcfModule::get_validator_final_score(validator);

            // Scores should be identical (deterministic)
            prop_assert_eq!(score1, score2, "Score calculation not deterministic");
            prop_assert_eq!(score2, score3, "Score calculation not deterministic");

            // Score should be within expected bounds
            let max_score = <Test as Config>::MaxValidatorScore::get();
            prop_assert!(
                score1 <= max_score,
                "Final score {} exceeds maximum {}",
                score1, max_score
            );

            Ok(())
        })
    }).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_test_seed_reproducibility() {
        // Verify that the same seed produces the same results
        let mut results1 = Vec::new();
        let mut results2 = Vec::new();

        for _ in 0..2 {
            let mut runner = proptest::test_runner::TestRunner::new(
                proptest::test_runner::Config::with_source_file(file!())
                    .with_seed(PROPERTY_TEST_SEED)
                    .with_cases(5)
            );

            let mut current_results = Vec::new();
            runner.run(&arb_operation_sequence(), |operations| {
                current_results.push(operations.len());
                Ok(())
            }).unwrap();

            if results1.is_empty() {
                results1 = current_results;
            } else {
                results2 = current_results;
            }
        }

        assert_eq!(results1, results2, "Property test seed should be reproducible");
    }
}