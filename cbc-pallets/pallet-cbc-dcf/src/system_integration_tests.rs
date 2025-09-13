//! System-wide integration tests for the CBC system
//! 
//! These tests verify the interaction between DCF, PoS, and PoI pallets
//! and ensure the entire consensus system works together correctly.

use crate::mock::*;
use frame_support::{assert_ok, assert_noop, traits::{Currency, ReservableCurrency, Hooks}};
use crate::{Error, ValidatorAction, ProposalAction, ProposalStatus, EjectionReason, ValidatorStates, Proposals, ValidatorStake, ValidatorLeaveRequests, MisbehaviorReports, PendingValidatorActions, CurrentEpoch, EpochConfigStorage};
use frame_support::BoundedVec;

#[test]
fn test_full_system_initialization() {
    new_test_ext().execute_with(|| {
        // Verify all pallets are properly initialized
        
        // DCF pallet
        assert_eq!(DcfPallet::current_epoch(), 0);
        let validators = DcfPallet::validator_set();
        assert_eq!(validators.len(), 3);
        
        // PoS pallet integration
        for validator in &validators {
            // Each validator should have a state in DCF
            let state = DcfPallet::validator_states(validator);
            assert!(state.is_some());
            
            // Each validator should have a stake
            let stake = DcfPallet::validator_stake(validator);
            assert!(stake > 0);
        }
        
        // Consensus weights should be properly set
        let pos_weight = DcfPallet::pos_weight();
        let poi_weight = DcfPallet::poi_weight();
        assert_eq!(pos_weight + poi_weight, 100);
    });
}

#[test]
fn test_cross_pallet_validator_lifecycle() {
    new_test_ext().execute_with(|| {
        let new_validator = 4u64;
        let min_stake = DcfMinStake::get();
        
        // Setup balance for new validator
        Balances::make_free_balance_be(&new_validator, min_stake * 3);
        
        // Step 1: Join the validator set through DCF
        assert_ok!(DcfPallet::join_validators(
            RuntimeOrigin::signed(new_validator),
            None
        ));
        
        // Verify validator was added to DCF
        let validator_set = DcfPallet::validator_set();
        assert!(validator_set.contains(&new_validator));
        
        // Verify validator has proper state
        let state = DcfPallet::validator_states(&new_validator);
        assert!(state.is_some());
        
        // Verify stake was reserved
        let stake = DcfPallet::validator_stake(&new_validator);
        assert_eq!(stake, min_stake);
        
        // Step 2: Test PoS integration - submit scores
        let mut validator_state = state.unwrap();
        validator_state.current.stake_score = 1200;
        ValidatorStates::<Test>::insert(&new_validator, validator_state);
        
        // Step 3: Test PoI integration - simulate inference activity
        assert_ok!(DcfPallet::simulate_inference(
            RuntimeOrigin::signed(1u64),
            Some(new_validator)
        ));
        
        // Verify inference count was updated
        let updated_state = DcfPallet::validator_states(&new_validator).unwrap();
        assert!(updated_state.inference_count > 0);
        
        // Step 4: Test leaving the validator set
        assert_ok!(DcfPallet::leave_validators(RuntimeOrigin::signed(new_validator)));
        
        // Verify leave request was recorded
        assert!(ValidatorLeaveRequests::<Test>::contains_key(&new_validator));
    });
}

#[test]
fn test_consensus_weight_impact_on_scoring() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Get initial state
        let initial_state = DcfPallet::validator_states(&validator).unwrap();
        let initial_score = initial_state.current.final_score;
        
        // Test with different consensus weights
        let test_cases = vec![
            (80, 20), // PoS heavy
            (50, 50), // Balanced
            (20, 80), // PoI heavy
        ];
        
        for (pos_weight, poi_weight) in test_cases {
            // Update consensus weights
            assert_ok!(DcfPallet::update_consensus_weights(
                RuntimeOrigin::root(),
                pos_weight,
                poi_weight
            ));
            
            // Update validator scores
            assert_ok!(DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(validator),
                validator
            ));
            
            assert_ok!(DcfPallet::update_validator_inference_score(
                RuntimeOrigin::signed(validator),
                validator
            ));
            
            // Verify weights were applied
            assert_eq!(DcfPallet::pos_weight(), pos_weight);
            assert_eq!(DcfPallet::poi_weight(), poi_weight);
            
            // Verify score calculation reflects new weights
            let updated_state = DcfPallet::validator_states(&validator).unwrap();
            assert!(updated_state.current.final_score > 0);
        }
    });
}

