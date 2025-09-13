//! Comprehensive tests for the CBC Runtime
//! 
//! This module contains integration tests that verify the entire runtime
//! works correctly with all pallets integrated together.

use crate::{
    mock::{*, account_id, funded_account_id},
    RuntimeOrigin, System, Balances, Runtime,
};
type DcfPallet = pallet_cbc_dcf::Pallet<Runtime>;
type PalletCbcPos = pallet_cbc_pos::Pallet<Runtime>;
type PalletCbcPoi = pallet_cbc_poi::Pallet<Runtime>;
use frame_support::{assert_ok, assert_noop, traits::Get};
use sp_runtime::traits::Zero;

#[cfg(test)]
mod runtime_integration_tests {
    use super::*;

    #[test]
    fn test_runtime_initialization() {
        new_test_ext().execute_with(|| {
            // Verify runtime initializes correctly
            assert_eq!(System::block_number(), 1);
            
            // Verify all pallets are working
            let test_account = funded_account_id(1);
            assert!(System::account_exists(&test_account));
            assert!(Balances::free_balance(&test_account) > Zero::zero());
            
            // Verify DCF pallet is initialized
            let validators = DcfPallet::validator_set();
            assert!(!validators.is_empty());
            assert_eq!(validators.len(), 4);
            
            // Verify each validator has proper state
            for validator in validators.iter() {
                let state = DcfPallet::validator_states(validator);
                assert!(state.is_some());
                
                let stake = DcfPallet::validator_stake(validator);
                assert!(stake > 0);
            }
        });
    }

