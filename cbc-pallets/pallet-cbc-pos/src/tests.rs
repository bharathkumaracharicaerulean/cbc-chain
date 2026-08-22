//! Industrial, production-grade unit test suite for `pallet-cbc-pos`
//!
//! Using actual pallet trait implementations and storage without artificial mock wrappers.

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        EpochTotalRewarded, EpochTotalSlashed, Error, Event, GenesisConfig,
        ValidatorEpochRewarded, ValidatorEpochSlashed, Stake, Validators, ValidatorScores, SlashingCount,
    };
    use frame_support::{assert_noop, assert_ok, traits::Get};
    use sp_runtime::{BuildStorage, DispatchError};

    // Named account constants to avoid magic numbers across tests
    const VALIDATOR_A: u64 = 1;
    const VALIDATOR_B: u64 = 2;
    const UNREGISTERED_VAL: u64 = 999;

    /// Helper function to advance to a specific block number for system event triggers
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    /// Helper to fund an account with free balance via pallet_balances force_set_balance extrinsic
    fn set_balance(account: u64, amount: u64) {
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), account, amount));
    }

    /// Helper to assert reserved balance for an account
    fn assert_reserved_balance(account: u64, expected: u64) {
        assert_eq!(Balances::reserved_balance(&account), expected);
    }

    /// Helper to assert free balance for an account
    fn assert_free_balance(account: u64, expected: u64) {
        assert_eq!(Balances::free_balance(&account), expected);
    }

    /// Setup helper to register a validator via real signed extrinsic
    fn register_validator(validator: u64) {
        assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(validator)));
    }

    /// Setup helper to register and increase stake for a validator via real extrinsics
    fn register_and_stake(validator: u64, amount: u64) {
        set_balance(validator, amount * 10);
        register_validator(validator);
        assert_ok!(PalletCbcPos::increase_validator_stake(
            RuntimeOrigin::signed(validator),
            amount
        ));
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
                validators: vec![VALIDATOR_A, VALIDATOR_B],
                validator_scores: vec![100, 80],
                current_epoch: 5,
                slashing_count: vec![(VALIDATOR_A, 1)],
            }
            .assimilate_storage(&mut storage)
            .unwrap();

            let mut ext = sp_io::TestExternalities::from(storage);
            ext.execute_with(|| {
                assert_eq!(Validators::<Test>::get(VALIDATOR_A), Some(true));
                assert_eq!(Validators::<Test>::get(VALIDATOR_B), Some(true));
                assert_eq!(ValidatorScores::<Test>::get(VALIDATOR_A), Some(100));
                assert_eq!(ValidatorScores::<Test>::get(VALIDATOR_B), Some(80));
                assert_eq!(PalletCbcPos::current_epoch(), 5);
                assert_eq!(SlashingCount::<Test>::get(VALIDATOR_A), Some(1));
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
                register_validator(VALIDATOR_A);
                assert_eq!(Validators::<Test>::get(VALIDATOR_A), Some(true));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorRegistered {
                    validator: VALIDATOR_A,
                }));
            });
        }

        #[test]
        fn test_register_validator_already_registered() {
            new_test_ext().execute_with(|| {
                register_validator(VALIDATOR_A);
                assert_noop!(
                    PalletCbcPos::register_validator(RuntimeOrigin::signed(VALIDATOR_A)),
                    Error::<Test>::ValidatorAlreadyRegistered
                );
            });
        }

        #[test]
        fn test_register_validator_max_limit() {
            new_test_ext().execute_with(|| {
                let max_validators: u32 = <Test as crate::Config>::MaxValidators::get();
                for i in 1..=max_validators {
                    register_validator(i as u64);
                }
                let overflow_account = (max_validators + 1) as u64;
                assert_noop!(
                    PalletCbcPos::register_validator(RuntimeOrigin::signed(overflow_account)),
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
                register_validator(VALIDATOR_A);

                assert_ok!(PalletCbcPos::deactivate_validator(&VALIDATOR_A));
                assert_eq!(Validators::<Test>::get(VALIDATOR_A), Some(false));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorDeactivated {
                    validator: VALIDATOR_A,
                }));

                assert_ok!(PalletCbcPos::activate_validator(&VALIDATOR_A));
                assert_eq!(Validators::<Test>::get(VALIDATOR_A), Some(true));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorActivated {
                    validator: VALIDATOR_A,
                }));
            });
        }

        #[test]
        fn test_activate_validator_unregistered() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::activate_validator(&UNREGISTERED_VAL),
                    Error::<Test>::ValidatorNotRegistered
                );
            });
        }

        #[test]
        fn test_deactivate_validator_unregistered() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::deactivate_validator(&UNREGISTERED_VAL),
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
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                set_balance(VALIDATOR_A, 50000);
                register_validator(VALIDATOR_A);
                let initial_reserved = Balances::reserved_balance(&VALIDATOR_A);

                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));

                assert_eq!(Stake::<Test>::get(VALIDATOR_A), min_stake);
                assert_reserved_balance(VALIDATOR_A, initial_reserved + min_stake);
                assert!(get_handler_events().contains(&HandlerEvent::Joined {
                    validator: VALIDATOR_A,
                    stake: min_stake,
                }));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeBonded {
                    validator: VALIDATOR_A,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_bond_stake_insufficient_stake() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_validator(VALIDATOR_A);
                assert_noop!(
                    PalletCbcPos::bond_stake(RuntimeOrigin::signed(VALIDATOR_A), min_stake - 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_bond_stake_not_registered() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                assert_noop!(
                    PalletCbcPos::bond_stake(RuntimeOrigin::signed(VALIDATOR_A), min_stake),
                    Error::<Test>::ValidatorNotRegistered
                );
            });
        }

        #[test]
        fn test_unbond_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake * 2
                ));
                assert_reserved_balance(VALIDATOR_A, min_stake * 2);

                assert_ok!(PalletCbcPos::unbond_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));

                assert_eq!(Stake::<Test>::get(VALIDATOR_A), min_stake);
                assert_reserved_balance(VALIDATOR_A, min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeUnbonded {
                    validator: VALIDATOR_A,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_unbond_stake_below_minimum() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));
                assert_noop!(
                    PalletCbcPos::unbond_stake(RuntimeOrigin::signed(VALIDATOR_A), 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_unbond_stake_excessive() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::bond_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));
                assert_noop!(
                    PalletCbcPos::unbond_stake(RuntimeOrigin::signed(VALIDATOR_A), min_stake + 100),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_increase_validator_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                set_balance(VALIDATOR_A, 50000);
                register_validator(VALIDATOR_A);
                let initial_reserved = Balances::reserved_balance(&VALIDATOR_A);

                assert_ok!(PalletCbcPos::increase_validator_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));

                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), min_stake);
                assert_reserved_balance(VALIDATOR_A, initial_reserved + min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeIncreased {
                    validator: VALIDATOR_A,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_increase_validator_stake_not_registered() {
            new_test_ext().execute_with(|| {
                set_balance(UNREGISTERED_VAL, 50000);
                assert_noop!(
                    PalletCbcPos::increase_validator_stake(RuntimeOrigin::signed(UNREGISTERED_VAL), 1000),
                    Error::<Test>::ValidatorNotInSet
                );
            });
        }

        #[test]
        fn test_decrease_validator_stake_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_and_stake(VALIDATOR_A, min_stake * 3);
                let reserved_before = Balances::reserved_balance(&VALIDATOR_A);

                assert_ok!(PalletCbcPos::decrease_validator_stake(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    min_stake
                ));

                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), min_stake * 2);
                assert_reserved_balance(VALIDATOR_A, reserved_before - min_stake);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeDecreased {
                    validator: VALIDATOR_A,
                    amount: min_stake,
                }));
            });
        }

        #[test]
        fn test_decrease_validator_stake_below_min() {
            new_test_ext().execute_with(|| {
                let min_stake: u64 = <Test as crate::Config>::MinStake::get();
                register_and_stake(VALIDATOR_A, min_stake);
                assert_noop!(
                    PalletCbcPos::decrease_validator_stake(RuntimeOrigin::signed(VALIDATOR_A), 1),
                    Error::<Test>::InsufficientStake
                );
            });
        }

        #[test]
        fn test_slash_validator_stake_direct() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_and_stake(VALIDATOR_A, 5000);

                assert_ok!(PalletCbcPos::slash_validator_stake(&VALIDATOR_A, 1000));
                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), 4000);
                assert_eq!(SlashingCount::<Test>::get(&VALIDATOR_A), Some(1));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorStakeSlashed {
                    validator: VALIDATOR_A,
                    old_stake: 5000,
                    new_stake: 4000,
                    slashed_amount: 1000,
                }));
            });
        }

        #[test]
        fn test_reward_validator_stake_direct() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_and_stake(VALIDATOR_A, 5000);

                assert_ok!(PalletCbcPos::reward_validator(&VALIDATOR_A, 1000));
                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), 6000);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::StakeBonded {
                    validator: VALIDATOR_A,
                    amount: 1000,
                }));
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
                register_validator(VALIDATOR_A);
                let min_score: u32 = <Test as crate::Config>::MinValidatorScore::get();
                assert_ok!(PalletCbcPos::submit_score(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A,
                    min_score + 10
                ));
                assert_eq!(
                    ValidatorScores::<Test>::get(&VALIDATOR_A),
                    Some(min_score + 10)
                );
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSubmitted {
                    validator: VALIDATOR_A,
                    score: min_score + 10,
                }));
            });
        }

        #[test]
        fn test_submit_score_too_low() {
            new_test_ext().execute_with(|| {
                register_validator(VALIDATOR_A);
                let min_score: u32 = <Test as crate::Config>::MinValidatorScore::get();
                assert_noop!(
                    PalletCbcPos::submit_score(
                        RuntimeOrigin::signed(VALIDATOR_A),
                        VALIDATOR_A,
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
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::boost_score(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A,
                    20
                ));
                assert_eq!(ValidatorScores::<Test>::get(&VALIDATOR_A), Some(20));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreBoosted {
                    validator: VALIDATOR_A,
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
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::boost_score(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A,
                    50
                ));
                assert_ok!(PalletCbcPos::slash_score(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A,
                    20
                ));
                assert_eq!(ValidatorScores::<Test>::get(&VALIDATOR_A), Some(30));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ScoreSlashed {
                    validator: VALIDATOR_A,
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
                register_validator(VALIDATOR_A);
                assert_ok!(PalletCbcPos::slash_validator_call(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A
                ));
                assert_eq!(SlashingCount::<Test>::get(&VALIDATOR_A), Some(1));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorSlashed {
                    validator: VALIDATOR_A,
                    slashing_count: 1,
                }));
            });
        }

        #[test]
        fn test_slash_validator_call_max_count_removal() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                let max_slash: u32 = <Test as crate::Config>::MaxSlashingCount::get();
                for _ in 0..max_slash {
                    assert_ok!(PalletCbcPos::slash_validator_call(
                        RuntimeOrigin::signed(VALIDATOR_A),
                        VALIDATOR_A
                    ));
                }
                assert!(!Validators::<Test>::contains_key(&VALIDATOR_A));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorRemoved {
                    validator: VALIDATOR_A,
                    reason: b"Max slashing count reached".to_vec(),
                }));
            });
        }

        #[test]
        fn test_slash_validator_root_only() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_and_stake(VALIDATOR_A, 5000);
                let reserved_before = Balances::reserved_balance(&VALIDATOR_A);

                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::root(),
                    VALIDATOR_A,
                    500
                ));

                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), 4500);
                assert_reserved_balance(VALIDATOR_A, reserved_before - 500);
                assert_eq!(EpochTotalSlashed::<Test>::get(), 500);
                assert_eq!(ValidatorEpochSlashed::<Test>::get(VALIDATOR_A), 500);
                assert_eq!(PalletCbcPos::get_slashing_history(VALIDATOR_A).len(), 1);

                assert_noop!(
                    PalletCbcPos::slash_validator(RuntimeOrigin::signed(VALIDATOR_A), VALIDATOR_A, 500),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_slash_validator_percentage_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                set_balance(VALIDATOR_A, 50000);
                register_and_stake(VALIDATOR_A, 5000);

                let pre_free = Balances::free_balance(&VALIDATOR_A);
                let pre_reserved = Balances::reserved_balance(&VALIDATOR_A);
                let pre_stake = Stake::<Test>::get(&VALIDATOR_A);

                // 50% of 5000 = 2500 slashed from reserved stake
                assert_ok!(PalletCbcPos::slash_validator_percentage(
                    RuntimeOrigin::root(),
                    VALIDATOR_A,
                    50
                ));

                let post_free = Balances::free_balance(&VALIDATOR_A);
                let post_reserved = Balances::reserved_balance(&VALIDATOR_A);
                let post_stake = Stake::<Test>::get(&VALIDATOR_A);

                // Liquid free balance is UNTOUCHED
                assert_eq!(post_free, pre_free);
                // Reserved stake balance is reduced by 2500
                assert_eq!(pre_reserved - post_reserved, 2500);
                // On-chain Stake storage is reduced by 2500
                assert_eq!(pre_stake - post_stake, 2500);
                // ValidatorHandler callback was invoked with 2500 slashed amount
                assert!(get_handler_events().contains(&HandlerEvent::Slashed {
                    validator: VALIDATOR_A,
                    amount: 2500,
                    penalty: 2,
                }));

                assert_noop!(
                    PalletCbcPos::slash_validator_percentage(RuntimeOrigin::root(), VALIDATOR_A, 150),
                    Error::<Test>::InvalidStakeAmount
                );
            });
        }

        #[test]
        fn test_slash_multiple_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_and_stake(VALIDATOR_A, 5000);
                register_and_stake(VALIDATOR_B, 5000);

                assert_ok!(PalletCbcPos::slash_multiple_validators(
                    RuntimeOrigin::root(),
                    vec![VALIDATOR_A, VALIDATOR_B],
                    1000
                ));

                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), 4000);
                assert_eq!(Stake::<Test>::get(&VALIDATOR_B), 4000);
                assert_eq!(EpochTotalSlashed::<Test>::get(), 2000);
                assert_eq!(ValidatorEpochSlashed::<Test>::get(VALIDATOR_A), 1000);
                assert_eq!(ValidatorEpochSlashed::<Test>::get(VALIDATOR_B), 1000);
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
                register_validator(VALIDATOR_A);
                let bal_before = Balances::free_balance(&VALIDATOR_A);

                assert_ok!(PalletCbcPos::reward_validator_call(
                    RuntimeOrigin::root(),
                    VALIDATOR_A,
                    1000
                ));

                assert_free_balance(VALIDATOR_A, bal_before + 1000);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_A), 1000);
                assert_eq!(EpochTotalRewarded::<Test>::get(), 1000);

                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::signed(VALIDATOR_A), VALIDATOR_A, 1000),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_reward_multiple_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                register_validator(VALIDATOR_B);
                let bal_a_before = Balances::free_balance(&VALIDATOR_A);
                let bal_b_before = Balances::free_balance(&VALIDATOR_B);

                assert_ok!(PalletCbcPos::reward_multiple_validators(
                    RuntimeOrigin::root(),
                    vec![VALIDATOR_A, VALIDATOR_B],
                    500
                ));

                assert_free_balance(VALIDATOR_A, bal_a_before + 500);
                assert_free_balance(VALIDATOR_B, bal_b_before + 500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_A), 500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_B), 500);
                assert_eq!(EpochTotalRewarded::<Test>::get(), 1000);
            });
        }

        #[test]
        fn test_reward_all_active_validators() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                register_validator(VALIDATOR_B);
                let bal_a_before = Balances::free_balance(&VALIDATOR_A);
                let bal_b_before = Balances::free_balance(&VALIDATOR_B);

                assert_ok!(PalletCbcPos::reward_all_active_validators(
                    RuntimeOrigin::root(),
                    500
                ));

                assert_free_balance(VALIDATOR_A, bal_a_before + 500);
                assert_free_balance(VALIDATOR_B, bal_b_before + 500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_A), 500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_B), 500);
                assert_eq!(EpochTotalRewarded::<Test>::get(), 1000);
            });
        }

        #[test]
        fn test_distribute_epoch_rewards_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                register_validator(VALIDATOR_B);

                // Submit scores via extrinsic: val A -> 90 (high performer), val B -> 70
                assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(VALIDATOR_A), VALIDATOR_A, 90));
                assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(VALIDATOR_B), VALIDATOR_B, 70));

                // Distribute 10,000 pool:
                // Active validators = [1, 2] (total 2 validators).
                // Base pool (60%) = 6,000 -> split equally: 3,000 to val A, 3,000 to val B.
                // Performance pool (25%) = 2,500 -> val A score 90 >= threshold 80 -> 2,500 to val A.
                // Top performer pool (15%) = 1,500 -> top_performer_count = (2 * 20%) / 100 = 0.
                // Total credited rewards: Val A = 3000 + 2500 = 5500; Val B = 3000; Total EpochTotalRewarded = 8500.
                assert_ok!(PalletCbcPos::distribute_epoch_rewards(
                    RuntimeOrigin::root(),
                    10000
                ));

                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_A), 5500);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_B), 3000);
                assert_eq!(EpochTotalRewarded::<Test>::get(), 8500);
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::EpochRewardsDistributed {
                    epoch: 0,
                    total_distributed: 8500,
                }));
            });
        }

        #[test]
        fn test_distribute_epoch_rewards_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPos::distribute_epoch_rewards(RuntimeOrigin::signed(VALIDATOR_A), 10000),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn test_reward_validator_bounds_exceeded_validator() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                let max_reward: u64 = <Test as crate::Config>::MaxRewardPerValidator::get();
                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::root(), VALIDATOR_A, max_reward + 1),
                    Error::<Test>::RewardBoundsExceeded
                );
            });
        }

        #[test]
        fn test_reward_validator_bounds_exceeded_epoch() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_validator(VALIDATOR_A);
                let max_epoch_reward: u64 = <Test as crate::Config>::MaxRewardPerEpoch::get();
                EpochTotalRewarded::<Test>::put(max_epoch_reward);
                assert_noop!(
                    PalletCbcPos::reward_validator_call(RuntimeOrigin::root(), VALIDATOR_A, 100),
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
                register_and_stake(VALIDATOR_A, 1000);
                let slash_amount = PalletCbcPos::calculate_slash_amount(&VALIDATOR_A).unwrap();
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
                ValidatorEpochRewarded::<Test>::insert(VALIDATOR_A, 1000);
                ValidatorEpochSlashed::<Test>::insert(VALIDATOR_A, 500);

                PalletCbcPos::reset_epoch_totals();

                assert_eq!(EpochTotalRewarded::<Test>::get(), 0);
                assert_eq!(EpochTotalSlashed::<Test>::get(), 0);
                assert_eq!(ValidatorEpochRewarded::<Test>::get(VALIDATOR_A), 0);
                assert_eq!(ValidatorEpochSlashed::<Test>::get(VALIDATOR_A), 0);
            });
        }

        #[test]
        fn test_get_active_validators() {
            new_test_ext().execute_with(|| {
                register_validator(VALIDATOR_A);
                register_validator(VALIDATOR_B);
                let active = PalletCbcPos::get_active_validators();
                assert!(active.contains(&VALIDATOR_A));
                assert!(active.contains(&VALIDATOR_B));
            });
        }

        #[test]
        fn test_get_slashing_history() {
            new_test_ext().execute_with(|| {
                // Initial empty state
                let history = PalletCbcPos::get_slashing_history(VALIDATOR_A);
                assert!(history.is_empty());

                // After slash
                register_and_stake(VALIDATOR_A, 5000);
                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::root(),
                    VALIDATOR_A,
                    500
                ));

                let history_after = PalletCbcPos::get_slashing_history(VALIDATOR_A);
                assert_eq!(history_after.len(), 1);
                assert_eq!(history_after[0].amount, 500);
                assert_eq!(history_after[0].reason, crate::SlashReason::ManualSlash);
            });
        }

        #[test]
        fn test_force_eject_validator() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                register_and_stake(VALIDATOR_A, 5000);

                assert_ok!(PalletCbcPos::force_eject_validator(&VALIDATOR_A));

                assert!(!Validators::<Test>::contains_key(&VALIDATOR_A));
                assert_eq!(Stake::<Test>::get(&VALIDATOR_A), 0);
                assert_reserved_balance(VALIDATOR_A, 0);
                assert!(get_handler_events().contains(&HandlerEvent::Left {
                    validator: VALIDATOR_A
                }));
                System::assert_has_event(RuntimeEvent::PalletCbcPos(Event::ValidatorLeft {
                    validator: VALIDATOR_A,
                }));
            });
        }

        #[test]
        fn test_validator_stake_score() {
            new_test_ext().execute_with(|| {
                register_and_stake(VALIDATOR_A, 5000);
                assert_eq!(PalletCbcPos::validator_stake_score(&VALIDATOR_A), 5000);
            });
        }
    }
}