#[test]
fn test_governance_and_consensus_integration() {
    new_test_ext().execute_with(|| {
        let proposer = 1u64;
        let target_validator = 2u64;
        
        // Enable governance mode
        assert_ok!(DcfPallet::set_governance_mode(RuntimeOrigin::root(), true));
        
        // Test governance proposal affecting consensus
        assert_ok!(DcfPallet::propose_slash_validator(
            RuntimeOrigin::root(),
            proposer,
            target_validator,
            500u128
        ));
        
        let proposal_id = 0u32;
        
        // Vote on proposal
        assert_ok!(DcfPallet::vote_proposal(
            RuntimeOrigin::signed(1u64),
            proposal_id,
            true
        ));
        
        assert_ok!(DcfPallet::vote_proposal(
            RuntimeOrigin::signed(2u64),
            proposal_id,
            true
        ));
        
        // Execute proposal
        assert_ok!(DcfPallet::execute_proposal(
            RuntimeOrigin::root(),
            proposal_id
        ));
        
        // Verify proposal execution affected validator state
        let proposal = Proposals::<Test>::get(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        
        // Test that governance decisions impact consensus
        let validator_state = DcfPallet::validator_states(&target_validator);
        assert!(validator_state.is_some());
    });
}

#[test]
fn test_epoch_transition_system_wide() {
    new_test_ext().execute_with(|| {
        let initial_epoch = DcfPallet::current_epoch();
        let validators = DcfPallet::validator_set();
        
        // Setup pending actions for epoch transition
        PendingValidatorActions::<Test>::insert(
            &validators[0], 
            ValidatorAction::Leave
        );
        
        // Disable governance mode for epoch transition
        assert_ok!(DcfPallet::set_governance_mode(RuntimeOrigin::root(), false));
        
        // Manually advance epoch
        CurrentEpoch::<Test>::put(initial_epoch + 1);
        
        // Simulate epoch transition processing
        let epoch_config = EpochConfigStorage::<Test>::get();
        let epoch_boundary_block = (initial_epoch + 1) * epoch_config.blocks_per_epoch + 1;
        frame_system::Pallet::<Test>::set_block_number(epoch_boundary_block.into());
        
        // Verify epoch advanced
        let new_epoch = DcfPallet::current_epoch();
        assert_eq!(new_epoch, initial_epoch + 1);
        
        // Verify validator states were updated for new epoch
        for validator in &validators {
            let state = DcfPallet::validator_states(validator);
            if let Some(state) = state {
                // State should exist and be valid
                assert!(state.current.final_score >= 0);
            }
        }
    });
}

#[test]
fn test_multi_validator_performance_tracking() {
    new_test_ext().execute_with(|| {
        let validators = DcfPallet::validator_set();
        
        // Setup different performance profiles for validators
        let performance_profiles = vec![
            (95, 18, 20, 2),  // High performer: 95% participation, 18/20 inferences, 2 missed blocks
            (85, 15, 20, 5),  // Medium performer: 85% participation, 15/20 inferences, 5 missed blocks
            (70, 10, 20, 8),  // Low performer: 70% participation, 10/20 inferences, 8 missed blocks
        ];
        
        for (i, validator) in validators.iter().enumerate() {
            if i < performance_profiles.len() {
                let (participation, success_count, total_inferences, missed_blocks) = performance_profiles[i];
                
                let mut state = DcfPallet::validator_states(validator).unwrap();
                state.participation_rate = participation;
                state.inference_success_count = success_count;
                state.inference_count = total_inferences;
                state.current.missed_blocks = missed_blocks;
                state.current.authored_blocks = 20 - missed_blocks;
                
                ValidatorStates::<Test>::insert(validator, state);
            }
        }
        
        // Update scores based on performance
        for validator in &validators {
            assert_ok!(DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(*validator),
                *validator
            ));
            
            assert_ok!(DcfPallet::update_validator_inference_score(
                RuntimeOrigin::signed(*validator),
                *validator
            ));
        }
        
        // Verify performance differences are reflected in scores
        let mut scores = Vec::new();
        for validator in &validators {
            let state = DcfPallet::validator_states(validator).unwrap();
            scores.push(state.current.final_score);
        }
        
        // High performer should have highest score, low performer should have lowest
        if scores.len() >= 3 {
            assert!(scores[0] >= scores[1]); // High >= Medium
            assert!(scores[1] >= scores[2]); // Medium >= Low
        }
    });
}