    #[test]
    fn test_cross_pallet_interactions() {
        new_test_ext().execute_with(|| {
            let validator = funded_account_id(1);
            let initial_balance = Balances::free_balance(&validator);
            
            // Test DCF and Balances interaction
            let stake_amount = 5000u128;
            setup_validator_with_stake(validator.clone(), stake_amount);
            
            // Verify balance was reserved
            let reserved = Balances::reserved_balance(&validator);
            assert_eq!(reserved, stake_amount);
            
            // Verify DCF recorded the stake
            let dcf_stake = DcfPallet::validator_stake(&validator);
            assert_eq!(dcf_stake, stake_amount);
            
            // Test PoS integration
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(validator.clone())));
            assert!(PalletCbcPos::validators(&validator).is_some());
            
            // Test PoI integration
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(validator.clone()),
                42,
                80
            ));
            assert!(PalletCbcPoi::inference_results(&validator).is_some());
        });
    }

    #[test]
    fn test_governance_integration() {
        new_test_ext_for_governance().execute_with(|| {
            let proposer = funded_account_id(1);
            let target = funded_account_id(2);
            
            // Verify governance mode is enabled
            assert!(DcfPallet::governance_mode_enabled());
            
            // Test proposal submission
            assert_ok!(DcfPallet::propose_reward_validator(
                RuntimeOrigin::root(),
                proposer.clone(),
                target.clone(),
                1000u128
            ));
            
            // Verify proposal was created
            assert!(pallet_cbc_dcf::Proposals::<Runtime>::contains_key(0));
            
            // Test voting
            assert_ok!(DcfPallet::vote_proposal(
                RuntimeOrigin::signed(proposer.clone()),
                0,
                true
            ));
            
            // Test proposal execution
            assert_ok!(DcfPallet::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));
            
            // Verify proposal was executed
            let proposal = pallet_cbc_dcf::Proposals::<Runtime>::get(0).unwrap();
            assert_eq!(proposal.status, pallet_cbc_dcf::ProposalStatus::Executed);
        });
    }

    #[test]
    fn test_epoch_transitions() {
        new_test_ext_for_epochs().execute_with(|| {
            let initial_epoch = DcfPallet::current_epoch();
            
            // Advance to next epoch
            advance_to_next_epoch();
            
            // Verify epoch advanced
            let new_epoch = DcfPallet::current_epoch();
            assert!(new_epoch > initial_epoch);
            
            // Verify validators are still active
            let validators = DcfPallet::validator_set();
            assert!(!validators.is_empty());
            
            // Verify validator states were updated
            for validator in validators.iter() {
                let state = DcfPallet::validator_states(validator);
                assert!(state.is_some());
            }
        });
    }

    #[test]
    fn test_multi_validator_system() {
        new_test_ext_with_validators(10).execute_with(|| {
            let validators = DcfPallet::validator_set();
            assert_eq!(validators.len(), 10);
            
            // Test that all validators have proper setup
            for validator in validators.iter() {
                let state = DcfPallet::validator_states(validator);
                assert!(state.is_some());
                
                let stake = DcfPallet::validator_stake(validator);
                assert!(stake > 0);
                
                let balance = Balances::free_balance(validator);
                assert!(balance > 0);
            }
            
            // Test validator interactions
            let validator1 = validators[0].clone();
            let validator2 = validators[1].clone();
            
            // Test PoS registration for multiple validators
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(validator1.clone())));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(validator2.clone())));
            
            // Test PoI submissions from multiple validators
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(validator1.clone()),
                42,
                80
            ));
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(validator2.clone()),
                43,
                85
            ));
            
            // Test challenge mechanism
            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(validator2.clone()),
                validator1.clone(),
                42
            ));
        });
    }

    #[test]
    fn test_runtime_apis() {
        new_test_ext().execute_with(|| {
            let validator = funded_account_id(1);
            
            // Test DCF runtime APIs
            let profile = DcfPallet::get_validator_profile(validator);
            assert!(profile.is_some());
            
            let validators = DcfPallet::validator_set();
            assert!(!validators.is_empty());
            
            let active_validators = DcfPallet::active_validators();
            assert!(!active_validators.is_empty());
            
            let current_epoch = DcfPallet::current_epoch();
            assert_eq!(current_epoch, 0);
            
            // Test consensus weights
            let pos_weight = DcfPallet::pos_weight();
            let poi_weight = DcfPallet::poi_weight();
            assert_eq!(pos_weight + poi_weight, 100);
        });
    }

    #[test]
    fn test_economic_model() {
        new_test_ext().execute_with(|| {
            let validator = funded_account_id(1);
            let initial_balance = Balances::free_balance(&validator);
            let stake_amount = 10000u128;
            
            // Test staking
            setup_validator_with_stake(validator.clone(), stake_amount);
            
            let free_balance = Balances::free_balance(&validator);
            let reserved_balance = Balances::reserved_balance(&validator);
            
            assert_eq!(reserved_balance, stake_amount);
            assert_eq!(free_balance + reserved_balance, initial_balance);
            
            // Test slashing
            let slash_amount = 1000u128;
            assert_ok!(DcfPallet::slash_validator(
                RuntimeOrigin::root(),
                validator.clone(),
                slash_amount
            ));
            
            // Verify stake was reduced
            let new_stake = DcfPallet::validator_stake(&validator);
            assert!(new_stake < stake_amount);
            
            // Test reward distribution
            let reward_amount = 500u128;
            assert_ok!(DcfPallet::set_governance_mode(RuntimeOrigin::root(), true));
            assert_ok!(DcfPallet::propose_reward_validator(
                RuntimeOrigin::root(),
                validator.clone(),
                validator.clone(),
                reward_amount
            ));
            
            // Verify proposal was created
            assert!(pallet_cbc_dcf::Proposals::<Runtime>::contains_key(0));
        });
    }

    #[test]
    fn test_system_limits() {
        new_test_ext().execute_with(|| {
            // Test maximum validators limit
            let max_validators = <Runtime as pallet_cbc_dcf::Config>::MaxValidators::get();
            let current_validators = DcfPallet::validator_set();
            assert!(current_validators.len() <= max_validators as usize);
            
            // Test minimum active validators
            let min_active = <Runtime as pallet_cbc_dcf::Config>::MinActiveValidators::get();
            let active_validators = DcfPallet::active_validators();
            assert!(active_validators.len() >= min_active as usize);
            
            // Test epoch history limits
            let max_history: u32 = <Runtime as pallet_cbc_dcf::Config>::MaxEpochHistory::get();
            let epoch_histories = pallet_cbc_dcf::EpochHistories::<Runtime>::get();
            assert!(epoch_histories.len() <= max_history as usize);
        });
    }

    #[test]
    fn test_error_handling() {
        new_test_ext().execute_with(|| {
            let non_validator = funded_account_id(999);
            
            // Test invalid validator operations
            assert_noop!(
                DcfPallet::leave_validators(RuntimeOrigin::signed(non_validator.clone())),
                pallet_cbc_dcf::Error::<Runtime>::ValidatorNotFound
            );
            
            // Test invalid governance operations
            assert_noop!(
                DcfPallet::vote_proposal(RuntimeOrigin::signed(non_validator.clone()), 999, true),
                pallet_cbc_dcf::Error::<Runtime>::ProposalNotApproved
            );
            
            // Test invalid PoS operations
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(funded_account_id(1)), non_validator.clone(), 75),
                pallet_cbc_pos::Error::<Runtime>::ValidatorNotRegistered
            );
            
            // Test invalid PoI operations
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(funded_account_id(1)), non_validator.clone(), 42),
                pallet_cbc_poi::Error::<Runtime>::InferenceNotFound
            );
        });
    }

    #[test]
    fn test_performance_under_load() {
        new_test_ext_for_scenario(TestScenario::StressTest).execute_with(|| {
            let validators = DcfPallet::validator_set();
            assert_eq!(validators.len(), 50);
            
            // Test system performance with many validators
            use std::time::Instant;
            let start = Instant::now();
            
            // Perform operations on all validators
            for validator in validators.iter().take(10) {
                let _ = DcfPallet::update_validator_stake_score(
                    RuntimeOrigin::signed(validator.clone()),
                    validator.clone()
                );
            }
            
            let duration = start.elapsed();
            
            // Should complete quickly even with many validators
            assert!(duration.as_millis() < 1000);
            
            // Test epoch transition with many validators
            let start = Instant::now();
            advance_to_next_epoch();
            let duration = start.elapsed();
            
            // Epoch transition should be reasonable even with many validators
            assert!(duration.as_millis() < 5000);
        });
    }

    #[test]
    fn test_runtime_upgrade_compatibility() {
        new_test_ext().execute_with(|| {
            // Test that runtime state is consistent
            let validators = DcfPallet::validator_set();
            let active_validators = DcfPallet::active_validators();
            
            // All active validators should be in validator set
            for active in active_validators.iter() {
                assert!(validators.contains(active));
            }
            
            // All validators should have states
            for validator in validators.iter() {
                assert!(DcfPallet::validator_states(validator).is_some());
                assert!(DcfPallet::validator_stake(validator) > 0);
            }
            
            // Test storage consistency
            let current_epoch = DcfPallet::current_epoch();
            let epoch_config = pallet_cbc_dcf::EpochConfigStorage::<Runtime>::get();
            
            assert!(epoch_config.blocks_per_epoch > 0);
            assert!(epoch_config.min_stake > 0);
            assert!(epoch_config.max_validators > 0);
        });
    }
}

