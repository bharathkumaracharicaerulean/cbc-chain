#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::traits::BadOrigin;

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
            assert_noop!(
                PalletCbcPoi::submit_inference(RuntimeOrigin::root(), 42, 80),
                BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_challenge() {
        new_test_ext().execute_with(|| {
            // Try to challenge with root origin
            assert_noop!(
                PalletCbcPoi::challenge_inference(RuntimeOrigin::root(), 1, 42),
                BadOrigin
            );
        });
    }
} 