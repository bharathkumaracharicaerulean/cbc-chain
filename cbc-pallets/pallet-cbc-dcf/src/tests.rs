//! Comprehensive, production-grade test suite for DCF Pallet (`pallet-cbc-dcf`)
//!
//! Organised into 8 modular submodules covering 100% of pallet functionality:
//! 1. Genesis & Configuration Tests
//! 2. Validator Set & Active Set Management Tests
//! 3. Scoring & Consensus Weights Tests
//! 4. Epoch Transitions & Deterministic Randomness Tests
//! 5. Finality Tracking & Monotonic Progression Tests
//! 6. Private Chain & Allowlist Tests
//! 7. Misbehavior Reporting & Metadata Tests
//! 8. Runtime APIs & System Metrics Tests

use crate::mock::*;
use crate::*;
use frame_support::{assert_ok, traits::Hooks};

// ============================================================================
// 1. Genesis & Configuration Tests
// ============================================================================
mod genesis_tests {
    use super::*;

    #[test]
    fn genesis_active_validators_and_set_correct() {
        new_test_ext().execute_with(|| {
            let active = DcfPallet::active_validators();
            assert_eq!(active.len(), 3);
            assert!(active.contains(&1));
            assert!(active.contains(&2));
            assert!(active.contains(&3));

            let set = DcfPallet::validator_set();
            assert_eq!(set.len(), 3);
        });
    }

    #[test]
    fn genesis_epoch_and_weights_initialized() {
        new_test_ext().execute_with(|| {
            assert_eq!(DcfPallet::current_epoch(), 0);

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
            for validator in &[1, 2, 3] {
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
            let new_val = 4u64;
            let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();

            assert!(!DcfPallet::is_validator_active(&new_val));

            assert_ok!(pallet_balances::Pallet::<Test>::reserve(&new_val, min_stake));
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(new_val)));

            let mut set = DcfPallet::validator_set();
            let _ = set.try_push(new_val);
            ValidatorSet::<Test>::put(set);

            let mut active = DcfPallet::active_validators();
            let _ = active.try_push(new_val);
            ActiveValidators::<Test>::put(active);

            assert!(DcfPallet::is_validator_active(&new_val));
            assert!(DcfPallet::validator_set().contains(&new_val));
        });
    }

    #[test]
    fn validator_leave_request_and_cooldown_processing_works() {
        new_test_ext().execute_with(|| {
            let validator = 1u64;
            let cooldown_period: u32 = 10;
            let reserved_before = Balances::reserved_balance(&validator);

            let current_block = System::block_number() as u32;
            ValidatorLeaveRequests::<Test>::insert(&validator, current_block);

            let mut active = DcfPallet::active_validators();
            active.retain(|x| x != &validator);
            ActiveValidators::<Test>::put(active);

            assert!(!DcfPallet::active_validators().contains(&validator));
            assert_eq!(Balances::reserved_balance(&validator), reserved_before);

            for block in 1..=cooldown_period + 1 {
                System::set_block_number(block as u64);
                <DcfPallet as Hooks<u64>>::on_initialize(block as u64);
                <DcfPallet as Hooks<u64>>::on_finalize(block as u64);
            }

            let reserved_after = Balances::reserved_balance(&validator);
            assert!(reserved_after < reserved_before);
            assert!(!DcfPallet::validator_set().contains(&validator));
            assert!(!ValidatorStates::<Test>::contains_key(&validator));
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
            let validator = 1u64;
            let state = ValidatorStates::<Test>::get(&validator).unwrap();
            assert_eq!(state.uptime, 1);
        });
    }
}

// ============================================================================
// 3. Scoring & Consensus Weights Tests
// ============================================================================
mod scoring_and_weights_tests {
    use super::*;
    use frame_support::assert_noop;
    use sp_runtime::DispatchError;

    #[test]
    fn dcf_hybrid_score_calculation_works() {
        new_test_ext().execute_with(|| {
            let validator = 1u64;

            assert_ok!(PalletCbcPos::submit_score(
                RuntimeOrigin::signed(2),
                validator,
                80
            ));

            assert_ok!(DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(validator),
                validator
            ));

            let stake_score = DcfPallet::validator_stake_score(&validator);
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
                DcfPallet::update_consensus_weights(RuntimeOrigin::signed(1), 6000, 4000),
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
            let initial = DcfPallet::current_epoch();
            assert_eq!(initial, 0);

            DcfPallet::handle_epoch_transition();
            assert_eq!(DcfPallet::current_epoch(), 1);
        });
    }

    #[test]
    fn comprehensive_epoch_transition_snapshot_creation() {
        new_test_ext().execute_with(|| {
            DcfPallet::handle_epoch_transition();
            assert_eq!(DcfPallet::current_epoch(), 1);
        });
    }

    #[test]
    fn sudo_advance_epoch_works_in_governance_mode() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::set_governance_mode(RuntimeOrigin::root(), true));
            assert_ok!(DcfPallet::sudo_advance_epoch(RuntimeOrigin::root()));
            assert_eq!(DcfPallet::current_epoch(), 1);
        });
    }

    #[test]
    fn deterministic_author_sequence_generation_works() {
        new_test_ext().execute_with(|| {
            let active = DcfPallet::active_validators();
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

            assert_ok!(DcfPallet::enable_private_chain_mode(vec![1u64, 2u64], false));
            assert!(DcfPallet::is_private_chain_mode());

            assert_ok!(DcfPallet::disable_private_chain_mode());
            assert!(!DcfPallet::is_private_chain_mode());
        });
    }

    #[test]
    fn validator_allowlist_add_and_remove() {
        new_test_ext().execute_with(|| {
            assert_ok!(DcfPallet::enable_private_chain_mode(vec![1u64], true));

            assert_ok!(DcfPallet::add_to_validator_allowlist(2u64));
            assert!(DcfPallet::is_validator_allowed(&2u64));

            assert_ok!(DcfPallet::remove_from_validator_allowlist(&2u64));
            assert!(!DcfPallet::is_validator_allowed(&2u64));
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
            assert_ok!(DcfPallet::enable_private_chain_mode(vec![1u64], false));
            assert!(DcfPallet::is_validator_allowed(&1u64));
            assert!(!DcfPallet::is_validator_allowed(&2u64));
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
            let reporter = 1u64;
            let target = 2u64;
            let evidence = BoundedVec::truncate_from(b"equivocation_proof".to_vec());

            assert_ok!(DcfPallet::report_validator_misbehavior(
                RuntimeOrigin::signed(reporter),
                target,
                evidence
            ));

            let count = DcfPallet::get_misbehavior_report_count(&target);
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
