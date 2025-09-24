//! Unit tests for robust slashing and reward accounting with overflow protection.
//!
//! This module contains comprehensive tests for the enhanced slashing and reward
//! accounting system, including overflow protection, bounds enforcement, and
//! enhanced event emission.

use super::*;
use crate::mock::*;
use frame_support::{
    assert_err, assert_ok,
    traits::ReservableCurrency,
};

type DcfModule = DcfPallet;
type AccountId = <Test as frame_system::Config>::AccountId;
type Balance = <Test as pallet_balances::Config>::Balance;

/// Test helper to setup a validator with stake
fn setup_validator_with_stake(validator: &AccountId, stake: Balance) {
    // Reserve the stake
    let _ = Balances::reserve(validator, stake);
    ValidatorStake::<Test>::insert(validator, stake);
    
    // Initialize validator state
    let validator_state = ValidatorState {
        last_active_epoch: 0,
        current: EpochStats {
            epoch: 0,
            stake_score: stake as u64,
            inference_score: 100,
            final_score: 1000,
            authored_blocks: 0,
            missed_blocks: 0,
        },
        history: BoundedVec::default(),
        uptime: 1,
        inference_success_count: 10,
        participation_rate: 100,
        inference_count: 10,
        last_active_block: 1,
        name: None,
        trust_score: 1000,
    };
    ValidatorStates::<Test>::insert(validator, validator_state);
    
    // Add to validator set
    let mut validator_set = ValidatorSet::<Test>::get();
    if !validator_set.contains(validator) {
        let _ = validator_set.try_push(*validator);
        ValidatorSet::<Test>::put(validator_set);
    }
    
    // Add to active validators
    let mut active_validators = ActiveValidators::<Test>::get();
    if !active_validators.contains(validator) {
        let _ = active_validators.try_push(*validator);
        ActiveValidators::<Test>::put(active_validators);
    }
}

#[test]
fn test_slash_validator_with_bounds_checking() {
    new_test_ext().execute_with(|| {
        let validator = 10u64; // Use different ID to avoid genesis conflicts
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        let pre_balance = Balances::free_balance(&validator);
        
        // Test normal slashing within bounds
        let slash_amount = 1_000u128;
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator, 
            slash_amount, 
            SlashReason::PoorPerformance
        ));
        
        // Check that epoch tracking was updated
        assert_eq!(EpochTotalSlashed::<Test>::get(), slash_amount);
        assert_eq!(ValidatorEpochSlashed::<Test>::get(&validator), slash_amount);
        
        // Check event was emitted with correct data
        let events = System::events();
        let slash_event = events.iter().find(|e| {
            matches!(e.event, RuntimeEvent::DcfPallet(Event::ValidatorSlashed { .. }))
        }).expect("ValidatorSlashed event should be emitted");
        
        if let RuntimeEvent::DcfPallet(Event::ValidatorSlashed { 
            validator: event_validator, 
            amount, 
            pre_balance: event_pre_balance,
            post_balance: event_post_balance,
            reason 
        }) = &slash_event.event {
            assert_eq!(*event_validator, validator);
            assert_eq!(*amount, slash_amount);
            assert_eq!(*event_pre_balance, pre_balance);
            assert!(event_post_balance < event_pre_balance);
            assert_eq!(*reason, SlashReason::PoorPerformance);
        }
    });
}

#[test]
fn test_slash_validator_exceeds_epoch_bounds() {
    new_test_ext().execute_with(|| {
        let validator = 11u64;
        let initial_stake = 100_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Try to slash more than the epoch limit
        let excessive_slash = 50001u128;
        assert_err!(
            DcfModule::execute_slash_validator_with_reason(
                &validator, 
                excessive_slash, 
                SlashReason::Misbehavior
            ),
            Error::<Test>::SlashingBoundsExceeded
        );
        
        // Check that no slashing occurred
        assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
        assert_eq!(ValidatorEpochSlashed::<Test>::get(&validator), 0);
    });
}

#[test]
fn test_slash_validator_exceeds_per_validator_bounds() {
    new_test_ext().execute_with(|| {
        let validator = 12u64;
        let initial_stake = 100_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Try to slash more than the per-validator limit
        let excessive_slash = 20001u128;
        assert_err!(
            DcfModule::execute_slash_validator_with_reason(
                &validator, 
                excessive_slash, 
                SlashReason::Misbehavior
            ),
            Error::<Test>::SlashingBoundsExceeded
        );
        
        // Check that no slashing occurred
        assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
        assert_eq!(ValidatorEpochSlashed::<Test>::get(&validator), 0);
    });
}