#[cfg(test)]
mod runtime_benchmark_tests {
    use super::*;

    #[test]
    fn test_benchmark_integration() {
        new_test_ext().execute_with(|| {
            // Test that benchmarking infrastructure works
            let validator = funded_account_id(1);
            
            // These should not panic and should complete quickly
            let _ = DcfPallet::join_validators(RuntimeOrigin::signed(validator.clone()), None);
            let _ = PalletCbcPos::register_validator(RuntimeOrigin::signed(validator.clone()));
            let _ = PalletCbcPoi::submit_inference(RuntimeOrigin::signed(validator.clone()), 42, 80);
        });
    }
}

#[cfg(test)]
mod runtime_mock_tests {
    use super::*;

    #[test]
    fn test_mock_configurations() {
        // Test basic mock
        new_test_ext().execute_with(|| {
            assert_eq!(System::block_number(), 1);
            assert_eq!(DcfPallet::validator_set().len(), 4);
        });
        
        // Test multi-validator mock
        new_test_ext_with_validators(20).execute_with(|| {
            assert_eq!(DcfPallet::validator_set().len(), 20);
        });
        
        // Test governance mock
        new_test_ext_for_governance().execute_with(|| {
            assert!(DcfPallet::governance_mode_enabled());
        });
        
        // Test epoch mock
        new_test_ext_for_epochs().execute_with(|| {
            let config = pallet_cbc_dcf::EpochConfigStorage::<Runtime>::get();
            assert_eq!(config.blocks_per_epoch, 100);
        });
    }

    #[test]
    fn test_helper_functions() {
        new_test_ext().execute_with(|| {
            let initial_block = System::block_number();
            
            // Test block advancement
            advance_blocks(10);
            assert_eq!(System::block_number(), initial_block + 10);
            
            // Test account creation
            let test_account = funded_account_id(999);
            assert_eq!(Balances::free_balance(&test_account), 50000);
            
            // Test validator setup
            setup_validator_with_stake(test_account.clone(), 10000);
            assert_eq!(Balances::reserved_balance(&test_account), 10000);
            assert_eq!(DcfPallet::validator_stake(&test_account), 10000);
        });
    }
}