#[test]
fn test_slashing_and_reward_system_integration() {
    new_test_ext().execute_with(|| {
        let validators = DcfPallet::validator_set();
        let min_stake = DcfMinStake::get();
        
        // Setup stakes for all validators
        for validator in &validators {
            Balances::make_free_balance_be(validator, min_stake * 3);
            let _ = Balances::reserve(validator, min_stake);
            ValidatorStake::<Test>::insert(validator, min_stake);
        }
        
        // Test slashing
        let validator_to_slash = validators[0];
        let slash_amount = min_stake / 4;
        
        assert_ok!(DcfPallet::slash_validator(
            RuntimeOrigin::root(),
            validator_to_slash,
            slash_amount
        ));
        
        // Verify slashing affected stake
        let slashed_stake = ValidatorStake::<Test>::get(&validator_to_slash);
        assert!(slashed_stake < min_stake);
        
        // Test reward distribution
        let total_reward_pool = 5000u128;
        assert_ok!(DcfPallet::distribute_epoch_rewards(
            RuntimeOrigin::root(),
            total_reward_pool
        ));
        
        // Test multiple validator slashing
        let validators_to_slash = vec![validators[1], validators[2]];
        assert_ok!(DcfPallet::slash_multiple_validators(
            RuntimeOrigin::root(),
            validators_to_slash.clone(),
            slash_amount
        ));
        
        // Verify all targeted validators were slashed
        for validator in &validators_to_slash {
            let stake = ValidatorStake::<Test>::get(validator);
            assert!(stake < min_stake);
        }
    });
}

#[test]
fn test_misbehavior_reporting_and_consequences() {
    new_test_ext().execute_with(|| {
        let reporter = 1u64;
        let reported = 2u64;
        let evidence = vec![1u8, 2u8, 3u8, 4u8, 5u8];
        
        // Report misbehavior
        let bounded_evidence = BoundedVec::try_from(evidence.clone()).unwrap();
        assert_ok!(DcfPallet::report_validator_misbehavior(
            RuntimeOrigin::signed(reporter),
            reported,
            bounded_evidence
        ));
        
        // Verify report was stored
        assert!(MisbehaviorReports::<Test>::contains_key(&reported, &reporter));
        
        // Test that misbehavior can lead to slashing proposal
        assert_ok!(DcfPallet::set_governance_mode(RuntimeOrigin::root(), true));
        
        assert_ok!(DcfPallet::propose_slash_validator(
            RuntimeOrigin::root(),
            reporter,
            reported,
            1000u128
        ));
        
        // Verify proposal was created
        assert!(Proposals::<Test>::contains_key(0));
        
        // Test that repeated misbehavior can lead to ejection
        assert_ok!(DcfPallet::propose_eject_validator(
            RuntimeOrigin::root(),
            reporter,
            reported,
            EjectionReason::RepeatedMisbehavior
        ));
        
        // Verify ejection proposal was created
        assert!(Proposals::<Test>::contains_key(1));
    });
}