#[test]
fn test_reward_validator_with_bounds_checking() {
    new_test_ext().execute_with(|| {
        let validator = 13u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        let pre_balance = Balances::free_balance(&validator);
        
        // Test normal reward within bounds
        let reward_amount = 1_000u128;
        assert_ok!(DcfModule::execute_reward_validator_with_reason(
            &validator, 
            reward_amount, 
            RewardReason::ExceptionalPerformance
        ));
        
        // Check that epoch tracking was updated
        assert_eq!(EpochTotalRewarded::<Test>::get(), reward_amount);
        assert_eq!(ValidatorEpochRewarded::<Test>::get(&validator), reward_amount);
        
        // Check event was emitted with correct data
        let events = System::events();
        let reward_event = events.iter().find(|e| {
            matches!(e.event, RuntimeEvent::DcfPallet(Event::ValidatorRewarded { .. }))
        }).expect("ValidatorRewarded event should be emitted");
        
        if let RuntimeEvent::DcfPallet(Event::ValidatorRewarded { 
            validator: event_validator, 
            amount, 
            pre_balance: event_pre_balance,
            post_balance: event_post_balance,
            reason 
        }) = &reward_event.event {
            assert_eq!(*event_validator, validator);
            assert_eq!(*amount, reward_amount);
            assert_eq!(*event_pre_balance, pre_balance);
            assert!(event_post_balance > event_pre_balance);
            assert_eq!(*reason, RewardReason::ExceptionalPerformance);
        }
    });
}

#[test]
fn test_reward_validator_exceeds_epoch_bounds() {
    new_test_ext().execute_with(|| {
        let validator = 14u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Try to reward more than the epoch limit
        let excessive_reward = 30001u128;
        assert_err!(
            DcfModule::execute_reward_validator_with_reason(
                &validator, 
                excessive_reward, 
                RewardReason::ManualReward
            ),
            Error::<Test>::RewardBoundsExceeded
        );
        
        // Check that no reward occurred
        assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
        assert_eq!(ValidatorEpochRewarded::<Test>::get(&validator), 0);
    });
}

#[test]
fn test_reward_validator_exceeds_per_validator_bounds() {
    new_test_ext().execute_with(|| {
        let validator = 15u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Try to reward more than the per-validator limit
        let excessive_reward = 10001u128;
        assert_err!(
            DcfModule::execute_reward_validator_with_reason(
                &validator, 
                excessive_reward, 
                RewardReason::ManualReward
            ),
            Error::<Test>::RewardBoundsExceeded
        );
        
        // Check that no reward occurred
        assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
        assert_eq!(ValidatorEpochRewarded::<Test>::get(&validator), 0);
    });
}

#[test]
fn test_arithmetic_overflow_protection_in_slashing() {
    new_test_ext().execute_with(|| {
        let validator = 16u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Set up a scenario where arithmetic would overflow
        // First, slash up to near the limit
        let near_limit_slash = 49999u128;
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator, 
            near_limit_slash, 
            SlashReason::PoorPerformance
        ));
        
        // Now try to slash an amount that would cause overflow when added
        let overflow_slash = 10u128; // This would exceed the limit
        assert_err!(
            DcfModule::execute_slash_validator_with_reason(
                &validator, 
                overflow_slash, 
                SlashReason::PoorPerformance
            ),
            Error::<Test>::SlashingBoundsExceeded
        );
    });
}

#[test]
fn test_arithmetic_overflow_protection_in_rewards() {
    new_test_ext().execute_with(|| {
        let validator = 17u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Set up a scenario where arithmetic would overflow
        // First, reward up to near the limit
        let near_limit_reward = 29999u128;
        assert_ok!(DcfModule::execute_reward_validator_with_reason(
            &validator, 
            near_limit_reward, 
            RewardReason::ExceptionalPerformance
        ));
        
        // Now try to reward an amount that would cause overflow when added
        let overflow_reward = 10u128; // This would exceed the limit
        assert_err!(
            DcfModule::execute_reward_validator_with_reason(
                &validator, 
                overflow_reward, 
                RewardReason::ExceptionalPerformance
            ),
            Error::<Test>::RewardBoundsExceeded
        );
    });
}

