//! DoS Protection and Rate Limiting Tests
//!
//! This module contains comprehensive tests for the DoS protection and rate limiting
//! functionality implemented in the DCF pallet. It verifies that:
//!
//! - Per-block rate limits are enforced correctly
//! - Per-account rate limits work within time windows
//! - Minimum intervals between operations are respected
//! - Weight bounds prevent excessive computation
//! - Rate limiting configuration can be updated safely
//! - Operations are properly recorded for tracking

use super::*;
use crate::mock::*;
use frame_support::{
    assert_err, assert_ok,
    weights::Weight,
};
use sp_runtime::traits::BadOrigin;

// Type alias for the DCF pallet
type DcfModule = Pallet<Test>;

/// Test per-block rate limiting for proposal submissions
#[test]
fn per_block_proposal_rate_limiting_works() {
    new_test_ext().execute_with(|| {
        // Initialize with Alice as validator
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        
        // Set a low per-block limit for testing
        let mut config = RateLimitConfig::default();
        config.max_proposals_per_block = 2;
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First proposal should succeed
        let action1 = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            action1,
            None
        ));
        
        // Second proposal should succeed
        let action2 = ProposalAction::Reward { validator: BOB, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(BOB),
            action2,
            None
        ));
        
        // Third proposal should fail due to per-block limit
        let action3 = ProposalAction::Reward { validator: CHARLIE, amount: 1000 };
        assert_err!(
            DcfModule::submit_proposal(RuntimeOrigin::signed(CHARLIE), action3, None),
            Error::<Test>::PerBlockRateLimitExceeded
        );
        
        // Move to next block and reset counters
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // Now the proposal should succeed again
        let action4 = ProposalAction::Reward { validator: CHARLIE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(CHARLIE),
            action4,
            None
        ));
    });
}

/// Test per-account rate limiting for proposal submissions
#[test]
fn per_account_proposal_rate_limiting_works() {
    new_test_ext().execute_with(|| {
        // Initialize with Alice as validator
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        
        // Set a low per-account limit for testing
        let mut config = RateLimitConfig::default();
        config.max_proposals_per_account = 2;
        config.proposal_rate_window = 10; // 10 blocks window
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First proposal should succeed
        let action1 = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            action1,
            None
        ));
        
        // Move to next block to avoid per-block limits
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // Second proposal from same account should succeed
        let action2 = ProposalAction::Reward { validator: BOB, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            action2,
            None
        ));
        
        // Move to next block
        System::set_block_number(3);
        DcfModule::on_initialize(3);
        
        // Third proposal from same account should fail due to per-account limit
        let action3 = ProposalAction::Reward { validator: CHARLIE, amount: 1000 };
        assert_err!(
            DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action3, None),
            Error::<Test>::PerAccountRateLimitExceeded
        );
        
        // Move beyond the rate window
        System::set_block_number(15);
        DcfModule::on_initialize(15);
        
        // Now the proposal should succeed again
        let action4 = ProposalAction::Reward { validator: DAVE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(
            RuntimeOrigin::signed(ALICE),
            action4,
            None
        ));
    });
}

