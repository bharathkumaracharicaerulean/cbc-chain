//! Comprehensive tests for validator lifecycle edge case handling.
//!
//! This module tests the enhanced validator lifecycle management system,
//! focusing on edge cases and error conditions that can occur during
//! validator join, leave, and rejoin operations.
//!
//! # Test Coverage
//! - Cooldown period validation and enforcement
//! - Concurrent leave request prevention
//! - Rejoin validation with stake reservation checks
//! - Error handling and clear error messages
//! - Edge cases in validator state transitions

use super::*;
use crate::mock::*;
use frame_support::{
    assert_err, assert_ok,
    traits::Currency,
};

/// Test that validators cannot join while in cooldown state.
/// 
/// This test verifies requirement 10.1: "WHEN a validator attempts to join 
/// while cooling down THEN the system SHALL reject the request"
#[test]
fn test_join_while_in_cooldown_rejected() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        
        // First, join the validator set
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Then leave to enter cooldown
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Verify validator is in cooldown (has leave request)
        assert!(ValidatorLeaveRequests::<Test>::contains_key(&validator));
        
        // Attempt to join again while in cooldown should fail
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorHasPendingLeaveRequest
        );
        
        println!("✓ Validator correctly rejected when attempting to join while in cooldown");
    });
}

/// Test that validators cannot join while in recently removed cooldown.
/// 
/// This test verifies the recently removed cooldown mechanism that prevents
/// immediate rejoining after the leave cooldown expires.
#[test]
fn test_join_while_in_recently_removed_cooldown() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        
        // Simulate validator being recently removed (bypass normal leave process)
        let current_block = System::block_number().saturated_into::<u32>();
        RecentlyRemovedValidators::<Test>::insert(&validator, current_block);
        
        // Attempt to join while in recently removed cooldown should fail
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorRejoinCooldownNotExpired
        );
        
        // Advance blocks to expire cooldown
        let cooldown_period = 1000u32; // From mock configuration
        System::set_block_number((current_block + cooldown_period + 1).into());
        
        // Now joining should succeed
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Verify recently removed entry was cleaned up
        assert!(!RecentlyRemovedValidators::<Test>::contains_key(&validator));
        
        println!("✓ Recently removed cooldown correctly enforced and cleaned up");
    });
}

/// Test that concurrent leave requests are prevented.
/// 
/// This test verifies requirement 10.2: "WHEN concurrent leave requests are made 
/// THEN the system SHALL prevent conflicts"
#[test]
fn test_concurrent_leave_requests_prevented() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance and join
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // First leave request should succeed
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Verify leave request was recorded
        assert!(ValidatorLeaveRequests::<Test>::contains_key(&validator));
        
        // Second leave request should fail with specific error
        assert_err!(
            DcfPallet::leave_validators(RuntimeOrigin::signed(validator)),
            Error::<Test>::ConcurrentLeaveRequestNotAllowed
        );
        
        println!("✓ Concurrent leave requests correctly prevented");
    });
}

/// Test rejoin validation with stake reservation checks.
/// 
/// This test verifies requirement 10.3: "WHEN rejoining after cooldown THEN 
/// the system SHALL verify cooldown expiry and successful stake re-reservation"
#[test]
fn test_rejoin_validation_with_stake_checks() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Test case 1: Insufficient balance for stake reservation
        let _ = Balances::make_free_balance_be(&validator, min_stake / 2); // Insufficient
        
        // Simulate validator being recently removed but cooldown expired
        let current_block = System::block_number().saturated_into::<u32>();
        let cooldown_period = 1000u32; // From mock configuration
        let past_block = current_block.saturating_sub(cooldown_period + 1);
        RecentlyRemovedValidators::<Test>::insert(&validator, past_block);
        
        // Attempt to join with insufficient balance should fail
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::InsufficientStake // This error comes first in validation
        );
        
        // Test case 2: Sufficient balance, successful rejoin
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        
        // Now joining should succeed
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Verify stake was reserved
        assert_eq!(Balances::reserved_balance(&validator), min_stake);
        
        // Verify recently removed entry was cleaned up
        assert!(!RecentlyRemovedValidators::<Test>::contains_key(&validator));
        
        println!("✓ Rejoin validation with stake checks working correctly");
    });
}