#[test]
fn test_epoch_bounds_reset_on_transition() {
    new_test_ext().execute_with(|| {
        let validator = 18u64;
        let initial_stake = 10_000u128;
        
        // Setup validator
        setup_validator_with_stake(&validator, initial_stake);
        
        // Perform some slashing and rewarding
        let slash_amount = 1_000u128;
        let reward_amount = 500u128;
        
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator, 
            slash_amount, 
            SlashReason::PoorPerformance
        ));
        
        assert_ok!(DcfModule::execute_reward_validator_with_reason(
            &validator, 
            reward_amount, 
            RewardReason::ExceptionalPerformance
        ));
        
        // Check that tracking is updated
        assert_eq!(EpochTotalSlashed::<Test>::get(), slash_amount);
        assert_eq!(EpochTotalRewarded::<Test>::get(), reward_amount);
        assert_eq!(ValidatorEpochSlashed::<Test>::get(&validator), slash_amount);
        assert_eq!(ValidatorEpochRewarded::<Test>::get(&validator), reward_amount);
        
        // Trigger epoch transition
        let _ = DcfModule::handle_epoch_transition();
        
        // Check that epoch bounds were reset
        assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
        assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
        assert_eq!(ValidatorEpochSlashed::<Test>::get(&validator), 0);
        assert_eq!(ValidatorEpochRewarded::<Test>::get(&validator), 0);
    });
}

#[test]
fn test_multiple_validators_epoch_bounds() {
    new_test_ext().execute_with(|| {
        let validator1 = 19u64;
        let validator2 = 20u64;
        let initial_stake = 10_000u128;
        
        // Setup validators
        setup_validator_with_stake(&validator1, initial_stake);
        setup_validator_with_stake(&validator2, initial_stake);
        
        // Slash both validators
        let slash_amount = 50000u128 / 3; // Each gets 1/3 of limit
        
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator1, 
            slash_amount, 
            SlashReason::PoorPerformance
        ));
        
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator2, 
            slash_amount, 
            SlashReason::PoorPerformance
        ));
        
        // Check epoch total
        assert_eq!(EpochTotalSlashed::<Test>::get(), slash_amount * 2);
        
        // Try to slash more than remaining epoch limit
        let remaining_limit = 50000u128 - (slash_amount * 2);
        let excessive_slash = remaining_limit + 1;
        
        assert_err!(
            DcfModule::execute_slash_validator_with_reason(
                &validator1, 
                excessive_slash, 
                SlashReason::Misbehavior
            ),
            Error::<Test>::SlashingBoundsExceeded
        );
    });
}

#[test]
fn test_saturating_arithmetic_in_score_calculations() {
    new_test_ext().execute_with(|| {
        let validator = 21u64;
        let initial_stake = 10_000u128;
        
        // Setup validator with maximum score
        setup_validator_with_stake(&validator, initial_stake);
        
        // Set validator to maximum score
        ValidatorStates::<Test>::mutate(&validator, |maybe_state| {
            if let Some(state) = maybe_state {
                state.current.final_score = 10000u64; // MaxValidatorScore
            }
        });
        
        // Reward the validator - score should not overflow
        let reward_amount = 1_000u128;
        assert_ok!(DcfModule::execute_reward_validator_with_reason(
            &validator, 
            reward_amount, 
            RewardReason::ExceptionalPerformance
        ));
        
        // Check that score is capped at maximum
        let final_score = ValidatorStates::<Test>::get(&validator)
            .map(|s| s.current.final_score)
            .unwrap_or(0);
        assert_eq!(final_score, 10000u64); // MaxValidatorScore
        
        // Now slash the validator - score should not underflow
        let large_slash = 50_000u128; // Large amount to test underflow protection
        assert_ok!(DcfModule::execute_slash_validator_with_reason(
            &validator, 
            large_slash, 
            SlashReason::Misbehavior
        ));
        
        // Check that score didn't underflow (should be >= 0)
        let final_score_after_slash = ValidatorStates::<Test>::get(&validator)
            .map(|s| s.current.final_score)
            .unwrap_or(0);
        // Score should be reduced but not negative (saturating_sub prevents underflow)
        assert!(final_score_after_slash < 10000u64); // MaxValidatorScore
    });
}