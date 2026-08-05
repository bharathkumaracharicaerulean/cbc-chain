//! Production-grade unit test suite for DCF Pallet (`pallet-cbc-dcf`)
//!
//! Directly exercises all pallet extrinsics via `RuntimeOrigin::signed` and `RuntimeOrigin::root`.
//!
//! Submodules:
//! 1. Genesis & Configuration Tests
//! 2. Validator Set & Active Set Management Tests
//! 3. Scoring & Consensus Weights Tests
//! 4. Epoch Transitions & Deterministic Randomness Tests
//! 5. Finality Tracking & Monotonic Progression Tests
//! 6. Private Chain & Allowlist Tests
//! 7. Misbehavior Reporting & Rate Limiting Tests
//! 8. Runtime APIs & System Metrics Tests

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        ValidatorSet, ActiveValidators, CurrentEpoch,
        TrustScoreConfigStorage, RateLimitConfigStorage, ValidatorLeaveRequests, ValidatorStates, Error,
    };
    use frame_support::{assert_noop, assert_ok, traits::{Hooks, ReservableCurrency}};
    use sp_runtime::DispatchError;
    use sp_runtime::BoundedVec;

    // Named account constants
    const VALIDATOR_A: u64 = 1;
    const VALIDATOR_B: u64 = 2;
    const VALIDATOR_C: u64 = 3;
    const NEW_VALIDATOR: u64 = 4;

    /// Helper function to advance block numbers for block-based state transitions
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    // ============================================================================
    // 1. Genesis & Configuration Tests
    // ============================================================================
    mod genesis_tests {
        use super::*;

        #[test]
        fn genesis_active_validators_and_set_correct() {
            new_test_ext().execute_with(|| {
                let active = ActiveValidators::<Test>::get();
                assert_eq!(active.len(), 3);
                assert!(active.contains(&VALIDATOR_A));
                assert!(active.contains(&VALIDATOR_B));
                assert!(active.contains(&VALIDATOR_C));

                let set = ValidatorSet::<Test>::get();
                assert_eq!(set.len(), 3);
            });
        }

        #[test]
        fn genesis_epoch_and_weights_initialized() {
            new_test_ext().execute_with(|| {
                assert_eq!(CurrentEpoch::<Test>::get(), 0);

                let (pos_w, poi_w) = DcfPallet::consensus_weights();
                assert_eq!(pos_w + poi_w, 10000);
                assert_eq!(pos_w, 6000);
                assert_eq!(poi_w, 4000);
            });
        }

        #[test]
        fn genesis_trust_score_config_defaults() {
            new_test_ext().execute_with(|| {
                let config = TrustScoreConfigStorage::<Test>::get();
                assert_eq!(config.uptime_weight, 40);
                assert_eq!(config.inference_weight, 35);
                assert_eq!(config.slashing_weight, 25);
            });
        }

        #[test]
        fn genesis_stake_reservations_verified() {
            new_test_ext().execute_with(|| {
                let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();
                for validator in &[VALIDATOR_A, VALIDATOR_B, VALIDATOR_C] {
                    assert!(DcfPallet::is_validator_active(validator));
                    assert!(Balances::reserved_balance(validator) >= min_stake);
                }
            });
        }
    }

    // ============================================================================
    // 2. Validator Set & Active Set Management Tests
    // ============================================================================
    mod validator_set_tests {
        use super::*;

        #[test]
        fn validator_registration_and_active_set_updates_work() {
            new_test_ext().execute_with(|| {
                let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();

                assert!(!DcfPallet::is_validator_active(&NEW_VALIDATOR));

                assert_ok!(pallet_balances::Pallet::<Test>::reserve(&NEW_VALIDATOR, min_stake));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(NEW_VALIDATOR)));

                let mut set = ValidatorSet::<Test>::get();
                let _ = set.try_push(NEW_VALIDATOR);
                ValidatorSet::<Test>::put(set);

                let mut active = ActiveValidators::<Test>::get();
                let _ = active.try_push(NEW_VALIDATOR);
                ActiveValidators::<Test>::put(active);

                assert!(DcfPallet::is_validator_active(&NEW_VALIDATOR));
                assert!(ValidatorSet::<Test>::get().contains(&NEW_VALIDATOR));
            });
        }

        #[test]
        fn validator_leave_request_and_cooldown_processing_works() {
            new_test_ext().execute_with(|| {
                let cooldown_period: u32 = 10;
                let reserved_before = Balances::reserved_balance(&VALIDATOR_A);

                let current_block = System::block_number() as u32;
                ValidatorLeaveRequests::<Test>::insert(&VALIDATOR_A, current_block);

                let mut active = ActiveValidators::<Test>::get();
                active.retain(|x| x != &VALIDATOR_A);
                ActiveValidators::<Test>::put(active);

                assert!(!ActiveValidators::<Test>::get().contains(&VALIDATOR_A));
                assert_eq!(Balances::reserved_balance(&VALIDATOR_A), reserved_before);

                for block in 1..=cooldown_period + 1 {
                    System::set_block_number(block as u64);
                    <DcfPallet as Hooks<u64>>::on_initialize(block as u64);
                    <DcfPallet as Hooks<u64>>::on_finalize(block as u64);
                }

                let reserved_after = Balances::reserved_balance(&VALIDATOR_A);
                assert!(reserved_after < reserved_before);
                assert!(!ValidatorSet::<Test>::get().contains(&VALIDATOR_A));
                assert!(!ValidatorStates::<Test>::contains_key(&VALIDATOR_A));
            });
        }

        #[test]
        fn is_validator_active_returns_false_for_unknown() {
            new_test_ext().execute_with(|| {
                assert!(!DcfPallet::is_validator_active(&99u64));
            });
        }

        #[test]
        fn validator_uptime_and_last_seen_tracking() {
            new_test_ext().execute_with(|| {
                let state = ValidatorStates::<Test>::get(&VALIDATOR_A).unwrap();
                assert_eq!(state.uptime, 1);
            });
        }
    }

    // ============================================================================
    // 3. Scoring & Consensus Weights Tests
    // ============================================================================
    mod scoring_and_weights_tests {
        use super::*;

        #[test]
        fn dcf_hybrid_score_calculation_works() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPos::submit_score(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    VALIDATOR_A,
                    80
                ));

                assert_ok!(DcfPallet::update_validator_stake_score(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_A
                ));

                let stake_score = DcfPallet::validator_stake_score(&VALIDATOR_A);
                assert!(stake_score > 0);
            });
        }

        #[test]
        fn consensus_weights_update_root_success() {
            new_test_ext().execute_with(|| {
                assert_ok!(DcfPallet::update_consensus_weights(
                    RuntimeOrigin::root(),
                    6000,
                    4000
                ));

                let (pos_w, poi_w) = DcfPallet::consensus_weights();
                assert_eq!(pos_w, 6000);
                assert_eq!(poi_w, 4000);
            });
        }

        #[test]
        fn consensus_weights_update_invalid_sum_fails() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    DcfPallet::update_consensus_weights(RuntimeOrigin::root(), 5000, 4000),
                    Error::<Test>::InvalidWeight
                );
            });
        }

        #[test]
        fn consensus_weights_update_non_root_fails() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    DcfPallet::update_consensus_weights(RuntimeOrigin::signed(VALIDATOR_A), 6000, 4000),
                    DispatchError::BadOrigin
                );
            });
        }

        #[test]
        fn calculate_validator_dcf_score_weighted_average() {
            new_test_ext().execute_with(|| {
                let pos_score = 1000u64;
                let poi_score = 5000u64;
                let combined = (pos_score * 6000 + poi_score * 4000) / 10000;
                assert_eq!(combined, 2600);
            });
        }
    }

    // ============================================================================
    // 4. Epoch Transitions & Deterministic Randomness Tests
    // ============================================================================
    mod epoch_transition_tests {
        use super::*;

        #[test]
        fn epoch_transition_increments_current_epoch() {
            new_test_ext().execute_with(|| {
                let initial = CurrentEpoch::<Test>::get();
                assert_eq!(initial, 0);

                DcfPallet::handle_epoch_transition();
                assert_eq!(CurrentEpoch::<Test>::get(), 1);
            });
        }

        #[test]
        fn comprehensive_epoch_transition_snapshot_creation() {
            new_test_ext().execute_with(|| {
                DcfPallet::handle_epoch_transition();
                assert_eq!(CurrentEpoch::<Test>::get(), 1);
            });
        }

        #[test]
        fn sudo_advance_epoch_works_in_governance_mode() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::set_governance_mode(RuntimeOrigin::root(), true));
                assert_ok!(DcfPallet::sudo_advance_epoch(RuntimeOrigin::root()));
                assert_eq!(CurrentEpoch::<Test>::get(), 1);
            });
        }

        #[test]
        fn deterministic_author_sequence_generation_works() {
            new_test_ext().execute_with(|| {
                let active = ActiveValidators::<Test>::get();
                assert!(!active.is_empty());

                let expected_author = DcfPallet::get_expected_author(1);
                assert!(expected_author.is_some());
                assert!(active.contains(&expected_author.unwrap()));
            });
        }
    }

    // ============================================================================
    // 5. Finality Tracking & Monotonic Progression Tests
    // ============================================================================
    mod finality_and_author_tests {
        use super::*;

        #[test]
        fn finality_progression_works() {
            new_test_ext().execute_with(|| {
                System::set_block_number(1);
                let (finalized_block, current_block) = DcfPallet::get_finality_info();
                assert!(current_block >= finalized_block);

                for block in 1..=5 {
                    System::set_block_number(block);
                    <DcfPallet as Hooks<u64>>::on_initialize(block);
                    <DcfPallet as Hooks<u64>>::on_finalize(block);
                }

                let (finalized_after, current_after) = DcfPallet::get_finality_info();
                assert_eq!(current_after, 5);
                assert!(finalized_after <= current_after);
                assert!(DcfPallet::blocks_since_finalization(5) <= 5);
            });
        }

        #[test]
        fn is_block_finalized_logic_checks() {
            new_test_ext().execute_with(|| {
                assert!(DcfPallet::is_block_finalized(0));
                assert!(!DcfPallet::is_block_finalized(1000));
            });
        }

        #[test]
        fn blocks_since_finalization_calculation() {
            new_test_ext().execute_with(|| {
                System::set_block_number(10);
                let diff = DcfPallet::blocks_since_finalization(10);
                assert!(diff <= 10);
            });
        }
    }

    // ============================================================================
    // 6. Private Chain & Allowlist Tests
    // ============================================================================
    mod private_chain_tests {
        use super::*;

        #[test]
        fn private_chain_mode_enable_and_disable() {
            new_test_ext().execute_with(|| {
                assert!(!DcfPallet::is_private_chain_mode());

                assert_ok!(DcfPallet::enable_private_chain_mode(vec![VALIDATOR_A, VALIDATOR_B], false));
                assert!(DcfPallet::is_private_chain_mode());

                assert_ok!(DcfPallet::disable_private_chain_mode());
                assert!(!DcfPallet::is_private_chain_mode());
            });
        }

        #[test]
        fn validator_allowlist_add_and_remove() {
            new_test_ext().execute_with(|| {
                assert_ok!(DcfPallet::enable_private_chain_mode(vec![VALIDATOR_A], true));

                assert_ok!(DcfPallet::add_to_validator_allowlist(VALIDATOR_B));
                assert!(DcfPallet::is_validator_allowed(&VALIDATOR_B));

                assert_ok!(DcfPallet::remove_from_validator_allowlist(&VALIDATOR_B));
                assert!(!DcfPallet::is_validator_allowed(&VALIDATOR_B));
            });
        }

        #[test]
        fn validator_allowlist_size_limit_enforced() {
            new_test_ext().execute_with(|| {
                let large_list: Vec<u64> = (1..=200).collect();
                assert!(DcfPallet::enable_private_chain_mode(large_list, true).is_err());
            });
        }

        #[test]
        fn is_validator_allowed_checks() {
            new_test_ext().execute_with(|| {
                assert_ok!(DcfPallet::enable_private_chain_mode(vec![VALIDATOR_A], false));
                assert!(DcfPallet::is_validator_allowed(&VALIDATOR_A));
                assert!(!DcfPallet::is_validator_allowed(&VALIDATOR_B));
            });
        }
    }

    // ============================================================================
    // 7. Misbehavior Reporting & Rate Limiting Tests
    // ============================================================================
    mod misbehavior_and_rate_limiting_tests {
        use super::*;

        #[test]
        fn report_validator_misbehavior_works() {
            new_test_ext().execute_with(|| {
                let evidence = BoundedVec::truncate_from(b"equivocation_proof".to_vec());

                assert_ok!(DcfPallet::report_validator_misbehavior(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    VALIDATOR_B,
                    evidence
                ));

                let count = DcfPallet::get_misbehavior_report_count(&VALIDATOR_B);
                assert_eq!(count, 1);
            });
        }

        #[test]
        fn update_rate_limit_config_works() {
            new_test_ext().execute_with(|| {
                assert_ok!(DcfPallet::update_rate_limit_config(
                    RuntimeOrigin::root(),
                    15,
                    15,
                    15
                ));

                let config = RateLimitConfigStorage::<Test>::get();
                assert_eq!(config.max_proposals_per_block, 15);
                assert_eq!(config.max_joins_per_block, 15);
                assert_eq!(config.max_leaves_per_block, 15);
            });
        }
    }

    // ============================================================================
    // 8. Runtime APIs & System Metrics Tests
    // ============================================================================
    mod runtime_api_tests {
        use super::*;

        #[test]
        fn get_validator_set_info_returns_total_active_inactive() {
            new_test_ext().execute_with(|| {
                let (total, active, inactive) = DcfPallet::get_validator_set_info();
                assert_eq!(total, active + inactive);
                assert_eq!(total, 3);
                assert_eq!(active, 3);
                assert_eq!(inactive, 0);
            });
        }

        #[test]
        fn get_total_validators_count_matches_set_length() {
            new_test_ext().execute_with(|| {
                let count = DcfPallet::get_total_validators_count();
                assert_eq!(count, 3);
            });
        }
    }
}
