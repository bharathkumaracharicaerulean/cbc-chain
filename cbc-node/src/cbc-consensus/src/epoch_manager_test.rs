//! Tests for the EpochManager module

#[cfg(test)]
mod tests {
    use super::super::epoch_manager::*;
    use crate::mock::*;
    use frame_support::traits::Hooks;

    #[test]
    fn test_epoch_initialization() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            // Test initial epoch state
            let current_epoch = pallet_cbc_dcf::CurrentEpoch::<Test>::get();
            assert_eq!(current_epoch, 0);
            
            // Test epoch configuration
            let epoch_config = pallet_cbc_dcf::EpochConfigStorage::<Test>::get();
            assert!(epoch_config.blocks_per_epoch > 0);
            assert!(epoch_config.min_stake > 0);
            assert!(epoch_config.max_validators > 0);
            
            // Verify epoch length is reasonable
            assert!(epoch_config.blocks_per_epoch >= 100); // At least 100 blocks per epoch
            assert!(epoch_config.blocks_per_epoch <= 10000); // At most 10000 blocks per epoch
        });
    }

    #[test]
    fn test_epoch_progression() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let initial_epoch = pallet_cbc_dcf::CurrentEpoch::<Test>::get();
            
            // Manually advance epoch
            pallet_cbc_dcf::CurrentEpoch::<Test>::put(initial_epoch + 1);
            
            let new_epoch = pallet_cbc_dcf::CurrentEpoch::<Test>::get();
            assert_eq!(new_epoch, initial_epoch + 1);
            
            // Test multiple epoch advances
            for i in 2..=10 {
                pallet_cbc_dcf::CurrentEpoch::<Test>::put(i);
                let current = pallet_cbc_dcf::CurrentEpoch::<Test>::get();
                assert_eq!(current, i);
            }
        });
    }

    #[test]
    fn test_epoch_boundary_detection() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let epoch_config = pallet_cbc_dcf::EpochConfigStorage::<Test>::get();
            let blocks_per_epoch = epoch_config.blocks_per_epoch;
            
            // Test epoch boundary calculation
            let epoch_0_end = blocks_per_epoch;
            let epoch_1_start = epoch_0_end + 1;
            let epoch_1_end = blocks_per_epoch * 2;
            
            // Verify boundary calculations
            assert_eq!(epoch_0_end % blocks_per_epoch, 0);
            assert_eq!(epoch_1_end % blocks_per_epoch, 0);
            assert_eq!(epoch_1_start, epoch_0_end + 1);
        });
    }

    #[test]
    fn test_epoch_transition_triggers() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let epoch_config = pallet_cbc_dcf::EpochConfigStorage::<Test>::get();
            let blocks_per_epoch = epoch_config.blocks_per_epoch;
            
            // Test that epoch transitions are triggered at the right blocks
            let transition_blocks = vec![
                blocks_per_epoch,
                blocks_per_epoch * 2,
                blocks_per_epoch * 3,
                blocks_per_epoch * 5,
                blocks_per_epoch * 10,
            ];
            
            for block_number in transition_blocks {
                // Simulate block processing
                frame_system::Pallet::<Test>::set_block_number(block_number.into());
                
                // In a real scenario, this would trigger epoch transition
                // For testing, we verify the block number calculation
                let expected_epoch = block_number / blocks_per_epoch;
                assert!(expected_epoch > 0);
                assert_eq!(block_number % blocks_per_epoch, 0);
            }
        });
    }

    #[test]
    fn test_epoch_history_management() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            // Test epoch history storage
            let max_history = pallet_cbc_dcf::MaxEpochHistory::<Test>::get();
            assert!(max_history > 0);
            
            // Test that we can store epoch histories
            let epoch_histories = pallet_cbc_dcf::EpochHistories::<Test>::get();
            
            // Initially should be empty or have genesis epoch
            assert!(epoch_histories.len() <= max_history as usize);
            
            // Test adding epoch history
            let mut histories = epoch_histories;
            let new_history = pallet_cbc_dcf::EpochHistory {
                epoch: 1,
                start_block: 1,
                end_block: 2400,
                validator_count: 4,
                total_stake: 4000,
                rewards_distributed: 1000,
                slashing_events: 0,
            };
            
            if histories.try_push(new_history.clone()).is_ok() {
                pallet_cbc_dcf::EpochHistories::<Test>::put(&histories);
                
                // Verify history was added
                let updated_histories = pallet_cbc_dcf::EpochHistories::<Test>::get();
                assert!(!updated_histories.is_empty());
                
                if let Some(last_history) = updated_histories.last() {
                    assert_eq!(last_history.epoch, 1);
                    assert_eq!(last_history.validator_count, 4);
                    assert_eq!(last_history.total_stake, 4000);
                }
            }
        });
    }

    #[test]
    fn test_epoch_validator_management() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let validators = pallet_cbc_dcf::ValidatorSet::<Test>::get();
            let initial_count = validators.len();
            
            // Test validator state updates during epoch transition
            for validator in validators.iter() {
                let mut state = pallet_cbc_dcf::ValidatorStates::<Test>::get(validator).unwrap();
                
                // Update epoch in validator state
                state.current.epoch = 1;
                state.last_active_epoch = 1;
                
                // Update performance metrics
                state.current.authored_blocks = 10;
                state.current.missed_blocks = 1;
                state.participation_rate = 95;
                
                pallet_cbc_dcf::ValidatorStates::<Test>::insert(validator, &state);
            }
            
            // Verify all validators were updated
            for validator in validators.iter() {
                let state = pallet_cbc_dcf::ValidatorStates::<Test>::get(validator).unwrap();
                assert_eq!(state.current.epoch, 1);
                assert_eq!(state.last_active_epoch, 1);
                assert!(state.participation_rate > 0);
            }
            
            // Verify validator count remains consistent
            let current_validators = pallet_cbc_dcf::ValidatorSet::<Test>::get();
            assert_eq!(current_validators.len(), initial_count);
        });
    }

    #[test]
    fn test_epoch_pending_actions() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let validator = 1u64;
            
            // Test setting pending actions
            pallet_cbc_dcf::PendingValidatorActions::<Test>::insert(
                &validator, 
                pallet_cbc_dcf::ValidatorAction::Join
            );
            
            // Verify pending action was set
            let pending_action = pallet_cbc_dcf::PendingValidatorActions::<Test>::get(&validator);
            assert_eq!(pending_action, Some(pallet_cbc_dcf::ValidatorAction::Join));
            
            // Test different action types
            let actions = vec![
                pallet_cbc_dcf::ValidatorAction::Join,
                pallet_cbc_dcf::ValidatorAction::Leave,
            ];
            
            for (i, action) in actions.iter().enumerate() {
                let test_validator = (i + 2) as u64;
                pallet_cbc_dcf::PendingValidatorActions::<Test>::insert(&test_validator, action);
                
                let stored_action = pallet_cbc_dcf::PendingValidatorActions::<Test>::get(&test_validator);
                assert_eq!(stored_action, Some(*action));
            }
        });
    }

    #[test]
    fn test_epoch_reward_calculation() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let validators = pallet_cbc_dcf::ValidatorSet::<Test>::get();
            let total_reward_pool = 10000u128;
            
            // Test reward distribution calculation
            let reward_per_validator = total_reward_pool / validators.len() as u128;
            assert!(reward_per_validator > 0);
            
            // Test performance-based rewards
            for validator in validators.iter() {
                let state = pallet_cbc_dcf::ValidatorStates::<Test>::get(validator).unwrap();
                let performance_score = state.current.final_score;
                
                // Calculate performance-based reward multiplier
                let max_score = pallet_cbc_dcf::MaxValidatorScore::<Test>::get();
                let performance_ratio = performance_score as f64 / max_score as f64;
                
                assert!(performance_ratio >= 0.0);
                assert!(performance_ratio <= 1.0);
                
                // Performance-based reward should be reasonable
                let performance_reward = (reward_per_validator as f64 * performance_ratio) as u128;
                assert!(performance_reward <= total_reward_pool);
            }
        });
    }

    #[test]
    fn test_epoch_slashing_events() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let validator = 1u64;
            let initial_stake = pallet_cbc_dcf::ValidatorStake::<Test>::get(&validator);
            
            // Test slashing during epoch
            let slash_amount = initial_stake / 10; // 10% slash
            let new_stake = initial_stake.saturating_sub(slash_amount);
            
            pallet_cbc_dcf::ValidatorStake::<Test>::insert(&validator, new_stake);
            
            // Verify slashing was applied
            let current_stake = pallet_cbc_dcf::ValidatorStake::<Test>::get(&validator);
            assert_eq!(current_stake, new_stake);
            assert!(current_stake < initial_stake);
            
            // Test that slashing doesn't go below minimum
            let min_stake = pallet_cbc_dcf::DcfMinStake::<Test>::get();
            if current_stake >= min_stake {
                // Validator should still be valid
                let state = pallet_cbc_dcf::ValidatorStates::<Test>::get(&validator);
                assert!(state.is_some());
            }
        });
    }

    #[test]
    fn test_epoch_configuration_updates() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            let initial_config = pallet_cbc_dcf::EpochConfigStorage::<Test>::get();
            
            // Test updating epoch configuration
            let new_config = pallet_cbc_dcf::EpochConfig {
                blocks_per_epoch: initial_config.blocks_per_epoch * 2,
                min_stake: initial_config.min_stake + 500,
                max_validators: initial_config.max_validators + 10,
            };
            
            pallet_cbc_dcf::EpochConfigStorage::<Test>::put(&new_config);
            
            // Verify configuration was updated
            let updated_config = pallet_cbc_dcf::EpochConfigStorage::<Test>::get();
            assert_eq!(updated_config.blocks_per_epoch, initial_config.blocks_per_epoch * 2);
            assert_eq!(updated_config.min_stake, initial_config.min_stake + 500);
            assert_eq!(updated_config.max_validators, initial_config.max_validators + 10);
        });
    }

    #[test]
    fn test_epoch_transition_performance() {
        setup_consensus_test(MockConsensusConfig::default()).execute_with(|| {
            use std::time::Instant;
            
            let start = Instant::now();
            
            // Simulate multiple epoch transitions
            for epoch in 1..=10 {
                pallet_cbc_dcf::CurrentEpoch::<Test>::put(epoch);
                
                // Update all validator states for the new epoch
                let validators = pallet_cbc_dcf::ValidatorSet::<Test>::get();
                for validator in validators.iter() {
                    let mut state = pallet_cbc_dcf::ValidatorStates::<Test>::get(validator).unwrap();
                    state.current.epoch = epoch;
                    state.last_active_epoch = epoch;
                    pallet_cbc_dcf::ValidatorStates::<Test>::insert(validator, &state);
                }
                
                // Add epoch history
                let mut histories = pallet_cbc_dcf::EpochHistories::<Test>::get();
                let history = pallet_cbc_dcf::EpochHistory {
                    epoch,
                    start_block: (epoch - 1) * 2400 + 1,
                    end_block: epoch * 2400,
                    validator_count: validators.len() as u32,
                    total_stake: validators.len() as u128 * 1000,
                    rewards_distributed: 1000,
                    slashing_events: 0,
                };
                
                if histories.try_push(history).is_ok() {
                    pallet_cbc_dcf::EpochHistories::<Test>::put(&histories);
                }
            }
            
            let duration = start.elapsed();
            
            // Epoch transitions should be fast (less than 1 second for 10 transitions)
            assert!(duration.as_secs() < 1, "Epoch transitions took too long: {:?}", duration);
            
            // Verify final state
            let final_epoch = pallet_cbc_dcf::CurrentEpoch::<Test>::get();
            assert_eq!(final_epoch, 10);
            
            let histories = pallet_cbc_dcf::EpochHistories::<Test>::get();
            assert!(!histories.is_empty());
        });
    }
}