/// Test minimum interval enforcement for validator status changes
#[test]
fn minimum_interval_enforcement_works() {
    new_test_ext().execute_with(|| {
        // Set a minimum interval for testing
        let mut config = RateLimitConfig::default();
        config.min_validator_status_interval = 5; // 5 blocks minimum
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First join should succeed
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        
        // Immediate leave should fail due to minimum interval
        assert_err!(
            DcfModule::leave_validators(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::MinimumIntervalViolation
        );
        
        // Move forward but not enough blocks
        System::set_block_number(4);
        DcfModule::on_initialize(4);
        
        // Still should fail
        assert_err!(
            DcfModule::leave_validators(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::MinimumIntervalViolation
        );
        
        // Move beyond minimum interval
        System::set_block_number(7);
        DcfModule::on_initialize(7);
        
        // Now leave should succeed
        assert_ok!(DcfModule::leave_validators(RuntimeOrigin::signed(ALICE)));
    });
}

/// Test weight bounds validation for operations with large loops
#[test]
fn weight_bounds_validation_works() {
    new_test_ext().execute_with(|| {
        // Set very low weight limits for testing
        let mut config = RateLimitConfig::default();
        config.max_validator_iteration_weight = Weight::from_parts(1000, 0); // Very low limit
        config.max_loop_iterations = 2; // Very low iteration limit
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // Create many validators to exceed weight limits
        let validators = vec![ALICE, BOB, CHARLIE, DAVE, EVE];
        
        // This should fail due to weight limits
        assert_err!(
            DcfModule::propose_reward_multiple_validators(
                RuntimeOrigin::root(),
                ALICE,
                validators,
                1000
            ),
            Error::<Test>::WeightLimitExceeded
        );
        
        // Smaller list should succeed
        let small_validators = vec![ALICE, BOB];
        assert_ok!(DcfModule::propose_reward_multiple_validators(
            RuntimeOrigin::root(),
            ALICE,
            small_validators,
            1000
        ));
    });
}

/// Test rate limiting configuration updates
#[test]
fn rate_limit_config_updates_work() {
    new_test_ext().execute_with(|| {
        // Get initial config
        let initial_config = DcfModule::rate_limit_config();
        
        // Update with new values
        let mut new_config = RateLimitConfig::default();
        new_config.max_proposals_per_block = 10;
        new_config.max_joins_per_block = 5;
        
        assert_ok!(DcfModule::update_rate_limit_config(
            RuntimeOrigin::root(),
            new_config.clone()
        ));
        
        // Verify config was updated
        let updated_config = DcfModule::rate_limit_config();
        assert_eq!(updated_config.max_proposals_per_block, 10);
        assert_eq!(updated_config.max_joins_per_block, 5);
        
        // Check that event was emitted
        System::assert_has_event(
            Event::RateLimitConfigUpdated {
                max_proposals_per_block: 10,
                max_joins_per_block: 5,
                max_leaves_per_block: new_config.max_leaves_per_block,
            }.into()
        );
    });
}

/// Test invalid rate limiting configuration rejection
#[test]
fn invalid_rate_limit_config_rejected() {
    new_test_ext().execute_with(|| {
        // Try to set invalid config (zero limits)
        let mut invalid_config = RateLimitConfig::default();
        invalid_config.max_proposals_per_block = 0; // Invalid
        
        assert_err!(
            DcfModule::update_rate_limit_config(RuntimeOrigin::root(), invalid_config),
            Error::<Test>::InvalidRateLimitConfig
        );
        
        // Try to set excessively high limits
        let mut invalid_config2 = RateLimitConfig::default();
        invalid_config2.max_proposals_per_block = 1000; // Too high
        
        assert_err!(
            DcfModule::update_rate_limit_config(RuntimeOrigin::root(), invalid_config2),
            Error::<Test>::InvalidRateLimitConfig
        );
    });
}

/// Test that only root can update rate limiting configuration
#[test]
fn rate_limit_config_requires_root() {
    new_test_ext().execute_with(|| {
        let config = RateLimitConfig::default();
        
        // Non-root should fail
        assert_err!(
            DcfModule::update_rate_limit_config(RuntimeOrigin::signed(ALICE), config.clone()),
            BadOrigin
        );
        
        // Root should succeed
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
    });
}

/// Test rate limiting for validator join operations
#[test]
fn validator_join_rate_limiting_works() {
    new_test_ext().execute_with(|| {
        // Set low join limit for testing
        let mut config = RateLimitConfig::default();
        config.max_joins_per_block = 1;
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First join should succeed
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        
        // Second join in same block should fail
        assert_err!(
            DcfModule::join_validators(RuntimeOrigin::signed(BOB), None),
            Error::<Test>::PerBlockRateLimitExceeded
        );
        
        // Move to next block
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // Now join should succeed
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(BOB), None));
    });
}

/// Test rate limiting for validator leave operations
#[test]
fn validator_leave_rate_limiting_works() {
    new_test_ext().execute_with(|| {
        // Setup validators first
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(BOB), None));
        
        // Move to next block to avoid join rate limits
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // Set low leave limit for testing
        let mut config = RateLimitConfig::default();
        config.max_leaves_per_block = 1;
        config.min_validator_status_interval = 0; // Disable interval check for this test
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First leave should succeed
        assert_ok!(DcfModule::leave_validators(RuntimeOrigin::signed(ALICE)));
        
        // Second leave in same block should fail
        assert_err!(
            DcfModule::leave_validators(RuntimeOrigin::signed(BOB)),
            Error::<Test>::PerBlockRateLimitExceeded
        );
        
        // Move to next block
        System::set_block_number(3);
        DcfModule::on_initialize(3);
        
        // Now leave should succeed
        assert_ok!(DcfModule::leave_validators(RuntimeOrigin::signed(BOB)));
    });
}

