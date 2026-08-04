#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        EpochTotalRewarded, EpochTotalSlashed, Error, Event, GenesisConfig,
        ValidatorEpochRewarded, ValidatorEpochSlashed,
    };
    use frame_support::{assert_noop, assert_ok, traits::Get};
    use sp_runtime::{BuildStorage, DispatchError};

    // Helper function to set block number in system pallet
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    // ================================================================================================
    // 1. Genesis Configuration Tests
    // ================================================================================================

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

    // ================================================================================================
    // 2. Validator Registration & Activation Management
    // ================================================================================================

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
    fn test_activate_and_deactivate_validator() {
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

    // ================================================================================================
    // 3. Stake Bonding, Unbonding & Stake Adjustment Tests
    // ================================================================================================

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
                min_stake + 500
            ));
            assert_ok!(PalletCbcPos::unbond_stake(
                RuntimeOrigin::signed(validator),
                500
            ));
            assert_eq!(PalletCbcPos::stake(validator), min_stake);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeUnbonded {
                validator,
                amount: 500,
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
                PalletCbcPos::unbond_stake(RuntimeOrigin::signed(validator), 100),
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
    fn test_unbond_stake_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::unbond_stake(RuntimeOrigin::signed(1), 500),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_increase_validator_stake_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(validator),
                2000
            ));
            assert_eq!(PalletCbcPos::stake(validator), 2000);
            assert_eq!(Balances::reserved_balance(validator), 2000);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(
                Event::ValidatorStakeIncreased {
                    validator,
                    amount: 2000,
                },
            ));
        });
    }

    #[test]
    fn test_increase_validator_stake_not_registered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::increase_validator_stake(RuntimeOrigin::signed(1), 1000),
                Error::<Test>::ValidatorNotInSet
            );
        });
    }

    #[test]
    fn test_increase_validator_stake_insufficient_balance() {
        new_test_ext().execute_with(|| {
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_noop!(
                PalletCbcPos::increase_validator_stake(RuntimeOrigin::signed(validator), 20000),
                Error::<Test>::InsufficientStake
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
                min_stake + 1000
            ));
            assert_ok!(PalletCbcPos::decrease_validator_stake(
                RuntimeOrigin::signed(validator),
                500
            ));
            assert_eq!(PalletCbcPos::stake(validator), min_stake + 500);
            assert_eq!(Balances::reserved_balance(validator), min_stake + 500);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(
                Event::ValidatorStakeDecreased {
                    validator,
                    amount: 500,
                },
            ));
        });
    }

    #[test]
    fn test_decrease_validator_stake_not_registered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::decrease_validator_stake(RuntimeOrigin::signed(1), 500),
                Error::<Test>::ValidatorNotInSet
            );
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
                PalletCbcPos::decrease_validator_stake(RuntimeOrigin::signed(validator), 100),
                Error::<Test>::InsufficientStake
            );
        });
    }

    // ================================================================================================
    // 4. Score Management Tests
    // ================================================================================================

    #[test]
    fn test_submit_score_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 75));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(75));
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSubmitted {
                validator: 1,
                score: 75,
            }));
        });
    }

    #[test]
    fn test_submit_score_not_registered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 75),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_submit_score_too_low() {
        new_test_ext().execute_with(|| {
            let min_score: u32 = <Test as crate::Config>::MinValidatorScore::get();
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, min_score - 1),
                Error::<Test>::ScoreTooLow
            );
        });
    }

    #[test]
    fn test_boost_score_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 50));
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(2), 1, 20));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(70));
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreBoosted {
                validator: 1,
                old_score: 50,
                new_score: 70,
                boost_amount: 20,
            }));
        });
    }

    #[test]
    fn test_boost_score_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::boost_score(RuntimeOrigin::signed(2), 1, 10),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_boost_score_saturating_overflow() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::submit_score(
                RuntimeOrigin::signed(2),
                1,
                u32::MAX - 10
            ));
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(2), 1, 50));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(u32::MAX));
        });
    }

    #[test]
    fn test_slash_score_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 80));
            assert_ok!(PalletCbcPos::slash_score(RuntimeOrigin::signed(2), 1, 30));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(50));
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSlashed {
                validator: 1,
                old_score: 80,
                new_score: 50,
                slash_amount: 30,
            }));
        });
    }

    #[test]
    fn test_slash_score_underflow() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 50));
            assert_ok!(PalletCbcPos::slash_score(RuntimeOrigin::signed(2), 1, 100));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(0));
        });
    }

    #[test]
    fn test_slash_score_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::slash_score(RuntimeOrigin::signed(2), 1, 10),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    // ================================================================================================
    // 5. Currency & Stake Slashing Tests
    // ================================================================================================

    #[test]
    fn test_slash_validator_call_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::slash_validator_call(RuntimeOrigin::signed(2), 1));
            assert_eq!(PalletCbcPos::slashing_count(1), Some(1));
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorSlashed {
                validator: 1,
                slashing_count: 1,
            }));
        });
    }

    #[test]
    fn test_slash_validator_call_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::slash_validator_call(RuntimeOrigin::signed(2), 1),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_slash_validator_call_max_count_removal() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::slash_validator_call(RuntimeOrigin::signed(2), 1));
            assert_ok!(PalletCbcPos::slash_validator_call(RuntimeOrigin::signed(2), 1));
            assert_ok!(PalletCbcPos::slash_validator_call(RuntimeOrigin::signed(2), 1));

            assert_eq!(PalletCbcPos::validators(1), None);
            assert_eq!(PalletCbcPos::validator_scores(1), None);
            assert_eq!(PalletCbcPos::slashing_count(1), None);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorRemoved {
                validator: 1,
                reason: b"Max slashing count reached".to_vec(),
            }));
        });
    }

    #[test]
    fn test_slash_validator_root_only() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_noop!(
                PalletCbcPos::slash_validator(RuntimeOrigin::signed(1), 1, 100),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_slash_validator_execution() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let initial_balance = Balances::free_balance(1);
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::root(),
                1,
                1000
            ));
            assert_eq!(Balances::free_balance(1), initial_balance - 1000);
            assert_eq!(EpochTotalSlashed::<Test>::get(), 1000);
            assert_eq!(ValidatorEpochSlashed::<Test>::get(1), 1000);
        });
    }

    #[test]
    fn test_slash_validator_default_amount() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let initial_balance = Balances::free_balance(1);
            // Default amount 0 triggers SlashPercent (10%) of free_balance (10000) = 1000
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::root(),
                1,
                0
            ));
            assert_eq!(Balances::free_balance(1), initial_balance - 1000);
            assert_eq!(EpochTotalSlashed::<Test>::get(), 1000);
        });
    }

    #[test]
    fn test_slash_validator_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::slash_validator(RuntimeOrigin::root(), 999, 1000),
                Error::<Test>::ValidatorNotInSet
            );
        });
    }

    #[test]
    fn test_slash_validator_percentage_success() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(1),
                2000
            ));
            assert_ok!(PalletCbcPos::slash_validator_percentage(
                RuntimeOrigin::root(),
                1,
                50
            ));
            // 50% of stake (2000) is 1000
            assert_eq!(EpochTotalSlashed::<Test>::get(), 1000);
        });
    }

    #[test]
    fn test_slash_validator_percentage_invalid() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_noop!(
                PalletCbcPos::slash_validator_percentage(RuntimeOrigin::root(), 1, 101),
                Error::<Test>::InvalidStakeAmount
            );
        });
    }

    #[test]
    fn test_slash_multiple_validators() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
            assert_ok!(PalletCbcPos::slash_multiple_validators(
                RuntimeOrigin::root(),
                vec![1, 2],
                500
            ));
            assert_eq!(ValidatorEpochSlashed::<Test>::get(1), 500);
            assert_eq!(ValidatorEpochSlashed::<Test>::get(2), 500);
            assert_eq!(EpochTotalSlashed::<Test>::get(), 1000);
        });
    }

    #[test]
    fn test_slash_validator_bounds_exceeded_epoch() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let max_slash_epoch: u64 = <Test as crate::Config>::MaxSlashPerEpoch::get();
            assert_noop!(
                PalletCbcPos::slash_validator(
                    RuntimeOrigin::root(),
                    1,
                    max_slash_epoch + 1
                ),
                Error::<Test>::SlashingBoundsExceeded
            );
        });
    }

    #[test]
    fn test_slash_validator_bounds_exceeded_validator() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let max_slash_validator: u64 = <Test as crate::Config>::MaxSlashPerValidator::get();
            assert_noop!(
                PalletCbcPos::slash_validator(
                    RuntimeOrigin::root(),
                    1,
                    max_slash_validator + 1
                ),
                Error::<Test>::SlashingBoundsExceeded
            );
        });
    }

    #[test]
    fn test_slash_validator_stake_internal() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(validator),
                2000
            ));

            assert_ok!(PalletCbcPos::slash_validator_stake(&validator, 500));
            assert_eq!(PalletCbcPos::stake(validator), 1500);
            assert_eq!(PalletCbcPos::slashing_count(validator), Some(1));
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeSlashed {
                validator,
                old_stake: 2000,
                new_stake: 1500,
                slashed_amount: 500,
            }));
        });
    }

    #[test]
    fn test_slash_validator_stake_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::slash_validator_stake(&999, 500),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    // ================================================================================================
    // 6. Currency Rewards & Epoch Distribution Tests
    // ================================================================================================

    #[test]
    fn test_reward_validator_call_root_only() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_noop!(
                PalletCbcPos::reward_validator_call(RuntimeOrigin::signed(1), 1, 500),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_reward_validator_execution() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let initial_balance = Balances::free_balance(1);
            assert_ok!(PalletCbcPos::reward_validator_call(
                RuntimeOrigin::root(),
                1,
                2000
            ));
            assert_eq!(Balances::free_balance(1), initial_balance + 2000);
            assert_eq!(EpochTotalRewarded::<Test>::get(), 2000);
            assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 2000);
        });
    }

    #[test]
    fn test_reward_validator_default_amount() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let initial_balance = Balances::free_balance(1);
            let default_reward: u64 = <Test as crate::Config>::ValidatorReward::get();
            assert_ok!(PalletCbcPos::reward_validator_call(
                RuntimeOrigin::root(),
                1,
                0
            ));
            assert_eq!(Balances::free_balance(1), initial_balance + default_reward);
            assert_eq!(EpochTotalRewarded::<Test>::get(), default_reward);
        });
    }

    #[test]
    fn test_reward_validator_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::reward_validator_call(RuntimeOrigin::root(), 999, 1000),
                Error::<Test>::ValidatorNotInSet
            );
        });
    }

    #[test]
    fn test_reward_validator_bounds_exceeded_epoch() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let max_reward_epoch: u64 = <Test as crate::Config>::MaxRewardPerEpoch::get();
            assert_noop!(
                PalletCbcPos::reward_validator_call(
                    RuntimeOrigin::root(),
                    1,
                    max_reward_epoch + 1
                ),
                Error::<Test>::RewardBoundsExceeded
            );
        });
    }

    #[test]
    fn test_reward_validator_bounds_exceeded_validator() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            let max_reward_validator: u64 = <Test as crate::Config>::MaxRewardPerValidator::get();
            assert_noop!(
                PalletCbcPos::reward_validator_call(
                    RuntimeOrigin::root(),
                    1,
                    max_reward_validator + 1
                ),
                Error::<Test>::RewardBoundsExceeded
            );
        });
    }

    #[test]
    fn test_reward_multiple_validators() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
            assert_ok!(PalletCbcPos::reward_multiple_validators(
                RuntimeOrigin::root(),
                vec![1, 2],
                1000
            ));
            assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 1000);
            assert_eq!(ValidatorEpochRewarded::<Test>::get(2), 1000);
            assert_eq!(EpochTotalRewarded::<Test>::get(), 2000);
        });
    }

    #[test]
    fn test_reward_all_active_validators() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));
            assert_ok!(PalletCbcPos::reward_all_active_validators(
                RuntimeOrigin::root(),
                1000
            ));
            // TestValidatorHandler returns active validators vec![1, 2]
            assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 1000);
            assert_eq!(ValidatorEpochRewarded::<Test>::get(2), 1000);
        });
    }

    #[test]
    fn test_distribute_epoch_rewards_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(2)));

            // Distribute 10,000 pool
            // Active validators = [1, 2].
            // Scores: 1 -> 90 (>=80 high performer, top 20% performer), 2 -> 60.
            // Base pool 60% = 6000 (3000 each)
            // Performance pool 25% = 2500 (2500 for validator 1)
            // Top performer pool 15% = 1500 (1500 for validator 1)
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
    fn test_reward_validator_internal() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::reward_validator(&validator, 1000));
            assert_eq!(PalletCbcPos::stake(validator), 1000);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeUpdated {
                validator,
                old_stake: 0,
                new_stake: 1000,
            }));
        });
    }

    #[test]
    fn test_reward_validator_internal_unregistered() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPos::reward_validator(&999, 1000),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    // ================================================================================================
    // 7. Helper & Administration Functions Tests
    // ================================================================================================

    #[test]
    fn test_force_eject_validator() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(validator),
                2000
            ));
            assert_eq!(Balances::reserved_balance(validator), 2000);

            assert_ok!(PalletCbcPos::force_eject_validator(&validator));
            assert_eq!(PalletCbcPos::validators(validator), None);
            assert_eq!(PalletCbcPos::stake(validator), 0);
            assert_eq!(Balances::reserved_balance(validator), 0);
            System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorLeft {
                validator,
            }));
        });
    }

    #[test]
    fn test_reset_epoch_totals() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::reward_validator_call(
                RuntimeOrigin::root(),
                1,
                1000
            ));
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::root(),
                1,
                500
            ));

            assert_eq!(EpochTotalRewarded::<Test>::get(), 1000);
            assert_eq!(EpochTotalSlashed::<Test>::get(), 500);

            PalletCbcPos::reset_epoch_totals();

            assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
            assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
            assert_eq!(ValidatorEpochRewarded::<Test>::get(1), 0);
            assert_eq!(ValidatorEpochSlashed::<Test>::get(1), 0);
        });
    }

    #[test]
    fn test_calculate_epoch_economic_impact() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            assert_ok!(PalletCbcPos::reward_validator_call(
                RuntimeOrigin::root(),
                1,
                2000
            ));
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::root(),
                1,
                1000
            ));

            let (rewarded, slashed) = PalletCbcPos::calculate_epoch_economic_impact(1);
            assert_eq!(rewarded, 2000);
            assert_eq!(slashed, 1000);
        });
    }

    #[test]
    fn test_calculate_slash_amount() {
        new_test_ext().execute_with(|| {
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(validator),
                5000
            ));

            // SlashPercent is 10%, 10% of 5000 is 500
            let slash_amount = PalletCbcPos::calculate_slash_amount(&validator).unwrap();
            assert_eq!(slash_amount, 500);
        });
    }

    #[test]
    fn test_validator_stake_score() {
        new_test_ext().execute_with(|| {
            let validator = 1u64;
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(
                validator
            )));
            assert_ok!(PalletCbcPos::increase_validator_stake(
                RuntimeOrigin::signed(validator),
                3500
            ));

            let stake_score = PalletCbcPos::validator_stake_score(&validator);
            assert_eq!(stake_score, 3500u128);
        });
    }

    #[test]
    fn test_get_slashing_history() {
        new_test_ext().execute_with(|| {
            let history = PalletCbcPos::get_slashing_history(1);
            assert!(history.is_empty());
        });
    }

    #[test]
    fn test_get_active_validators() {
        new_test_ext().execute_with(|| {
            let active = PalletCbcPos::get_active_validators();
            // TestValidatorHandler returns vec![1, 2]
            assert_eq!(active, vec![1, 2]);
        });
    }
}