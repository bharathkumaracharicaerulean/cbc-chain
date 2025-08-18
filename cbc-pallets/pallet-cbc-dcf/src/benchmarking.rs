//! Benchmarking setup for pallet-cbc-dcf
//! 
//! This module provides comprehensive benchmarks for all DCF operations
//! to measure performance characteristics and optimize for production use.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_benchmarking::account;
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_std::vec;
use crate::{Pallet as DcfPallet, Config, ProposalAction};

/// Helper function to create a validator account with initial state
fn create_validator<T: Config>(id: u32) -> T::AccountId {
    let validator: T::AccountId = account("validator", id, 0);
    
    // Create validator state
    let validator_state = ValidatorState {
        last_active_epoch: 0,
        current: EpochStats {
            epoch: 0,
            stake_score: 1000,
            inference_score: 800,
            final_score: 900,
            authored_blocks: 0,
            missed_blocks: 0,
        },
        history: BoundedVec::default(),
        uptime: 0,
        inference_success_count: 0,
        participation_rate: 100,
        inference_count: 0,
        last_active_block: 0,
        name: None,
    };
    
    ValidatorStates::<T>::insert(&validator, validator_state);
    validator
}

/// Helper function to setup multiple validators
fn setup_validators<T: Config>(count: u32) -> Vec<T::AccountId> {
    let mut validators = Vec::new();
    for i in 0..count {
        validators.push(create_validator::<T>(i));
    }
    
    // Add to validator set
    let bounded_validators = BoundedVec::try_from(validators.clone())
        .expect("Too many validators for benchmarking");
    ValidatorSet::<T>::put(bounded_validators.clone());
    ActiveValidators::<T>::put(bounded_validators);
    
    validators
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// Benchmark validator stake score update
    #[benchmark]
    fn update_validator_stake_score() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());

        #[extrinsic_call]
        update_validator_stake_score(origin, validator.clone());

        assert!(ValidatorStates::<T>::contains_key(&validator));
    }

    /// Benchmark validator inference score update
    #[benchmark]
    fn update_validator_inference_score() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());

        #[extrinsic_call]
        update_validator_inference_score(origin, validator.clone());

        assert!(ValidatorStates::<T>::contains_key(&validator));
    }

    /// Benchmark consensus weight updates
    #[benchmark]
    fn update_consensus_weights() {
        let origin = RawOrigin::Root;

        #[extrinsic_call]
        update_consensus_weights(origin, 70, 30);

        assert_eq!(PosWeight::<T>::get(), 70);
        assert_eq!(PoiWeight::<T>::get(), 30);
    }

    /// Benchmark governance mode toggle
    #[benchmark]
    fn set_governance_mode() {
        let origin = RawOrigin::Root;

        #[extrinsic_call]
        set_governance_mode(origin, true);

        assert!(GovernanceModeEnabled::<T>::get());
    }

    /// Benchmark epoch advancement
    #[benchmark]
    fn sudo_advance_epoch() {
        let origin = RawOrigin::Root;
        GovernanceModeEnabled::<T>::put(true);
        
        // Set up some validators and epoch config
        let _validators = setup_validators::<T>(3);
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 100,
        };
        EpochConfigStorage::<T>::put(epoch_config);

        #[extrinsic_call]
        sudo_advance_epoch(origin);

        // The epoch should be advanced
        assert!(CurrentEpoch::<T>::get() > 0);
    }

    /// Benchmark proposal submission
    #[benchmark]
    fn submit_proposal() {
        let proposer = create_validator::<T>(0);
        let target = create_validator::<T>(1);
        let origin = RawOrigin::Signed(proposer.clone());
        
        GovernanceModeEnabled::<T>::put(true);
        
        let action = ProposalAction::Slash {
            validator: target,
            amount: 1000u32.into(),
        };

        #[extrinsic_call]
        submit_proposal(origin, action, None);

        assert!(Proposals::<T>::contains_key(0));
    }

    /// Benchmark proposal voting
    #[benchmark]
    fn vote_proposal() {
        let proposer = create_validator::<T>(0);
        let voter = create_validator::<T>(1);
        let target = create_validator::<T>(2);
        
        GovernanceModeEnabled::<T>::put(true);
        
        // Create a proposal first
        let action = ProposalAction::Slash {
            validator: target,
            amount: 1000u32.into(),
        };
        
        let proposal = GovernanceProposal {
            proposer: proposer.clone(),
            action,
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
        };
        
        Proposals::<T>::insert(0, proposal);
        
        let origin = RawOrigin::Signed(voter);

        #[extrinsic_call]
        vote_proposal(origin, 0, true);

        let updated_proposal = Proposals::<T>::get(0).unwrap();
        assert_eq!(updated_proposal.votes_for, 1);
    }

    /// Benchmark proposal execution
    #[benchmark]
    fn execute_proposal() {
        let validators = setup_validators::<T>(5);
        let proposer = &validators[0];
        let target = &validators[1];
        
        GovernanceModeEnabled::<T>::put(true);
        
        // Create an approved proposal with a simple reward action
        let action = ProposalAction::Reward {
            validator: target.clone(),
            amount: 1000u32.into(),
        };
        
        let proposal = GovernanceProposal {
            proposer: proposer.clone(),
            action,
            status: ProposalStatus::Approved,
            votes_for: 10,
            votes_against: 0,
        };
        
        Proposals::<T>::insert(0, proposal);
        
        let origin = RawOrigin::Root;

        #[extrinsic_call]
        execute_proposal(origin, 0);

        let executed_proposal = Proposals::<T>::get(0).unwrap();
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
    }

    /// Benchmark validator join
    #[benchmark]
    fn join_validator_set() {
        let _validators = setup_validators::<T>(5); // Create enough validators
        let validator = create_validator::<T>(10); // Create a new validator not in active set
        let origin = RawOrigin::Signed(validator.clone());

        #[extrinsic_call]
        join_validator_set(origin);

        assert_eq!(
            PendingValidatorActions::<T>::get(&validator),
            Some(ValidatorAction::Join)
        );
    }

    /// Benchmark validator leave
    #[benchmark]
    fn leave_validator_set() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());
        
        // Add to active set first
        let bounded_validators = BoundedVec::try_from(vec![validator.clone()])
            .expect("Single validator should fit");
        ActiveValidators::<T>::put(bounded_validators);

        #[extrinsic_call]
        leave_validator_set(origin);

        assert_eq!(
            PendingValidatorActions::<T>::get(&validator),
            Some(ValidatorAction::Leave)
        );
    }

    /// Benchmark validator name setting
    #[benchmark]
    fn set_validator_name() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());
        let name = b"Benchmark Validator".to_vec();

        #[extrinsic_call]
        set_validator_name(origin, name.clone());

        let stored_name = ValidatorNames::<T>::get(&validator).unwrap();
        assert_eq!(stored_name.to_vec(), name);
    }

    /// Benchmark off-chain PoI score application
    #[benchmark]
    fn apply_offchain_poi_scores() {
        let validators = setup_validators::<T>(3);
        let validator = &validators[0];
        let origin = RawOrigin::Signed(validator.clone());
        let block_number = 100u32;

        // This will likely not find any offchain scores, but should complete without error
        #[extrinsic_call]
        apply_offchain_poi_scores(origin, block_number);

        // Function should complete without error
        assert!(ValidatorStates::<T>::contains_key(&validator));
    }

    /// Benchmark on_initialize hook with multiple validators
    #[benchmark]
    fn on_initialize_with_validators(v: Linear<1, 100>) {
        let validators = setup_validators::<T>(v);
        let block_number = 1000u32.into();
        
        // Set up epoch config
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 100,
        };
        EpochConfigStorage::<T>::put(epoch_config);

        #[block]
        {
            DcfPallet::<T>::on_initialize(block_number);
        }

        // Verify validators are still tracked
        assert_eq!(ValidatorSet::<T>::get().len(), validators.len());
    }

    /// Benchmark score decay for multiple validators
    #[benchmark]
    fn apply_score_decay_multiple(v: Linear<1, 50>) {
        let validators = setup_validators::<T>(v);
        let current_epoch = 5u32;

        #[block]
        {
            for validator in &validators {
                let _ = DcfPallet::<T>::apply_score_decay(validator, current_epoch);
            }
        }

        // Verify all validators still exist
        for validator in &validators {
            assert!(ValidatorStates::<T>::contains_key(validator));
        }
    }

    /// Benchmark validator set operations
    #[benchmark]
    fn validator_set_operations(v: Linear<1, 100>) {
        let validators = setup_validators::<T>(v);

        #[block]
        {
            // Test various validator set operations
            let _active = DcfPallet::<T>::active_validators();
            let _validator_set = DcfPallet::<T>::validator_set();
            
            // Test validator queries
            for validator in &validators {
                let _ = DcfPallet::<T>::validator_states(validator);
                let _ = DcfPallet::<T>::is_validator_active(validator);
            }
        }

        assert_eq!(validators.len(), v as usize);
    }

    /// Benchmark runtime API calls
    #[benchmark]
    fn runtime_api_calls(v: Linear<1, 50>) {
        let validators = setup_validators::<T>(v);

        #[block]
        {
            // Test runtime API performance
            let _ = DcfPallet::<T>::current_epoch();
            let _ = DcfPallet::<T>::active_validators();
            let _ = DcfPallet::<T>::pos_weight();
            let _ = DcfPallet::<T>::poi_weight();
            
            // Test per-validator APIs
            for validator in &validators {
                let _ = DcfPallet::<T>::get_validator_profile(validator.clone());
                let _ = DcfPallet::<T>::validator_states(validator);
            }
        }

        assert_eq!(validators.len(), v as usize);
    }

    /// Benchmark epoch transition with multiple validators
    #[benchmark]
    fn epoch_transition_multiple(v: Linear<1, 50>) {
        let validators = setup_validators::<T>(v);
        
        // Set up some pending actions
        for (i, validator) in validators.iter().enumerate() {
            if i % 2 == 0 {
                PendingValidatorActions::<T>::insert(validator, ValidatorAction::Join);
            }
        }

        #[block]
        {
            // Simulate epoch transition by advancing epoch manually
            let current_epoch = DcfPallet::<T>::current_epoch();
            CurrentEpoch::<T>::put(current_epoch + 1);
        }

        // Verify epoch was advanced
        assert_eq!(CurrentEpoch::<T>::get(), 1);
    }

    /// Benchmark governance proposal with multiple voters
    #[benchmark]
    fn governance_with_multiple_voters(v: Linear<3, 50>) {
        let validators = setup_validators::<T>(v);
        let proposer = &validators[0];
        let target = &validators[1];
        
        GovernanceModeEnabled::<T>::put(true);
        
        // Create proposal
        let action = ProposalAction::Reward {
            validator: target.clone(),
            amount: 1000u32.into(),
        };
        
        let proposal = GovernanceProposal {
            proposer: proposer.clone(),
            action,
            status: ProposalStatus::Pending,
            votes_for: 0,
            votes_against: 0,
        };
        
        Proposals::<T>::insert(0, proposal);

        #[block]
        {
            // All validators vote
            for (i, validator) in validators.iter().enumerate().skip(2) {
                let _ = DcfPallet::<T>::vote_proposal(
                    RawOrigin::Signed(validator.clone()).into(),
                    0,
                    i % 2 == 0, // Alternate votes
                );
            }
        }

        let final_proposal = Proposals::<T>::get(0).unwrap();
        assert!(final_proposal.votes_for + final_proposal.votes_against > 0);
    }

    /// Benchmark multiple validators joining simultaneously
    #[benchmark]
    fn join_validators(v: Linear<1, 20>) {
        let _existing_validators = setup_validators::<T>(5);
        let mut new_validators = Vec::new();
        
        // Create new validators that want to join
        for i in 100..(100 + v) {
            let validator = create_validator::<T>(i);
            new_validators.push(validator);
        }

        #[block]
        {
            // All new validators attempt to join
            for validator in &new_validators {
                let _ = DcfPallet::<T>::join_validator_set(
                    RawOrigin::Signed(validator.clone()).into()
                );
            }
        }

        // Verify all join requests were recorded
        for validator in &new_validators {
            assert_eq!(
                PendingValidatorActions::<T>::get(validator),
                Some(ValidatorAction::Join)
            );
        }
    }

    /// Benchmark multiple validators leaving simultaneously
    #[benchmark]
    fn leave_validators(v: Linear<1, 20>) {
        let validators = setup_validators::<T>(v + 5); // Ensure we have enough validators
        let leaving_validators = &validators[0..v as usize];

        #[block]
        {
            // Multiple validators request to leave
            for validator in leaving_validators {
                let _ = DcfPallet::<T>::leave_validator_set(
                    RawOrigin::Signed(validator.clone()).into()
                );
            }
        }

        // Verify all leave requests were recorded
        for validator in leaving_validators {
            assert_eq!(
                PendingValidatorActions::<T>::get(validator),
                Some(ValidatorAction::Leave)
            );
        }
    }

    /// Benchmark validator slashing operations
    #[benchmark]
    fn slash_validator() {
        let validators = setup_validators::<T>(5);
        let slasher = &validators[0];
        let target = &validators[1];
        
        // Give the target some balance to slash
        T::Currency::make_free_balance_be(target, 10000u32.into());
        let _ = T::Currency::reserve(target, 5000u32.into());
        
        let origin = RawOrigin::Signed(slasher.clone());
        let slash_amount = 1000u32.into();

        #[extrinsic_call]
        slash_validator(origin, target.clone(), slash_amount);

        // Verify the validator state was updated
        let state = ValidatorStates::<T>::get(target).unwrap();
        assert!(state.current.stake_score > 0);
    }

    /// Benchmark proposal execution with balance transfers
    #[benchmark]
    fn execute_proposals_with_transfers() {
        let validators = setup_validators::<T>(5);
        let proposer = &validators[0];
        let target = &validators[1];
        
        GovernanceModeEnabled::<T>::put(true);
        
        // Give the treasury some balance for rewards
        let treasury_account: T::AccountId = account("treasury", 0, 0);
        T::Currency::make_free_balance_be(&treasury_account, 100000u32.into());
        
        // Create a reward proposal that involves balance transfer
        let action = ProposalAction::Reward {
            validator: target.clone(),
            amount: 5000u32.into(),
        };
        
        let proposal = GovernanceProposal {
            proposer: proposer.clone(),
            action,
            status: ProposalStatus::Approved,
            votes_for: 10,
            votes_against: 0,
        };
        
        Proposals::<T>::insert(0, proposal);
        
        let origin = RawOrigin::Root;
        let initial_balance = T::Currency::free_balance(target);

        #[extrinsic_call]
        execute_proposal(origin, 0);

        let executed_proposal = Proposals::<T>::get(0).unwrap();
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
        
        // Verify balance transfer occurred (if implemented)
        let final_balance = T::Currency::free_balance(target);
        assert!(final_balance >= initial_balance);
    }

    /// Benchmark epoch auto-transition with validator set changes
    #[benchmark]
    fn epoch_auto_transition() {
        let validators = setup_validators::<T>(10);
        
        // Set up epoch config for auto-transition
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 100,
        };
        EpochConfigStorage::<T>::put(epoch_config);
        
        // Add some pending validator actions
        for (i, validator) in validators.iter().enumerate() {
            if i < 3 {
                PendingValidatorActions::<T>::insert(validator, ValidatorAction::Join);
            } else if i < 6 {
                PendingValidatorActions::<T>::insert(validator, ValidatorAction::Leave);
            }
        }
        
        let initial_epoch = CurrentEpoch::<T>::get();
        let block_number = (initial_epoch + 1) * 100 + 1; // Trigger epoch transition

        #[block]
        {
            // Simulate epoch auto-transition
            DcfPallet::<T>::on_initialize(block_number.into());
        }

        // Verify epoch was advanced and actions were processed
        assert!(CurrentEpoch::<T>::get() > initial_epoch);
    }

    /// Benchmark complex validator lifecycle operations
    #[benchmark]
    fn validator_lifecycle_operations(v: Linear<5, 30>) {
        let validators = setup_validators::<T>(v);
        
        // Set up various validator states and actions
        for (i, validator) in validators.iter().enumerate() {
            // Update scores
            let mut state = ValidatorStates::<T>::get(validator).unwrap();
            state.current.stake_score = (1000 + i as u64 * 100) % 2000;
            state.current.inference_score = (800 + i as u64 * 50) % 1500;
            state.current.authored_blocks = (i as u32 * 10) % 100;
            state.current.missed_blocks = (i as u32 * 2) % 20;
            ValidatorStates::<T>::insert(validator, state);
            
            // Add some pending actions
            if i % 3 == 0 {
                PendingValidatorActions::<T>::insert(validator, ValidatorAction::Join);
            } else if i % 3 == 1 {
                PendingValidatorActions::<T>::insert(validator, ValidatorAction::Leave);
            }
        }

        #[block]
        {
            // Process various lifecycle operations
            for validator in &validators {
                // Update scores
                let _ = DcfPallet::<T>::update_validator_stake_score(
                    RawOrigin::Signed(validator.clone()).into(),
                    validator.clone()
                );
                
                // Check validator status
                let _ = DcfPallet::<T>::is_validator_active(validator);
                let _ = DcfPallet::<T>::validator_states(validator);
            }
            
            // Process epoch transition
            let current_epoch = CurrentEpoch::<T>::get();
            CurrentEpoch::<T>::put(current_epoch + 1);
        }

        // Verify operations completed
        assert_eq!(validators.len(), v as usize);
    }

    /// Benchmark misbehavior reporting and slashing
    #[benchmark]
    fn misbehavior_reporting_and_slashing() {
        let validators = setup_validators::<T>(5);
        let reporter = &validators[0];
        let misbehaving_validator = &validators[1];
        
        // Give the misbehaving validator some balance
        T::Currency::make_free_balance_be(misbehaving_validator, 10000u32.into());
        let _ = T::Currency::reserve(misbehaving_validator, 5000u32.into());
        
        let evidence = b"Misbehavior evidence data".to_vec().try_into().unwrap();
        let origin = RawOrigin::Signed(reporter.clone());

        #[extrinsic_call]
        report_validator_misbehavior(origin, misbehaving_validator.clone(), evidence);

        // Verify misbehavior was reported
        assert!(MisbehaviorReports::<T>::contains_key(misbehaving_validator, reporter));
    }

    /// Benchmark validator stake increase
    #[benchmark]
    fn increase_validator_stake() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());
        
        // Give validator some balance
        T::Currency::make_free_balance_be(&validator, 20000u32.into());
        let additional_stake = 5000u32.into();

        #[extrinsic_call]
        increase_validator_stake(origin, additional_stake);

        // Verify stake was increased
        let reserved = T::Currency::reserved_balance(&validator);
        assert!(reserved >= additional_stake);
    }

    /// Benchmark validator stake decrease
    #[benchmark]
    fn decrease_validator_stake() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());
        
        // Give validator balance and reserve some
        T::Currency::make_free_balance_be(&validator, 20000u32.into());
        let _ = T::Currency::reserve(&validator, 10000u32.into());
        let decrease_amount = 2000u32.into();

        #[extrinsic_call]
        decrease_validator_stake(origin, decrease_amount);

        // Verify stake was decreased
        let reserved = T::Currency::reserved_balance(&validator);
        assert!(reserved < 10000u32.into());
    }

    /// Benchmark multiple validator slashing
    #[benchmark]
    fn slash_multiple_validators(v: Linear<2, 10>) {
        let validators = setup_validators::<T>(v);
        let origin = RawOrigin::Root;
        
        // Give all validators some balance to slash
        for validator in &validators {
            T::Currency::make_free_balance_be(validator, 20000u32.into());
            let _ = T::Currency::reserve(validator, 10000u32.into());
        }
        
        let slash_amount = 1000u32.into();

        #[extrinsic_call]
        slash_multiple_validators(origin, validators.clone(), slash_amount);

        // Verify all validators were slashed
        for validator in &validators {
            let reserved = T::Currency::reserved_balance(validator);
            assert!(reserved < 10000u32.into());
        }
    }

    /// Benchmark percentage-based validator slashing
    #[benchmark]
    fn slash_validator_percentage() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Root;
        
        // Give validator some balance to slash
        T::Currency::make_free_balance_be(&validator, 20000u32.into());
        let _ = T::Currency::reserve(&validator, 10000u32.into());
        let slash_percentage = 20u32; // 20%

        #[extrinsic_call]
        slash_validator_percentage(origin, validator.clone(), slash_percentage);

        // Verify validator was slashed by percentage
        let reserved = T::Currency::reserved_balance(&validator);
        assert!(reserved < 10000u32.into());
    }

    /// Benchmark validator metadata setting
    #[benchmark]
    fn set_validator_metadata() {
        let validator = create_validator::<T>(0);
        let origin = RawOrigin::Signed(validator.clone());
        
        let name = b"Test Validator".to_vec();
        let website = Some(b"https://example.com".to_vec());
        let contact = Some(b"test@example.com".to_vec());
        let description = Some(b"A test validator for benchmarking".to_vec());
        let location = Some(b"Test Location".to_vec());

        #[extrinsic_call]
        set_validator_metadata(origin, name, website, contact, description, location, None, None);

        // Verify metadata was set
        assert!(ValidatorMetadata::<T>::contains_key(&validator));
    }

    /// Benchmark reward proposal for multiple validators
    #[benchmark]
    fn propose_reward_multiple_validators(v: Linear<2, 10>) {
        let validators = setup_validators::<T>(v + 1);
        let proposer = &validators[0];
        let reward_validators = validators[1..].to_vec();
        let origin = RawOrigin::Root;
        
        GovernanceModeEnabled::<T>::put(true);
        let reward_amount = 1000u32.into();

        #[extrinsic_call]
        propose_reward_multiple_validators(origin, proposer.clone(), reward_validators.clone(), reward_amount);

        // Verify proposal was created
        assert!(Proposals::<T>::contains_key(0));
    }

    impl_benchmark_test_suite!(DcfPallet, crate::mock::new_test_ext(), crate::mock::Test);
}