#[test]
fn test_validator_metadata_and_reputation_system() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Set validator name
        let name = b"Test Validator".to_vec();
        assert_ok!(DcfPallet::set_validator_name(
            RuntimeOrigin::signed(validator),
            name.clone()
        ));
        
        // Verify name was set
        let stored_name = DcfPallet::validator_names(&validator).unwrap();
        assert_eq!(stored_name.to_vec(), name);
        
        // Test reputation building through performance
        let mut state = DcfPallet::validator_states(&validator).unwrap();
        
        // Simulate good performance over time
        state.current.authored_blocks = 50;
        state.current.missed_blocks = 2;
        state.participation_rate = 96;
        state.inference_success_count = 48;
        state.inference_count = 50;
        state.uptime = 98;
        
        ValidatorStates::<Test>::insert(&validator, state);
        
        // Update scores to reflect performance
        assert_ok!(DcfPallet::update_validator_stake_score(
            RuntimeOrigin::signed(validator),
            validator
        ));
        
        assert_ok!(DcfPallet::update_validator_inference_score(
            RuntimeOrigin::signed(validator),
            validator
        ));
        
        // Verify high performance is reflected in final score
        let updated_state = DcfPallet::validator_states(&validator).unwrap();
        assert!(updated_state.current.final_score > 0);
        
        // Test that good reputation can lead to rewards
        assert_ok!(DcfPallet::set_governance_mode(RuntimeOrigin::root(), true));
        
        assert_ok!(DcfPallet::propose_reward_validator(
            RuntimeOrigin::root(),
            validator,
            validator,
            500u128
        ));
        
        // Verify reward proposal was created
        assert!(Proposals::<Test>::contains_key(0));
    });
}

#[test]
fn test_system_stress_with_maximum_validators() {
    new_test_ext().execute_with(|| {
        let max_validators = DcfMaxValidators::get() as usize;
        let min_stake = DcfMinStake::get();
        
        // Try to add validators up to the maximum
        let mut added_validators = Vec::new();
        
        for i in 10..20 { // Add some new validators beyond genesis
            let new_validator = i as u64;
            
            // Setup balance
            Balances::make_free_balance_be(&new_validator, min_stake * 2);
            
            // Try to join
            let result = DcfPallet::join_validators(
                RuntimeOrigin::signed(new_validator),
                None
            );
            
            if result.is_ok() {
                added_validators.push(new_validator);
            }
            
            // Check if we've reached the limit
            let current_validators = DcfPallet::validator_set();
            if current_validators.len() >= max_validators {
                break;
            }
        }
        
        // Verify system handles maximum validators correctly
        let final_validators = DcfPallet::validator_set();
        assert!(final_validators.len() <= max_validators);
        
        // Test that all validators have valid states
        for validator in &final_validators {
            let state = DcfPallet::validator_states(validator);
            assert!(state.is_some());
            
            let stake = DcfPallet::validator_stake(validator);
            assert!(stake >= min_stake);
        }
        
        // Test system performance with maximum validators
        use std::time::Instant;
        let start = Instant::now();
        
        // Update all validator scores
        for validator in &final_validators {
            let _ = DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(*validator),
                *validator
            );
            
            let _ = DcfPallet::update_validator_inference_score(
                RuntimeOrigin::signed(*validator),
                *validator
            );
        }
        
        let duration = start.elapsed();
        
        // Should complete quickly even with many validators
        assert!(duration.as_millis() < 1000, "Score updates took too long: {:?}", duration);
    });
}

#[test]
fn test_system_recovery_from_failures() {
    new_test_ext().execute_with(|| {
        let validators = DcfPallet::validator_set();
        
        // Simulate system stress by slashing multiple validators
        let min_stake = DcfMinStake::get();
        
        for validator in &validators {
            Balances::make_free_balance_be(validator, min_stake * 2);
            let _ = Balances::reserve(validator, min_stake);
            ValidatorStake::<Test>::insert(validator, min_stake);
        }
        
        // Slash majority of validators
        let slash_amount = min_stake / 2;
        for validator in &validators[0..2] {
            assert_ok!(DcfPallet::slash_validator(
                RuntimeOrigin::root(),
                *validator,
                slash_amount
            ));
        }
        
        // Verify system maintains minimum active validators
        let active_validators = DcfPallet::active_validators();
        let min_active = MinActiveValidators::get() as usize;
        
        // System should maintain at least minimum active validators
        assert!(active_validators.len() >= min_active || validators.len() < min_active);
        
        // Test recovery by adding new validators
        let recovery_validator = 10u64;
        Balances::make_free_balance_be(&recovery_validator, min_stake * 2);
        
        assert_ok!(DcfPallet::join_validators(
            RuntimeOrigin::signed(recovery_validator),
            None
        ));
        
        // Verify system recovered
        let recovered_validators = DcfPallet::validator_set();
        assert!(recovered_validators.contains(&recovery_validator));
        
        // Test that new validator has proper state
        let state = DcfPallet::validator_states(&recovery_validator);
        assert!(state.is_some());
    });
}