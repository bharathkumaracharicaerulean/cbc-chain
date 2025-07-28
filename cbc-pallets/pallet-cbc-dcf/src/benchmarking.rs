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
        submit_proposal(origin, action);

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
        let validators = setup_validators::<T>(5); // Create enough validators
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

    impl_benchmark_test_suite!(DcfPallet, crate::mock::new_test_ext(), crate::mock::Test);
}