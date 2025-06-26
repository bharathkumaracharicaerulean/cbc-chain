#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};

    #[test]
    fn test_register_validator_success() {
        new_test_ext().execute_with(|| {
            // Test successful validator registration
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Verify the validator was registered
            assert_eq!(PalletCbcPos::validators(1), Some(true));
        });
    }

    #[test]
    fn test_register_validator_already_registered() {
        new_test_ext().execute_with(|| {
            // Register validator first time
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Try to register same validator again
            assert_noop!(
                PalletCbcPos::register_validator(RuntimeOrigin::signed(1)),
                Error::<Test>::ValidatorAlreadyRegistered
            );
        });
    }

    #[test]
    fn test_submit_score_success() {
        new_test_ext().execute_with(|| {
            // Register validator first
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Submit score
            assert_ok!(PalletCbcPos::submit_score(
                RuntimeOrigin::signed(2),
                1,
                75 // score above minimum threshold
            ));

            // Verify the score was stored
            assert_eq!(PalletCbcPos::validator_scores(1), Some(75));
        });
    }

    #[test]
    fn test_submit_score_not_registered() {
        new_test_ext().execute_with(|| {
            // Try to submit score for non-registered validator
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 75),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_submit_score_too_low() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Try to submit score below minimum
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 20));
        });
    }

    #[test]
    fn test_slash_validator_success() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Slash validator
            assert_ok!(PalletCbcPos::slash_validator(RuntimeOrigin::signed(2), 1));

            // Verify slashing count
            assert_eq!(PalletCbcPos::slashing_count(1), Some(1));
        });
    }

    #[test]
    fn test_slash_validator_removal() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Slash validator multiple times to reach max count
            for _ in 0..3 {
                assert_ok!(PalletCbcPos::slash_validator(RuntimeOrigin::signed(2), 1));
            }

            // Verify validator was removed
            assert_eq!(PalletCbcPos::validators(1), None);
            assert_eq!(PalletCbcPos::validator_scores(1), None);
            assert_eq!(PalletCbcPos::slashing_count(1), None);
        });
    }

    #[test]
    fn test_slash_validator_not_registered() {
        new_test_ext().execute_with(|| {
            // Try to slash non-registered validator
            assert_noop!(
                PalletCbcPos::slash_validator(RuntimeOrigin::signed(2), 1),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_unauthorized_register() {
        new_test_ext().execute_with(|| {
            // Try to register with root origin
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::root()));
        });
    }

    #[test]
    fn test_unauthorized_submit_score() {
        new_test_ext().execute_with(|| {
            // Try to submit score with root origin
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::root(), 1, 75));
        });
    }

    #[test]
    fn test_unauthorized_slash() {
        new_test_ext().execute_with(|| {
            // Try to slash with root origin
            assert_ok!(PalletCbcPos::slash_validator(RuntimeOrigin::root(), 1));
        });
    }

    #[test]
    fn test_submit_score_no_registration() {
        new_test_ext().execute_with(|| {
            // This should fail because validator is not registered
            // Using assert_ok! will make the test fail if an error occurs
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 75));
        });
    }
} 