/// Test detailed cooldown status reporting.
/// 
/// This test verifies that the system provides clear information about
/// cooldown status and remaining time.
#[test]
fn test_detailed_cooldown_status_reporting() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance and join
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Initially no cooldown
        assert_eq!(DcfPallet::get_validator_detailed_cooldown_status(validator), None);
        
        // Submit leave request
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Check cooldown status during leave request
        let cooldown_status = DcfPallet::get_validator_detailed_cooldown_status(validator);
        assert!(cooldown_status.is_some());
        let (blocks_remaining, can_rejoin) = cooldown_status.unwrap();
        assert_eq!(blocks_remaining, 1000u32); // From mock configuration
        assert_eq!(can_rejoin, false); // Cannot rejoin while leaving
        
        // Advance some blocks
        let advance_blocks = 100u32;
        let current_block = System::block_number().saturated_into::<u32>();
        System::set_block_number((current_block + advance_blocks).into());
        
        // Check updated cooldown status
        let cooldown_status = DcfPallet::get_validator_detailed_cooldown_status(validator);
        assert!(cooldown_status.is_some());
        let (blocks_remaining, can_rejoin) = cooldown_status.unwrap();
        assert_eq!(blocks_remaining, 1000u32 - advance_blocks);
        assert_eq!(can_rejoin, false); // Still cannot rejoin while leaving
        
        println!("✓ Detailed cooldown status reporting working correctly");
    });
}

/// Test leave request cancellation edge cases.
/// 
/// This test verifies that leave request cancellation works correctly
/// and handles edge cases properly.
#[test]
fn test_leave_request_cancellation_edge_cases() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Test case 1: Cancel non-existent leave request
        assert_err!(
            DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)),
            Error::<Test>::ValidatorNotFound
        );
        
        // Setup validator
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Test case 2: Cancel before cooldown expires (should succeed)
        assert_ok!(DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)));
        
        // Verify leave request was removed
        assert!(!ValidatorLeaveRequests::<Test>::contains_key(&validator));
        
        // Test case 3: Try to cancel after cooldown would have expired
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Advance blocks past cooldown period
        let current_block = System::block_number().saturated_into::<u32>();
        let cooldown_period = 1000u32; // From mock configuration
        System::set_block_number((current_block + cooldown_period + 1).into());
        
        // Cancellation after expiry should fail
        assert_err!(
            DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)),
            Error::<Test>::LeaveCooldownActive
        );
        
        println!("✓ Leave request cancellation edge cases handled correctly");
    });
}

/// Test validator state consistency during lifecycle transitions.
/// 
/// This test ensures that validator state remains consistent during
/// all lifecycle transitions and edge cases.
#[test]
fn test_validator_state_consistency() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        
        // Initial state: not in any set
        assert!(!ValidatorSet::<Test>::get().contains(&validator));
        assert!(!ActiveValidators::<Test>::get().contains(&validator));
        assert!(!ValidatorLeaveRequests::<Test>::contains_key(&validator));
        assert!(!RecentlyRemovedValidators::<Test>::contains_key(&validator));
        
        // Join validator set
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // State after joining
        assert!(ValidatorSet::<Test>::get().contains(&validator));
        assert_eq!(Balances::reserved_balance(&validator), min_stake);
        assert!(ValidatorStates::<Test>::contains_key(&validator));
        
        // Submit leave request
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // State during leave cooldown
        assert!(ValidatorSet::<Test>::get().contains(&validator)); // Still in set
        assert!(!ActiveValidators::<Test>::get().contains(&validator)); // Removed from active
        assert!(ValidatorLeaveRequests::<Test>::contains_key(&validator)); // Has leave request
        assert_eq!(Balances::reserved_balance(&validator), min_stake); // Stake still reserved
        
        // Cancel leave request
        assert_ok!(DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)));
        
        // State after cancellation
        assert!(ValidatorSet::<Test>::get().contains(&validator)); // Still in set
        assert!(!ValidatorLeaveRequests::<Test>::contains_key(&validator)); // No leave request
        assert_eq!(Balances::reserved_balance(&validator), min_stake); // Stake still reserved
        
        println!("✓ Validator state consistency maintained throughout lifecycle");
    });
}

