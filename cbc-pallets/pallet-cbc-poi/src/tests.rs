//! Production-grade unit test suite for `pallet-cbc-poi`
//!
//! Directly exercises all pallet extrinsics via `RuntimeOrigin::signed` and `RuntimeOrigin::root`.
//!
//! Submodules:
//! 1. Genesis Configuration Tests
//! 2. Inference Submission Extrinsic Tests
//! 3. Challenge Extrinsic Tests
//! 4. Simulate Inference Extrinsic Tests
//! 5. Off-Chain Worker Score Application Tests
//! 6. Scoring Logic & Handler Tests
//! 7. Off-Chain Storage & Helper Functions Tests

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        CurrentEpoch, Error, Event, GenesisConfig, InferenceErrorSeverity, ValidatorInferenceCount,
        InferenceResults, Challenges,
    };
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::DispatchError;

    // Named account constants
    const VALIDATOR_A: u64 = 1;
    const VALIDATOR_B: u64 = 2;
    const CHALLENGER: u64 = 3;
    const UNREGISTERED_VAL: u64 = 999;

    /// Helper function to advance block numbers for block-based state transitions
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
        use sp_runtime::BuildStorage;

        #[test]
        fn test_genesis_config_initialization() {
            let genesis = GenesisConfig::<Test> {
                inference_results: vec![(VALIDATOR_A, 42), (VALIDATOR_B, 85)],
                challenges: vec![(CHALLENGER, VALIDATOR_A, 42)],
                current_epoch: 5,
            };
            let mut storage = frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap();
            genesis.assimilate_storage(&mut storage).unwrap();

            let mut ext = sp_io::TestExternalities::from(storage);
            ext.execute_with(|| {
                assert_eq!(InferenceResults::<Test>::get(VALIDATOR_A), Some((42, 5)));
                assert_eq!(InferenceResults::<Test>::get(VALIDATOR_B), Some((85, 5)));
                assert_eq!(Challenges::<Test>::get(CHALLENGER), Some((VALIDATOR_A, 42, 5)));
                assert_eq!(CurrentEpoch::<Test>::get(), 5);
            });
        }
    }

    // ============================================================================
    // 2. Inference Submission Extrinsic Tests
    // ============================================================================
    mod submission_tests {
        use super::*;

        #[test]
        fn test_submit_inference_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                let (result, epoch) = InferenceResults::<Test>::get(VALIDATOR_A).unwrap();
                assert_eq!(result, 42);
                assert_eq!(epoch, 0);

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::PoiScoreUpdated {
                    validator: VALIDATOR_A,
                    old_score: 0,
                    new_score: 42,
                    epoch: 0,
                }));

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceSubmitted {
                    who: VALIDATOR_A,
                    result: 42,
                    confidence: 80,
                }));

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                    validator: VALIDATOR_A,
                    confidence: 80,
                }));
            });
        }

        #[test]
        fn test_submit_inference_confidence_tier_high() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    50,
                    95
                ));
                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                    validator: VALIDATOR_A,
                    confidence: 95,
                }));
            });
        }

        #[test]
        fn test_submit_inference_confidence_tier_medium() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    50,
                    75
                ));
                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                    validator: VALIDATOR_A,
                    confidence: 75,
                }));
            });
        }

        #[test]
        fn test_submit_inference_confidence_tier_low() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    50,
                    55
                ));
                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                    validator: VALIDATOR_A,
                    confidence: 55,
                }));
            });
        }

        #[test]
        fn test_submit_inference_confidence_too_low() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::submit_inference(RuntimeOrigin::signed(VALIDATOR_A), 42, 20),
                    Error::<Test>::ConfidenceTooLow
                );
            });
        }

        #[test]
        fn test_submit_inference_already_submitted() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                assert_noop!(
                    PalletCbcPoi::submit_inference(RuntimeOrigin::signed(VALIDATOR_A), 43, 80),
                    Error::<Test>::InferenceAlreadySubmitted
                );
            });
        }

        #[test]
        fn test_submit_inference_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::submit_inference(RuntimeOrigin::root(), 42, 80),
                    DispatchError::BadOrigin
                );
            });
        }
    }

    // ============================================================================
    // 3. Challenge Extrinsic Tests
    // ============================================================================
    mod challenge_tests {
        use super::*;

        #[test]
        fn test_challenge_inference_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));
                assert_ok!(PalletCbcPoi::challenge_inference(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    VALIDATOR_A,
                    42
                ));

                let (challenged, result, epoch) = Challenges::<Test>::get(VALIDATOR_B).unwrap();
                assert_eq!(challenged, VALIDATOR_A);
                assert_eq!(result, 42);
                assert_eq!(epoch, 0);

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceChallenged {
                    challenger: VALIDATOR_B,
                    challenged: VALIDATOR_A,
                    result: 42,
                }));

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceRejected {
                    validator: VALIDATOR_A,
                    confidence: 0,
                }));

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::ValidatorSlashed {
                    validator: VALIDATOR_A,
                    reason: b"Invalid inference".to_vec(),
                }));
            });
        }

        #[test]
        fn test_challenge_inference_self_challenge() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_A), VALIDATOR_A, 42),
                    Error::<Test>::CannotChallengeSelf
                );
            });
        }

        #[test]
        fn test_challenge_inference_already_challenged() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));
                assert_ok!(PalletCbcPoi::challenge_inference(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    VALIDATOR_A,
                    42
                ));

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_B), VALIDATOR_A, 42),
                    Error::<Test>::ChallengeAlreadyExists
                );
            });
        }

        #[test]
        fn test_challenge_inference_not_found() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_B), UNREGISTERED_VAL, 42),
                    Error::<Test>::InferenceNotFound
                );
            });
        }

        #[test]
        fn test_challenge_inference_invalid_result() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_B), VALIDATOR_A, 43),
                    Error::<Test>::InvalidChallenge
                );
            });
        }

        #[test]
        fn test_challenge_inference_too_old() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));
                CurrentEpoch::<Test>::put(15);

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_B), VALIDATOR_A, 42),
                    Error::<Test>::InferenceTooOld
                );
            });
        }

        #[test]
        fn test_challenge_inference_window_expired() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));
                CurrentEpoch::<Test>::put(6);

                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(VALIDATOR_B), VALIDATOR_A, 42),
                    Error::<Test>::ChallengeWindowExpired
                );
            });
        }

        #[test]
        fn test_challenge_inference_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::challenge_inference(RuntimeOrigin::root(), VALIDATOR_A, 42),
                    DispatchError::BadOrigin
                );
            });
        }
    }

    // ============================================================================
    // 4. Simulate Inference Extrinsic Tests
    // ============================================================================
    mod simulation_tests {
        use super::*;

        #[test]
        fn test_simulate_inference_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::simulate_inference(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    VALIDATOR_A
                ));

                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 1);
                System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceSubmitted {
                    who: VALIDATOR_A,
                    result: 72,
                    confidence: 89,
                }));
            });
        }

        #[test]
        fn test_simulate_inference_validator_not_found() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::simulate_inference(RuntimeOrigin::signed(VALIDATOR_A), UNREGISTERED_VAL),
                    Error::<Test>::ValidatorNotFound
                );
            });
        }

        #[test]
        fn test_simulate_inference_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcPoi::simulate_inference(RuntimeOrigin::root(), VALIDATOR_A),
                    DispatchError::BadOrigin
                );
            });
        }
    }

    // ============================================================================
    // 5. Off-Chain Worker Score Application Tests
    // ============================================================================
    mod offchain_score_tests {
        use super::*;

        #[test]
        fn test_apply_offchain_poi_scores_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcPoi::submit_score(VALIDATOR_A, 800, 1));

                assert_ok!(PalletCbcPoi::apply_offchain_poi_scores(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    1
                ));

                let (score, epoch) = InferenceResults::<Test>::get(VALIDATOR_A).unwrap();
                assert_eq!(score, 800);
                assert_eq!(epoch, 0);

                System::assert_has_event(RuntimeEvent::PalletCbcPoi(
                    Event::ValidatorPoiScoreUpdated {
                        validator: VALIDATOR_A,
                        poi_score: 800,
                    },
                ));
            });
        }

        #[test]
        fn test_update_validator_inference_score_success() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                assert_ok!(PalletCbcPoi::update_validator_inference_score(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    VALIDATOR_A
                ));
            });
        }

        #[test]
        fn test_update_validator_inference_score_not_found() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::update_validator_inference_score(
                    RuntimeOrigin::signed(VALIDATOR_B),
                    UNREGISTERED_VAL
                ));
            });
        }
    }

    // ============================================================================
    // 6. Scoring Logic & Handler Tests
    // ============================================================================
    mod scoring_handler_tests {
        use super::*;

        #[test]
        fn test_handle_valid_inference_high_confidence() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::handle_valid_inference(&VALIDATOR_A, 95));
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 1);
            });
        }

        #[test]
        fn test_handle_valid_inference_medium_confidence() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::handle_valid_inference(&VALIDATOR_A, 75));
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 1);
            });
        }

        #[test]
        fn test_handle_valid_inference_low_confidence() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::handle_valid_inference(&VALIDATOR_A, 60));
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 1);
            });
        }

        #[test]
        fn test_handle_invalid_inference_severities() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::handle_invalid_inference(
                    &VALIDATOR_A,
                    InferenceErrorSeverity::High
                ));
                assert_ok!(PalletCbcPoi::handle_invalid_inference(
                    &VALIDATOR_A,
                    InferenceErrorSeverity::Medium
                ));
                assert_ok!(PalletCbcPoi::handle_invalid_inference(
                    &VALIDATOR_A,
                    InferenceErrorSeverity::Low
                ));
            });
        }
    }

    // ============================================================================
    // 7. Off-Chain Storage & Helper Functions Tests
    // ============================================================================
    mod helper_tests {
        use super::*;

        #[test]
        fn test_get_score_and_inference_score() {
            new_test_ext().execute_with(|| {
                assert_eq!(PalletCbcPoi::get_score(&VALIDATOR_A), 0);
                assert_eq!(PalletCbcPoi::get_poi_score(&VALIDATOR_A), 0);
                assert_eq!(PalletCbcPoi::get_inference_score(&VALIDATOR_A), 0);

                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    88,
                    90
                ));

                assert_eq!(PalletCbcPoi::get_score(&VALIDATOR_A), 88);
                assert_eq!(PalletCbcPoi::get_poi_score(&VALIDATOR_A), 88);
                assert_eq!(PalletCbcPoi::get_inference_score(&VALIDATOR_A), 88);
                assert_eq!(PalletCbcPoi::validator_inference_score(&VALIDATOR_A), 88);
            });
        }

        #[test]
        fn test_submit_and_verify_score() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_score(VALIDATOR_A, 500, 10));
                assert!(PalletCbcPoi::verify_score(&VALIDATOR_A, 500, 10));
                assert!(!PalletCbcPoi::verify_score(&VALIDATOR_A, 500, 11));
                assert!(!PalletCbcPoi::verify_score(&VALIDATOR_A, 600, 10));
            });
        }

        #[test]
        fn test_record_inference_activity() {
            new_test_ext().execute_with(|| {
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 0);
                assert_ok!(PalletCbcPoi::record_inference_activity(&VALIDATOR_A));
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 1);
                assert_ok!(PalletCbcPoi::record_inference_activity(&VALIDATOR_A));
                assert_eq!(ValidatorInferenceCount::<Test>::get(VALIDATOR_A), 2);
            });
        }

        #[test]
        fn test_collect_inference_data_stored() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::submit_inference(
                    RuntimeOrigin::signed(VALIDATOR_A),
                    42,
                    80
                ));

                let data = PalletCbcPoi::collect_inference_data(&VALIDATOR_A, 0).unwrap();
                assert_eq!(data.validator, VALIDATOR_A);
                assert_eq!(data.epoch, 0);
                assert_eq!(data.inference_result, 42);
                assert_eq!(data.confidence_score, 0);
                assert_eq!(data.data_sources, vec![b"poi_pallet".to_vec()]);
            });
        }

        #[test]
        fn test_collect_inference_data_external_fallback() {
            new_test_ext().execute_with(|| {
                let data = PalletCbcPoi::collect_inference_data(&VALIDATOR_A, 0).unwrap();
                assert_eq!(data.validator, VALIDATOR_A);
                assert_eq!(data.epoch, 0);
                assert_eq!(data.data_sources, vec![b"simulation".to_vec()]);
            });
        }

        #[test]
        fn test_run_offchain_computation() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcPoi::run_offchain_computation(VALIDATOR_A));
            });
        }
    }
}