//! Industrial, production-grade unit test suite for `pallet-cbc-pos`
//!
//! Organised into 6 modular submodules:
//! 1. Genesis Configuration Tests
//! 2. Validator Registration & Activation Management Tests
//! 3. Stake Bonding, Unbonding & Reserve Adjustment Extrinsics
//! 4. Scoring (Submit, Boost, Slash) & Slashing Extrinsics
//! 5. Reward Distribution & Epoch Accounting Extrinsics
//! 6. Helper Functions & Economic Metrics Tests

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        EpochTotalRewarded, EpochTotalSlashed, Error, Event, GenesisConfig,
        ValidatorEpochRewarded, ValidatorEpochSlashed, Stake, Validators, ValidatorScores, SlashingCount,
    };
    use frame_support::{assert_noop, assert_ok, traits::Get};
    use sp_runtime::{BuildStorage, DispatchError};

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    // ============================================================================
    // 1. Genesis Configuration Tests
    // ============================================================================
    mod genesis_tests {
        use super::*;

        #[test]
        fn test_genesis_config_initialization() {
            let mut storage = frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap();

            GenesisConfig::<Test> {
                validators: vec![1, 2],
                validator_scores: vec![100, 80],
                current_epoch: 5,
                slashing_count: vec![(1, 1)],
            }
            .assimilate_storage(&mut storage)
            .unwrap();

            let mut ext = sp_io::TestExternalities::from(storage);
            ext.execute_with(|| {
                assert_eq!(PalletCbcPos::validators(1), Some(true));
                assert_eq!(PalletCbcPos::validators(2), Some(true));
                assert_eq!(PalletCbcPos::validator_scores(1), Some(100));
                assert_eq!(PalletCbcPos::validator_scores(2), Some(80));
                assert_eq!(PalletCbcPos::current_epoch(), 5);
                assert_eq!(PalletCbcPos::slashing_count(1), Some(1));
            });
        }
    }

    // ============================================================================
    // 2. Validator Registration & Activation Management Tests
    // ============================================================================
    mod registration_tests {
        use super::*;

        #[test]
        fn test_register_validator_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_eq!(PalletCbcPos::validators(1), Some(true));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorRegistered {
                    validator: 1,
                }));
            });
        }

        #[test]
        fn test_register_validator_already_registered() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_noop!(
                    PalletCbcPos::register_validator(RuntimeOrigin::signed(1)),
                    Error::<Test>::ValidatorAlreadyRegistered
                );
            });
        }

        #[test]
        fn test_register_validator_max_limit() {
            new_test_ext().execute_with(|| {
                let max_validators: u32 = <Test as crate::Config>::MaxValidators::get();
                for i in 1..=max_validators {
                    assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                        i as u64
                    )));
                }
                assert_noop!(
                    PalletCbcPos::register_validator(RuntimeOrigin::signed(
                        (max_validators + 1) as u64
                    )),
                    Error::<Test>::TooManyValidators
                );
            });
        }

        #[test]
        fn test_register_validator_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::register_validator(RuntimeOrigin::root()),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_activate_and_deactivate_validator_internal() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

                assert_ok!(PalletCbcPos::deactivate_validator(&1));
                assert_eq!(PalletCbcPos::validators(1), Some(false));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorDeactivated {
                    validator: 1,
                }));

                assert_ok!(PalletCbcPos::activate_validator(&1));
                assert_eq!(PalletCbcPos::validators(1), Some(true));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorActivated {
                    validator: 1,
                }));
            });
        }

        #[test]
        fn test_activate_validator_unregistered() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::activate_validator(&999),
                    Error::<Test>::ValidatorNotRegistered
                );
            });
        }

        #[test]
        fn test_deactivate_validator_unregistered() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::deactivate_validator(&999),
                    Error::<Test>::ValidatorNotRegistered
                );
            });
        }
    }

    // ============================================================================
    // 3. Stake Bonding, Unbonding & Reserve Adjustment Extrinsics
    // ============================================================================
    mod stake_tests {
        use super::*;

        #[test]
        fn test_bond_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_eq!(PalletCbcPos::stake(validator), min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeBonded {
                    validator,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_bond_stake_insufficient_stake() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_noop!(
                    PalletCbcPos::bond_stake(RuntimeOrigin::signed(validator), min_stake - 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_bond_stake_not_registered() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_noop!(
                    PalletCbcPos::bond_stake(RuntimeOrigin::signed(1), min_stake),
                    Error::<Test>::ValidatorNotRegistered
                );
            });
        }

        #[test]
        fn test_unbond_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake * 2
                ));
                assert_ok!(PalletCbcPos::unbond_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_eq!(PalletCbcPos::stake(validator), min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeUnbonded {
                    validator,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_unbond_stake_below_minimum() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_noop!(
                    PalletCbcPos::unbond_stake(RuntimeOrigin::signed(validator), 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_unbond_stake_excessive() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_noop!(
                    PalletCbcPos::unbond_stake(RuntimeOrigin::signed(validator), min_stake + 100),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_increase_validator_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_eq!(Stake::<Test>::get(&validator), min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeIncreased {
                    validator,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_increase_validator_stake_not_registered() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::increase_validator_stake(RuntimeOrigin::signed(999), 1000),
                    Error::<Test>::ValidatorNotInSet
                );
            });
        }

        #[test]
        fn test_decrease_validator_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake * 3
                ));
                assert_ok!(PalletCbcPos::decrease_validator_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_eq!(Stake::<Test>::get(&validator), min_stake * 2);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeDecreased {
                    validator,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_decrease_validator_stake_below_min() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(validator),
                    min_stake
                ));
                assert_noop!(
                    PalletCbcPos::decrease_validator_stake(RuntimeOrigin::signed(validator), 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }
    }

    // ============================================================================
    // 4. Scoring & Slashing Extrinsics Tests
    // ============================================================================
    mod scoring_and_slashing_tests {
        use super::*;

        #[test]
        fn test_submit_score_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                let min_score: u32 = <Test as crate::Config>::MinValidatorScore::get();
                assert_ok!(PalletCbcPos::submit_score(
                    RuntimeOrigin::signed(validator),
                    validator,
                    min_score + 10
                ));
                assert_eq!(
                    ValidatorScores::<Test>::get(&validator),
                    Some(min_score + 10)
                );
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSubmitted {
                    validator,
                    score: min_score + 10,
                }));
            });
        }

        #[test]
        fn test_submit_score_too_low() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                let min_score: u32 = <Test as crate::Config>::MinValidatorScore::get();
                assert_noop!(
                    PalletCbcPos::submit_score(
                        RuntimeOrigin::signed(validator),
                        validator,
                        min_score - 1
                    ),
                    Error::<Test>::ScoreTooLow
                );
            });
        }

        #[test]
        fn test_boost_score_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::boost_score(
                    RuntimeOrigin::signed(validator),
                    validator,
                    20
                ));
                assert_eq!(ValidatorScores::<Test>::get(&validator), Some(20));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreBoosted {
                    validator,
                    old_score: 0,
                    new_score: 20,
                    boost_amount: 20,
                }));
            });
        }

        #[test]
        fn test_slash_score_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::boost_score(
                    RuntimeOrigin::signed(validator),
                    validator,
                    50
                ));
                assert_ok!(PalletCbcPos::slash_score(
                    RuntimeOrigin::signed(validator),
                    validator,
                    20
                ));
                assert_eq!(ValidatorScores::<Test>::get(&validator), Some(30));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSlashed {
                    validator,
                    old_score: 50,
                    new_score: 30,
                    slash_amount: 20,
                }));
            });
        }

        #[test]
        fn test_slash_validator_call_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::slash_validator_call(
                    RuntimeOrigin::signed(validator),
                    validator
                ));
                assert_eq!(SlashingCount::<Test>::get(&validator), Some(1));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorSlashed {
                    validator,
                    slashing_count: 1,
                }));
            });
        }

        #[test]
        fn test_slash_validator_call_max_count_removal() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                let max_slash: u32 = <Test as crate::Config>::MaxSlashingCount::get();
                for _ in 0..max_slash {
                    assert_ok!(PalletCbcPos::slash_validator_call(
                        RuntimeOrigin::signed(validator),
                        validator
                    ));
                }
                assert!(!Validators::<Test>::contains_key(&validator));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorRemoved {
                    validator,
                    reason: b"Max slashing count reached".to_vec(),
                }));
            });
        }

        #[test]
        fn test_slash_validator_root_only() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::root(),
                    validator,
                    500
                ));
                assert_noop!(
                    PalletCbcPos::slash_validator(RuntimeOrigin::signed(1), validator, 500),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_slash_validator_percentage_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(validator),
                    10000
                ));
                assert_ok!(PalletCbcPos::slash_validator_percentage(
                    RuntimeOrigin::root(),
                    validator,
                    50
                ));
                assert_noop!(
                    PalletCbcPos::slash_validator_percentage(RuntimeOrigin::root(), validator, 150),
                    Error::<Test>::InvalidStakeAmount
                );
            });
        }

        #[test]
        fn test_slash_multiple_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
                assert_ok!(PalletCbcPos::slash_multiple_validators(
                    RuntimeOrigin::root(),
                    vec![1, 2],
                    100
                ));
            });
        }
    }

    // ============================================================================
    // 5. Reward Distribution & Epoch Accounting Extrinsics Tests
    // ============================================================================
    mod reward_tests {
        use super::*;

        #[test]
        fn test_reward_validator_call_root_only() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::reward_validator_call(
                    RuntimeOrigin::root(),
                    validator,
                    1000
                ));
                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::signed(1), validator, 1000),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_reward_multiple_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
                assert_ok!(PalletCbcPos::reward_multiple_validators(
                    RuntimeOrigin::root(),
                    vec![1, 2],
                    500
                ));
            });
        }

        #[test]
        fn test_reward_all_active_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
                assert_ok!(PalletCbcPos::reward_all_active_validators(
                    RuntimeOrigin::root(),
                    500
                ));
            });
        }

        #[test]
        fn test_distribute_epoch_rewards_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));

                // Distribute 10,000 pool:
                // Active validators = [1, 2] (total 2 validators).
                // Base pool (60%) = 6,000 -> split equally: 3,000 to val 1, 3,000 to val 2.
                // Performance pool (25%) = 2,500 -> val 1 score 90 >= threshold 80 -> 2,500 to val 1.
                // Top performer pool (15%) = 1,500 -> top_performer_count = (2 * 20%) / 100 = 0 (no validator qualifies).
                // Total credited rewards: Val 1 = 3000 + 2500 = 5500; Val 2 = 3000; Total EpochTotalRewarded = 8500.
                // Event reports full pool budget (10,000).
                assert_ok!(PalletCbcPos::distribute_epoch_rewards(
                    RuntimeOrigin::root(),
                    10000
                ));

                assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 5500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(2), 3000);
                assert_eq!(EpochTotalRewarded::<Test>::get(), 8500);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::EpochRewardsDistributed {
                    epoch: 0,
                    total_distributed: 10000,
                }));
            });
        }

        #[test]
        fn test_distribute_epoch_rewards_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::distribute_epoch_rewards(RuntimeOrigin::signed(1), 10000),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_reward_validator_bounds_exceeded_validator() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                let max_reward: u64 = <Test as crate::Config>::MaxRewardPerValidator::get();
                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::root(), validator, max_reward + 1),
                    Error::<Test>::RewardBoundsExceeded
                );
            });
        }

        #[test]
        fn test_reward_validator_bounds_exceeded_epoch() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                let max_epoch_reward: u64 = <Test as crate::Config>::MaxRewardPerEpoch::get();
                EpochTotalRewarded::<Test>::put(max_epoch_reward);
                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::root(), validator, 100),
                    Error::<Test>::RewardBoundsExceeded
                );
            });
        }
    }

    // ============================================================================
    // 6. Helper Functions & Economic Metrics Tests
    // ============================================================================
    mod helper_tests {
        use super::*;

        #[test]
        fn test_calculate_slash_amount() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                    validator
                )));
                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(validator),
                    1000
                ));
                let slash_amount = PalletCbcPos::calculate_slash_amount(&validator).unwrap();
                let slash_percent: u32 = <Test as crate::Config>::SlashPercent::get();
                assert_eq!(slash_amount, (1000 * slash_percent as u64) / 100);
            });
        }

        #[test]
        fn test_calculate_epoch_economic_impact() {
            new_test_ext().execute_with(|| {
                EpochTotalRewarded::<Test>::put(5000);
                EpochTotalSlashed::<Test>::put(2000);
                let (rewarded, slashed) = PalletCbcPos::calculate_epoch_economic_impact(1);
                assert_eq!(rewarded, 5000);
                assert_eq!(slashed, 2000);
            });
        }

        #[test]
        fn test_reset_epoch_totals() {
            new_test_ext().execute_with(|| {
                EpochTotalRewarded::<Test>::put(5000);
                EpochTotalSlashed::<Test>::put(2000);
                ValidatorEpochRewarded::<Test>::insert(1, 1000);
                ValidatorEpochSlashed::<Test>::insert(1, 500);

                PalletCbcPos::reset_epoch_totals();

                assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
                assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 0);
                assert_eq!(ValidatorEpochSlashed::<Test>::get(1), 0);
            });
        }

        #[test]
        fn test_get_active_validators() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
                let active = PalletCbcPos::get_active_validators();
                assert!(active.contains(&1));
                assert!(active.contains(&2));
            });
        }

        #[test]
        fn test_get_slashing_history() {
            new_test_ext().execute_with(|| {
                let validator = 1u64;
                let history = PalletCbcPos::get_slashing_history(validator);
                assert!(history.is_empty());
            });
        }
    }
}