/// Test error message clarity and specificity.
/// 
/// This test verifies requirement 10.4: "WHEN edge cases occur THEN 
/// the system SHALL provide clear error codes and messages"
#[test]
fn test_error_message_clarity() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Test specific error for joining while in cooldown
        let _ = Balances::make_free_balance_be(&validator, min_stake * 2);
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Should get specific error for pending leave request
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorHasPendingLeaveRequest
        );
        
        // Test specific error for concurrent leave requests
        assert_err!(
            DcfPallet::leave_validators(RuntimeOrigin::signed(validator)),
            Error::<Test>::ConcurrentLeaveRequestNotAllowed
        );
        
        // Test specific error for cooldown not expired
        assert_ok!(DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)));
        let current_block = System::block_number().saturated_into::<u32>();
        RecentlyRemovedValidators::<Test>::insert(&validator, current_block);
        
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorRejoinCooldownNotExpired
        );
        
        // Test specific error for insufficient stake
        let _ = Balances::make_free_balance_be(&validator, min_stake / 2);
        let cooldown_period = 1000u32; // From mock configuration
        System::set_block_number((current_block + cooldown_period + 1).into());
        
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::InsufficientStake
        );
        
        println!("✓ Error messages are clear and specific for each edge case");
    });
}

/// Test comprehensive validator lifecycle scenario.
/// 
/// This test runs through a complete validator lifecycle with multiple
/// edge cases to ensure the system handles complex scenarios correctly.
#[test]
fn test_comprehensive_validator_lifecycle_scenario() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        let min_stake = <Test as Config>::MinStake::get();
        
        // Ensure validator has sufficient balance
        let _ = Balances::make_free_balance_be(&validator, min_stake * 3);
        
        // Scenario 1: Normal join -> leave -> rejoin cycle
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Try to join while leaving (should fail)
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorHasPendingLeaveRequest
        );
        
        // Cancel and try again
        assert_ok!(DcfPallet::cancel_leave_request(RuntimeOrigin::signed(validator)));
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(validator)));
        
        // Simulate cooldown expiry and automatic removal
        let current_block = System::block_number().saturated_into::<u32>();
        let cooldown_period = 1000u32; // From mock configuration
        System::set_block_number((current_block + cooldown_period + 1).into());
        
        // Simulate the validator being moved to recently removed
        ValidatorLeaveRequests::<Test>::remove(&validator);
        RecentlyRemovedValidators::<Test>::insert(&validator, current_block);
        
        // Try to rejoin immediately (should fail due to recently removed cooldown)
        assert_err!(
            DcfPallet::join_validators(RuntimeOrigin::signed(validator), None),
            Error::<Test>::ValidatorRejoinCooldownNotExpired
        );
        
        // Advance past recently removed cooldown
        let new_block = System::block_number().saturated_into::<u32>();
        let cooldown_period = 1000u32; // From mock configuration
        System::set_block_number((new_block + cooldown_period + 1).into());
        
        // Now rejoin should succeed
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Verify final state is correct
        assert!(ValidatorSet::<Test>::get().contains(&validator));
        assert_eq!(Balances::reserved_balance(&validator), min_stake);
        assert!(!ValidatorLeaveRequests::<Test>::contains_key(&validator));
        assert!(!RecentlyRemovedValidators::<Test>::contains_key(&validator));
        
        println!("✓ Comprehensive validator lifecycle scenario completed successfully");
    });
}