/// Test rate limiting for governance voting
#[test]
fn governance_vote_rate_limiting_works() {
    new_test_ext().execute_with(|| {
        // Setup validators and proposal
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(BOB), None));
        
        let action = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action, None));
        
        // Set low vote limit for testing
        let mut config = RateLimitConfig::default();
        config.max_votes_per_block = 1;
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // Move to next block to avoid proposal rate limits
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // First vote should succeed
        assert_ok!(DcfModule::vote_proposal(RuntimeOrigin::signed(ALICE), 0, true));
        
        // Second vote in same block should fail
        assert_err!(
            DcfModule::vote_proposal(RuntimeOrigin::signed(BOB), 0, true),
            Error::<Test>::PerBlockRateLimitExceeded
        );
        
        // Move to next block
        System::set_block_number(3);
        DcfModule::on_initialize(3);
        
        // Now vote should succeed
        assert_ok!(DcfModule::vote_proposal(RuntimeOrigin::signed(BOB), 0, true));
    });
}

/// Test that block counter reset works correctly
#[test]
fn block_counter_reset_works() {
    new_test_ext().execute_with(|| {
        // Submit a proposal to increment counter
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        let action = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action, None));
        
        // Check counter is incremented
        assert_eq!(
            DcfModule::block_operation_counts(&DispatchableType::SubmitProposal),
            1
        );
        
        // Move to next block and initialize
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        // Check counter is reset
        assert_eq!(
            DcfModule::block_operation_counts(&DispatchableType::SubmitProposal),
            0
        );
        
        // Check that reset event was emitted
        System::assert_has_event(
            Event::BlockRateLimitCountersReset {
                block_number: 2,
            }.into()
        );
    });
}

/// Test operation recording and history tracking
#[test]
fn operation_recording_works() {
    new_test_ext().execute_with(|| {
        // Submit a proposal
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        let action = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action, None));
        
        // Check that operation was recorded
        let history = DcfModule::account_operation_history(&ALICE, &DispatchableType::SubmitProposal);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], 1); // Block 1
        
        // Check last operation block
        assert_eq!(
            DcfModule::last_operation_block(&ALICE, &DispatchableType::SubmitProposal),
            Some(1)
        );
        
        // Move to next block and submit another proposal
        System::set_block_number(2);
        DcfModule::on_initialize(2);
        
        let action2 = ProposalAction::Reward { validator: BOB, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action2, None));
        
        // Check history is updated
        let updated_history = DcfModule::account_operation_history(&ALICE, &DispatchableType::SubmitProposal);
        assert_eq!(updated_history.len(), 2);
        assert_eq!(updated_history[1], 2); // Block 2
        
        // Check last operation block is updated
        assert_eq!(
            DcfModule::last_operation_block(&ALICE, &DispatchableType::SubmitProposal),
            Some(2)
        );
    });
}

/// Test rate limit violation events are emitted
#[test]
fn rate_limit_violation_events_emitted() {
    new_test_ext().execute_with(|| {
        // Set low limits for testing
        let mut config = RateLimitConfig::default();
        config.max_proposals_per_block = 1;
        assert_ok!(DcfModule::update_rate_limit_config(RuntimeOrigin::root(), config));
        
        // First proposal succeeds
        assert_ok!(DcfModule::join_validators(RuntimeOrigin::signed(ALICE), None));
        let action1 = ProposalAction::Reward { validator: ALICE, amount: 1000 };
        assert_ok!(DcfModule::submit_proposal(RuntimeOrigin::signed(ALICE), action1, None));
        
        // Second proposal fails and should emit violation event
        let action2 = ProposalAction::Reward { validator: BOB, amount: 1000 };
        assert_err!(
            DcfModule::submit_proposal(RuntimeOrigin::signed(BOB), action2, None),
            Error::<Test>::PerBlockRateLimitExceeded
        );
        
        // Check that violation event was emitted
        System::assert_has_event(
            Event::RateLimitViolation {
                account: BOB,
                operation: 0, // SubmitProposal
                violation_type: 0, // PerBlock
                current_count: 1,
                limit: 1,
            }.into()
        );
    });
}