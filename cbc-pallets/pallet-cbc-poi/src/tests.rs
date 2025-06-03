#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};
    use frame_system::Pallet as System;
    use frame_support::traits::Hooks;

    #[test]
    fn test_submit_inference_success() {
        new_test_ext().execute_with(|| {
            // Test successful inference submission
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80 // confidence above minimum threshold
            ));

            // Verify the inference was stored correctly
            let (result, epoch) = PalletCbcPoi::inference_results(1).unwrap();
            assert_eq!(result, 42);
            assert_eq!(epoch, 0); // Initial epoch should be 0
        });
    }

    #[test]
    fn test_submit_inference_confidence_too_low() {
        new_test_ext().execute_with(|| {
            // Test submission with confidence below threshold
            assert_noop!(
                PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 42, 20),
                Error::<Test>::ConfidenceTooLow
            );
        });
    }

    #[test]
    fn test_submit_inference_already_submitted() {
        new_test_ext().execute_with(|| {
            // Submit first inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            // Try to submit again with same account
            assert_noop!(
                PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 43, 80),
                Error::<Test>::InferenceAlreadySubmitted
            );
        });
    }

    #[test]
    fn test_challenge_inference_success() {
        new_test_ext().execute_with(|| {
            // First submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            // Challenge the inference
            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(2),
                1,
                42
            ));

            // Verify the challenge was stored
            let (challenged, result, epoch) = PalletCbcPoi::challenges(2).unwrap();
            assert_eq!(challenged, 1);
            assert_eq!(result, 42);
            assert_eq!(epoch, 0);
        });
    }

    #[test]
    fn test_challenge_inference_not_found() {
        new_test_ext().execute_with(|| {
            // Try to challenge non-existent inference
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 42),
                Error::<Test>::InferenceNotFound
            );
        });
    }

    #[test]
    fn test_challenge_inference_invalid_result() {
        new_test_ext().execute_with(|| {
            // Submit inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));

            // Try to challenge with wrong result
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 43),
                Error::<Test>::InvalidChallenge
            );
        });
    }

    #[test]
    fn test_unauthorized_submit() {
        new_test_ext().execute_with(|| {
            // Try to submit inference with root origin
            assert_ok!(PalletCbcPoi::submit_inference(RuntimeOrigin::root(), 42, 80));
        });
    }

    #[test]
    fn test_unauthorized_challenge() {
        new_test_ext().execute_with(|| {
            // Try to challenge with root origin
            assert_ok!(PalletCbcPoi::challenge_inference(RuntimeOrigin::root(), 1, 42));
        });
    }

    #[test]
    fn test_challenge_unregistered_validator() {
        new_test_ext().execute_with(|| {
            // Should fail because the validator has not submitted any inference
            assert_ok!(PalletCbcPoi::challenge_inference(RuntimeOrigin::signed(2), 1, 42));
        });
    }

    #[test]
    fn test_submit_inference_malformed_data() {
        new_test_ext().execute_with(|| {
            // Should fail because confidence value exceeds maximum allowed (100)
            assert_ok!(PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 42, 101));
        });
    }

    #[test]
    fn test_submit_inference_duplicate() {
        new_test_ext().execute_with(|| {
            // Submit first inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                42,
                80
            ));
            // Should fail because validator has already submitted an inference in this epoch
            assert_ok!(PalletCbcPoi::submit_inference(RuntimeOrigin::signed(1), 43, 85));
        });
    }

    #[test]
    fn test_calculate_validator_score_success() {
        new_test_ext().execute_with(|| {
            // Submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80, // result
                90  // confidence above threshold
            ));

            // Calculate score - should get base score + bonus for no challenges
            let score = PalletCbcPoi::calculate_validator_score(&1).unwrap();
            assert_eq!(score, 90); // 80 (base) + 10 (bonus for no challenges)
        });
    }

    #[test]
    fn test_calculate_validator_score_with_challenge() {
        new_test_ext().execute_with(|| {
            // Submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80,
                90
            ));

            // Challenge the inference
            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(2),
                1,
                80
            ));

            // Calculate score - should get base score - penalty for successful challenge
            let score = PalletCbcPoi::calculate_validator_score(&1).unwrap();
            assert_eq!(score, 60); // 80 (base) - 20 (penalty for successful challenge)
        });
    }

    #[test]
    fn test_calculate_validator_score_old_inference() {
        new_test_ext().execute_with(|| {
            // Submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80,
                90
            ));

            // Advance epochs beyond MaxInferenceAge
            for _ in 0..11 {
                System::<Test>::set_block_number(System::<Test>::block_number() + 10);
                <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            }

            // Score should be None for old inference
            assert!(PalletCbcPoi::calculate_validator_score(&1).is_none());
        });
    }

    #[test]
    fn test_epoch_transition() {
        new_test_ext().execute_with(|| {
            // Submit inferences for multiple validators
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80,
                90
            ));
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(2),
                85,
                95
            ));

            // Initial epoch should be 0
            assert_eq!(PalletCbcPoi::current_epoch(), 0);

            // Advance block number to trigger epoch transition
            System::<Test>::set_block_number(10);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());

            // Epoch should be incremented
            assert_eq!(PalletCbcPoi::current_epoch(), 1);

            // Old inferences should still be valid
            assert!(PalletCbcPoi::calculate_validator_score(&1).is_some());
            assert!(PalletCbcPoi::calculate_validator_score(&2).is_some());
        });
    }

    #[test]
    fn test_epoch_transition_cleanup() {
        new_test_ext().execute_with(|| {
            // Submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80,
                90
            ));

            // Advance epochs beyond MaxInferenceAge
            for _ in 0..11 {
                System::<Test>::set_block_number(System::<Test>::block_number() + 10);
                <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            }

            // Old inference should be cleaned up
            assert!(!PalletCbcPoi::inference_results(1).is_some());
        });
    }

    #[test]
    fn test_multiple_challenges_impact() {
        new_test_ext().execute_with(|| {
            // Submit an inference
            assert_ok!(PalletCbcPoi::submit_inference(
                RuntimeOrigin::signed(1),
                80,
                90
            ));

            // Multiple validators challenge the inference
            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(2),
                1,
                80
            ));
            assert_ok!(PalletCbcPoi::challenge_inference(
                RuntimeOrigin::signed(3),
                1,
                80
            ));

            // Calculate score - should get base score - penalty for each successful challenge
            let score = PalletCbcPoi::calculate_validator_score(&1).unwrap();
            assert_eq!(score, 40); // 80 (base) - 20 (first challenge) - 20 (second challenge)
        });
    }

    #[test]
    fn test_epoch_transition_event() {
        new_test_ext().execute_with(|| {
            // Initial epoch should be 0
            assert_eq!(PalletCbcPoi::current_epoch(), 0);

            // Advance block number to trigger epoch transition
            System::<Test>::set_block_number(10);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());

            // Epoch should be incremented
            assert_eq!(PalletCbcPoi::current_epoch(), 1);

            // Check if epoch transition event was emitted
            let events = System::<Test>::events();
            let epoch_event = events.iter().find(|e| {
                matches!(
                    e.event,
                    RuntimeEvent::PalletCbcPoi(crate::Event::EpochTransitioned { epoch_number: 1 })
                )
            });
            assert!(epoch_event.is_some(), "Epoch transition event was not emitted");
        });
    }

    #[test]
    fn test_epoch_transition_timing() {
        new_test_ext().execute_with(|| {
            // Initial epoch should be 0
            assert_eq!(PalletCbcPoi::current_epoch(), 0);

            // Advance to block 9 (should not trigger transition)
            System::<Test>::set_block_number(9);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            assert_eq!(PalletCbcPoi::current_epoch(), 0);

            // Advance to block 10 (should trigger transition)
            System::<Test>::set_block_number(10);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            assert_eq!(PalletCbcPoi::current_epoch(), 1);

            // Advance to block 19 (should not trigger transition)
            System::<Test>::set_block_number(19);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            assert_eq!(PalletCbcPoi::current_epoch(), 1);

            // Advance to block 20 (should trigger transition)
            System::<Test>::set_block_number(20);
            <PalletCbcPoi as Hooks<u64>>::on_initialize(System::<Test>::block_number());
            assert_eq!(PalletCbcPoi::current_epoch(), 2);
        });
    }
} 