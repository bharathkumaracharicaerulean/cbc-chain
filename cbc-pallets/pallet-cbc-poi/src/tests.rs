#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        CurrentEpoch, Error, Event, GenesisConfig, InferenceErrorSeverity, ValidatorInferenceCount,
    };
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::{BuildStorage, DispatchError};

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
        let genesis = GenesisConfig::<Test> {
            inference_results: vec![(1, 42), (2, 85)],
            challenges: vec![(3, 1, 42)],
            current_epoch: 5,
        };
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        genesis.assimilate_storage(&mut storage).unwrap();

        let mut ext = sp_io::TestExternalities::from(storage);
        ext.execute_with(|| {
            assert_eq!(PalletCbcPoi::inference_results(1), Some((42, 5)));
            assert_eq!(PalletCbcPoi::inference_results(2), Some((85, 5)));
            assert_eq!(PalletCbcPoi::challenges(3), Some((1, 42, 5)));
            assert_eq!(PalletCbcPoi::current_epoch(), 5);
        });
    }

    // ================================================================================================
    // 2. Inference Submission Extrinsic Tests
    // ================================================================================================

    #[test]
    fn test_submit_inference_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            let (result, epoch) = PalletCbcPoi::inference_results(1).unwrap();
            assert_eq!(result, 42);
            assert_eq!(epoch, 0);

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::PoiScoreUpdated {
                validator: 1,
                old_score: 0,
                new_score: 42,
                epoch: 0,
            }));

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceSubmitted {
                who: 1,
                result: 42,
                confidence: 80,
            }));

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                validator: 1,
                confidence: 80,
            }));
        });
    }

    #[test]
    fn test_submit_inference_confidence_tier_high() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                50,
                95
            ));
            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                validator: 1,
                confidence: 95,
            }));
        });
    }

    #[test]
    fn test_submit_inference_confidence_tier_medium() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                50,
                75
            ));
            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                validator: 1,
                confidence: 75,
            }));
        });
    }

    #[test]
    fn test_submit_inference_confidence_tier_low() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                50,
                55
            ));
            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceAccepted {
                validator: 1,
                confidence: 55,
            }));
        });
    }

    #[test]
    fn test_submit_inference_confidence_too_low() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 42, 20),
                Error::<Test>::ConfidenceTooLow
            );
        });
    }

    #[test]
    fn test_submit_inference_already_submitted() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_noop!(
                PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 43, 80),
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

    // ================================================================================================
    // 3. Challenge Extrinsic Tests
    // ================================================================================================

    #[test]
    fn test_challenge_inference_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(2),
                1,
                42
            ));

            let (challenged, result, epoch) = PalletCbcPoi::challenges(2).unwrap();
            assert_eq!(challenged, 1);
            assert_eq!(result, 42);
            assert_eq!(epoch, 0);

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceChallenged {
                challenger: 2,
                challenged: 1,
                result: 42,
            }));

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceRejected {
                validator: 1,
                confidence: 0,
            }));

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::ValidatorSlashed {
                validator: 1,
                reason: b"Invalid inference".to_vec(),
            }));
        });
    }

    #[test]
    fn test_challenge_inference_self_challenge() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(1), 1, 42),
                Error::<Test>::CannotChallengeSelf
            );
        });
    }

    #[test]
    fn test_challenge_inference_already_challenged() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(2),
                1,
                42
            ));

            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 42),
                Error::<Test>::ChallengeAlreadyExists
            );
        });
    }

    #[test]
    fn test_challenge_inference_not_found() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 999, 42),
                Error::<Test>::InferenceNotFound
            );
        });
    }

    #[test]
    fn test_challenge_inference_invalid_result() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 43),
                Error::<Test>::InvalidChallenge
            );
        });
    }

    #[test]
    fn test_challenge_inference_too_old() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            // Set current epoch past MaxInferenceAge (10)
            CurrentEpoch::<Test>::put(15);

            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 42),
                Error::<Test>::InferenceTooOld
            );
        });
    }

    #[test]
    fn test_challenge_inference_window_expired() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            // Set current epoch past ChallengeWindow (5) but <= MaxInferenceAge (10)
            CurrentEpoch::<Test>::put(6);

            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 42),
                Error::<Test>::ChallengeWindowExpired
            );
        });
    }

    #[test]
    fn test_challenge_inference_bad_origin() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::root(), 1, 42),
                DispatchError::BadOrigin
            );
        });
    }

    // ================================================================================================
    // 4. Simulate Inference Extrinsic Tests
    // ================================================================================================

    #[test]
    fn test_simulate_inference_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            // Validator 1 is active in DummyPosInterface
            assert_ok!(PalletCbcPoi::simulate_inference(
                RuntimeOrigin::signed(2),
                1
            ));

            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 1);
            System::assert_has_event(RuntimeEvent::PalletCbcPoi(Event::InferenceSubmitted {
                who: 1,
                result: 72,
                confidence: 89,
            }));
        });
    }

    #[test]
    fn test_simulate_inference_validator_not_found() {
        new_test_ext().execute_with(|| {
            // Validator 999 is not in active validators [1, 2, 3]
            assert_noop!(
                PalletCbcPoi::simulate_inference(RuntimeOrigin::signed(1), 999),
                Error::<Test>::ValidatorNotFound
            );
        });
    }

    #[test]
    fn test_simulate_inference_bad_origin() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcPoi::simulate_inference(RuntimeOrigin::root(), 1),
                DispatchError::BadOrigin
            );
        });
    }

    // ================================================================================================
    // 5. Off-Chain Worker Score Application Tests
    // ================================================================================================

    #[test]
    fn test_apply_offchain_poi_scores_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            // Store offchain score for validator 1 at block 1
            assert_ok!(PalletCbcPoi::submit_score(1, 800, 1));

            assert_ok!(PalletCbcPoi::apply_offchain_poi_scores(
                RuntimeOrigin::signed(2),
                1
            ));

            let (score, epoch) = PalletCbcPoi::inference_results(1).unwrap();
            assert_eq!(score, 800);
            assert_eq!(epoch, 0);

            System::assert_has_event(RuntimeEvent::PalletCbcPoi(
                Event::ValidatorPoiScoreUpdated {
                    validator: 1,
                    poi_score: 800,
                },
            ));
        });
    }

    #[test]
    fn test_update_validator_inference_score_success() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            assert_ok!(PalletCbcPoi::update_validator_inference_score(
                RuntimeOrigin::signed(2),
                1
            ));
        });
    }

    #[test]
    fn test_update_validator_inference_score_not_found() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::update_validator_inference_score(
                RuntimeOrigin::signed(2),
                999
            ));
        });
    }

    // ================================================================================================
    // 6. Scoring Logic & Handler Tests
    // ================================================================================================

    #[test]
    fn test_handle_valid_inference_high_confidence() {
        new_test_ext().execute_with(|| {
            // High confidence >= 90
            assert_ok!(PalletCbcPoi::handle_valid_inference(&1, 95));
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 1);
        });
    }

    #[test]
    fn test_handle_valid_inference_medium_confidence() {
        new_test_ext().execute_with(|| {
            // Medium confidence >= 70, < 90
            assert_ok!(PalletCbcPoi::handle_valid_inference(&1, 75));
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 1);
        });
    }

    #[test]
    fn test_handle_valid_inference_low_confidence() {
        new_test_ext().execute_with(|| {
            // Low confidence < 70
            assert_ok!(PalletCbcPoi::handle_valid_inference(&1, 60));
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 1);
        });
    }

    #[test]
    fn test_handle_invalid_inference_severities() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::handle_invalid_inference(
                &1,
                InferenceErrorSeverity::High
            ));
            assert_ok!(PalletCbcPoi::handle_invalid_inference(
                &1,
                InferenceErrorSeverity::Medium
            ));
            assert_ok!(PalletCbcPoi::handle_invalid_inference(
                &1,
                InferenceErrorSeverity::Low
            ));
        });
    }

    // ================================================================================================
    // 7. Off-Chain Storage & Helper Functions Tests
    // ================================================================================================

    #[test]
    fn test_get_score_and_inference_score() {
        new_test_ext().execute_with(|| {
            assert_eq!(PalletCbcPoi::get_score(&1), 0);
            assert_eq!(PalletCbcPoi::get_poi_score(&1), 0);
            assert_eq!(PalletCbcPoi::get_inference_score(&1), 0);

            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                88,
                90
            ));

            assert_eq!(PalletCbcPoi::get_score(&1), 88);
            assert_eq!(PalletCbcPoi::get_poi_score(&1), 88);
            assert_eq!(PalletCbcPoi::get_inference_score(&1), 88);
            assert_eq!(PalletCbcPoi::validator_inference_score(&1), 88);
        });
    }

    #[test]
    fn test_submit_and_verify_score() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_score(1, 500, 10));
            assert!(PalletCbcPoi::verify_score(&1, 500, 10));
            assert!(!PalletCbcPoi::verify_score(&1, 500, 11));
            assert!(!PalletCbcPoi::verify_score(&1, 600, 10));
        });
    }

    #[test]
    fn test_record_inference_activity() {
        new_test_ext().execute_with(|| {
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 0);
            assert_ok!(PalletCbcPoi::record_inference_activity(&1));
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 1);
            assert_ok!(PalletCbcPoi::record_inference_activity(&1));
            assert_eq!(ValidatorInferenceCount::<Test>::get(1), 2);
        });
    }

    #[test]
    fn test_collect_inference_data_stored() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            let data = PalletCbcPoi::collect_inference_data(&1, 0).unwrap();
            assert_eq!(data.validator, 1);
            assert_eq!(data.epoch, 0);
            assert_eq!(data.inference_result, 42);
            assert_eq!(data.confidence_score, 0);
            assert_eq!(data.data_sources, vec![b"poi_pallet".to_vec()]);
        });
    }

    #[test]
    fn test_collect_inference_data_external_fallback() {
        new_test_ext().execute_with(|| {
            // Validator 1 has no stored inference result, falls back to external simulation
            let data = PalletCbcPoi::collect_inference_data(&1, 0).unwrap();
            assert_eq!(data.validator, 1);
            assert_eq!(data.epoch, 0);
            assert_eq!(data.data_sources, vec![b"simulation".to_vec()]);
        });
    }

    #[test]
    fn test_run_offchain_computation() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcPoi::run_offchain_computation(1));
        });
    }
}