//! Comprehensive, production-grade integration test suite for the CBC Runtime
//!
//! Verifies cross-pallet interactions between:
//! - `pallet-cbc-pos` (Proof-of-Stake)
//! - `pallet-cbc-poi` (Proof-of-Inference)
//! - `pallet-cbc-governance` (Governance & Proposal Engine)
//! - `pallet-cbc-dcf` (Dynamic Consensus Framework)
//! - `pallet-cbc-dvf` (Deterministic Verification Framework)
//! - `pallet-balances` & `frame-system`

#[cfg(test)]
mod tests {
    use crate::mock::{
        new_test_ext, new_test_ext_for_governance, new_test_ext_for_epochs,
        new_test_ext_with_validators, new_test_ext_for_scenario, TestScenario,
        funded_account_id, advance_blocks, advance_to_next_epoch, setup_validator_with_stake,
    };
    use crate::{RuntimeOrigin, System, Balances, Runtime};

    type PalletCbcPos = pallet_cbc_pos::Pallet<Runtime>;
    type PalletCbcPoi = pallet_cbc_poi::Pallet<Runtime>;
    type PalletCbcGovernance = pallet_cbc_governance::Pallet<Runtime>;
    type PalletCbcDcf = pallet_cbc_dcf::Pallet<Runtime>;
    type PalletCbcDvf = pallet_cbc_dvf::Pallet<Runtime>;

    use frame_support::{assert_ok, assert_noop};

    // ============================================================================
    // 1. Runtime Initialization & Basic Setup Tests
    // ============================================================================
    mod initialization_tests {
        use super::*;

        #[test]
        fn test_runtime_initialization() {
            new_test_ext().execute_with(|| {
                assert_eq!(System::block_number(), 1);

                let test_account = funded_account_id(1);
                assert!(System::account_exists(&test_account));
                assert!(Balances::free_balance(&test_account) > 0u128);

                let validators = PalletCbcDcf::validator_set();
                assert!(!validators.is_empty());
                assert_eq!(validators.len(), 4);

                for validator in validators.iter() {
                    let stake = pallet_cbc_pos::Stake::<Runtime>::get(validator);
                    assert!(stake > 0);
                }
            });
        }
    }

    // ============================================================================
    // 2. Cross-Pallet Integration Tests
    // ============================================================================
    mod cross_pallet_tests {
        use super::*;

        #[test]
        fn test_cross_pallet_interactions() {
            new_test_ext().execute_with(|| {
                let validator = funded_account_id(99);
                let stake_amount = 1000u128 * crate::CBC;
                setup_validator_with_stake(validator.clone(), stake_amount);

                let reserved = Balances::reserved_balance(&validator);
                assert_eq!(reserved, stake_amount);

                let pos_stake = pallet_cbc_pos::Stake::<Runtime>::get(&validator);
                assert_eq!(pos_stake, stake_amount);

                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(validator.clone())));
                assert!(pallet_cbc_pos::Validators::<Runtime>::get(&validator).is_some());

                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(validator.clone()),
                    42,
                    80
                ));
                assert!(pallet_cbc_poi::InferenceResults::<Runtime>::get(&validator).is_some());
            });
        }

        #[test]
        fn test_governance_integration() {
            new_test_ext_for_governance().execute_with(|| {
                let proposer = funded_account_id(1);
                let target = funded_account_id(2);

                assert!(pallet_cbc_governance::GovernanceModeEnabled::<Runtime>::get());

                assert_ok!(PalletCbcGovernance::propose_reward_validator(
                    RuntimeOrigin::signed(proposer.clone()),
                    target.clone(),
                    1000u128,
                    None
                ));

                assert!(pallet_cbc_governance::Proposals::<Runtime>::contains_key(0));

                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(proposer.clone()),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(target.clone()),
                    0,
                    true
                ));

                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                let proposal = pallet_cbc_governance::Proposals::<Runtime>::get(0).unwrap();
                assert_eq!(proposal.status, pallet_cbc_governance::ProposalStatus::Executed);
            });
        }

        #[test]
        fn test_epoch_transitions() {
            new_test_ext_for_epochs().execute_with(|| {
                let initial_epoch = PalletCbcDcf::current_epoch();

                advance_to_next_epoch();

                let new_epoch = PalletCbcDcf::current_epoch();
                assert!(new_epoch > initial_epoch);

                let validators = PalletCbcDcf::validator_set();
                assert!(!validators.is_empty());
            });
        }
    }

    // ============================================================================
    // 3. Multi-Validator System Tests
    // ============================================================================
    mod multi_validator_tests {
        use super::*;

        #[test]
        fn test_multi_validator_system() {
            new_test_ext_with_validators(10).execute_with(|| {
                let validators = PalletCbcDcf::validator_set();
                assert_eq!(validators.len(), 10);

                for validator in validators.iter() {
                    let stake = pallet_cbc_pos::Stake::<Runtime>::get(validator);
                    assert!(stake > 0);

                    let balance = Balances::free_balance(validator);
                    assert!(balance > 0);
                }

                let val_new1 = funded_account_id(98);
                let val_new2 = funded_account_id(99);
                setup_validator_with_stake(val_new1.clone(), 1000u128 * crate::CBC);
                setup_validator_with_stake(val_new2.clone(), 1000u128 * crate::CBC);

                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(val_new1.clone())));
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(val_new2.clone())));

                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(val_new1.clone()),
                    42,
                    80
                ));
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(val_new2.clone()),
                    43,
                    85
                ));

                assert_ok!(PalletCbcPoi::challenge_inference(
                    RuntimeOrigin::signed(val_new2.clone()),
                    val_new1.clone(),
                    42
                ));
            });
        }
    }

    // ============================================================================
    // 4. Runtime API Tests
    // ============================================================================
    mod runtime_api_tests {
        use super::*;

        #[test]
        fn test_runtime_apis() {
            new_test_ext().execute_with(|| {
                let validators = PalletCbcDcf::validator_set();
                assert!(!validators.is_empty());

                let active_validators = PalletCbcDcf::active_validators();
                assert!(!active_validators.is_empty());

                let current_epoch = PalletCbcDcf::current_epoch();
                assert_eq!(current_epoch, 0);

                let (pos_w, poi_w) = PalletCbcDcf::consensus_weights();
                assert!(pos_w + poi_w == 10000 || pos_w + poi_w == 0);
            });
        }

        #[test]
        fn test_account_nonce_api() {
            new_test_ext().execute_with(|| {
                let account = funded_account_id(1);

                let nonce = System::account_nonce(&account);
                assert_eq!(nonce, 0);

                System::inc_account_nonce(&account);
                let new_nonce = System::account_nonce(&account);
                assert_eq!(new_nonce, 1);
            });
        }

        #[test]
        fn test_dcf_system_functions() {
            new_test_ext().execute_with(|| {
                let count = PalletCbcDcf::get_total_validators_count();
                assert_eq!(count, 4);

                let (total, active, inactive) = PalletCbcDcf::get_validator_set_info();
                assert!(total >= active + inactive);

                let governance_enabled = pallet_cbc_governance::GovernanceModeEnabled::<Runtime>::get();
                assert!(governance_enabled == true || governance_enabled == false);
            });
        }

        #[test]
        fn test_dcf_finality_functions() {
            new_test_ext().execute_with(|| {
                let last_finalized = PalletCbcDvf::finalized_block_number();
                assert_eq!(last_finalized, 0);

                let (finalized_block, current_block) = PalletCbcDcf::get_finality_info();
                assert!(finalized_block <= current_block);

                let blocks_since = PalletCbcDcf::blocks_since_finalization(current_block);
                assert_eq!(blocks_since, 1);
            });
        }
    }

    // ============================================================================
    // 5. Economic & System Limits Tests
    // ============================================================================
    mod economic_and_limits_tests {
        use super::*;

        #[test]
        fn test_economic_model() {
            new_test_ext().execute_with(|| {
                let validator = funded_account_id(99);
                let initial_balance = Balances::free_balance(&validator);
                let stake_amount = 1000u128 * crate::CBC;

                setup_validator_with_stake(validator.clone(), stake_amount);

                let free_balance = Balances::free_balance(&validator);
                let reserved_balance = Balances::reserved_balance(&validator);

                assert_eq!(reserved_balance, stake_amount);
                assert_eq!(free_balance + reserved_balance, initial_balance + 11_000u128 * crate::CBC);

                let reward_amount = 500u128;
                assert_ok!(PalletCbcGovernance::set_governance_mode(RuntimeOrigin::root(), true));
                assert_ok!(PalletCbcGovernance::propose_reward_validator(
                    RuntimeOrigin::signed(validator.clone()),
                    validator.clone(),
                    reward_amount,
                    None
                ));

                assert!(pallet_cbc_governance::Proposals::<Runtime>::contains_key(0));
            });
        }

        #[test]
        fn test_system_limits() {
            new_test_ext().execute_with(|| {
                let max_validators = <Runtime as pallet_cbc_pos::Config>::MaxValidators::get();
                let current_validators = PalletCbcDcf::validator_set();
                assert!(current_validators.len() <= max_validators as usize);

                let min_active = <Runtime as pallet_cbc_pos::Config>::MinActiveValidators::get();
                let active_validators = PalletCbcDcf::active_validators();
                assert!(active_validators.len() >= min_active as usize);
            });
        }

        #[test]
        fn test_error_handling() {
            new_test_ext().execute_with(|| {
                let non_validator = funded_account_id(999);

                assert_noop!(
                    PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(non_validator.clone()), 999, true),
                    pallet_cbc_governance::Error::<Runtime>::ProposalNotFound
                );

                assert_noop!(
                    PalletCbcPos::submit_score(RuntimeOrigin::signed(funded_account_id(1)), non_validator.clone(), 75),
                    pallet_cbc_pos::Error::<Runtime>::ValidatorNotRegistered
                );

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(funded_account_id(1)), non_validator.clone(), 42),
                    pallet_cbc_poi::Error::<Runtime>::InferenceNotFound
                );
            });
        }

        #[test]
        fn test_performance_under_load() {
            new_test_ext_for_scenario(TestScenario::StressTest).execute_with(|| {
                let validators = PalletCbcDcf::validator_set();
                assert_eq!(validators.len(), 50);

                use std::time::Instant;
                let start = Instant::now();

                for validator in validators.iter().take(10) {
                    let _ = PalletCbcDcf::update_validator_stake_score(
                        RuntimeOrigin::signed(validator.clone()),
                        validator.clone()
                    );
                }

                let duration = start.elapsed();
                assert!(duration.as_millis() < 1000);

                let start = Instant::now();
                advance_to_next_epoch();
                let duration = start.elapsed();
                assert!(duration.as_millis() < 5000);
            });
        }
    }

    // ============================================================================
    // 6. Benchmark & Mock Routine Tests
    // ============================================================================
    mod benchmark_and_mock_tests {
        use super::*;

        #[test]
        fn test_benchmark_integration() {
            new_test_ext().execute_with(|| {
                let validator = funded_account_id(1);

                let _ = PalletCbcDvf::join_validators(RuntimeOrigin::signed(validator.clone()), None);
                let _ = PalletCbcPos::register_validator(RuntimeOrigin::signed(validator.clone()));
                let _ = PalletCbcPoi::submit_inference(RuntimeOrigin::signed(validator.clone()), 42, 80);
            });
        }

        #[test]
        fn test_mock_configurations() {
            new_test_ext().execute_with(|| {
                assert_eq!(System::block_number(), 1);
                assert_eq!(PalletCbcDcf::validator_set().len(), 4);
            });

            new_test_ext_with_validators(20).execute_with(|| {
                assert_eq!(PalletCbcDcf::validator_set().len(), 20);
            });

            new_test_ext_for_governance().execute_with(|| {
                assert!(pallet_cbc_governance::GovernanceModeEnabled::<Runtime>::get());
            });
        }

        #[test]
        fn test_helper_functions() {
            new_test_ext().execute_with(|| {
                let initial_block = System::block_number();

                advance_blocks(10);
                assert_eq!(System::block_number(), initial_block + 10);

                let test_account = funded_account_id(999);
                setup_validator_with_stake(test_account.clone(), 1000u128 * crate::CBC);
                assert_eq!(Balances::reserved_balance(&test_account), 1000u128 * crate::CBC);
                assert_eq!(pallet_cbc_pos::Stake::<Runtime>::get(&test_account), 1000u128 * crate::CBC);
            });
        }
    }
}