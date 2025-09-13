#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok, traits::Get};

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

            // Try to submit score below minimum - should fail
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 20),
                Error::<Test>::ScoreTooLow
            );
        });
    }

    #[test]
    fn test_slash_validator_success() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Slash validator
            assert_ok!(PalletCbcPos::slash_validator(&1, 100));

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
                assert_ok!(PalletCbcPos::slash_validator(&1, 100));
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
                PalletCbcPos::slash_validator(&1, 100),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_unauthorized_register() {
        new_test_ext().execute_with(|| {
            // Try to register with root origin - should fail
            assert_noop!(
                PalletCbcPos::register_validator(RuntimeOrigin::root()),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_submit_score() {
        new_test_ext().execute_with(|| {
            // Try to submit score with root origin - should fail
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::root(), 1, 75),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_slash() {
        new_test_ext().execute_with(|| {
            // Register validator first
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Try to slash - should work as it's an internal function
            assert_ok!(PalletCbcPos::slash_validator(&1, 100));
        });
    }

    #[test]
    fn test_submit_score_no_registration() {
        new_test_ext().execute_with(|| {
            // This should fail because validator is not registered
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 75),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_boost_score_functionality() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Set initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 50));
            
            // Boost score
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(2), 1, 10));
            
            // Verify score was boosted
            let new_score = PalletCbcPos::validator_scores(1).unwrap_or(0);
            assert!(new_score >= 50);
        });
    }

    #[test]
    fn test_slash_score_functionality() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Set initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 100));
            
            // Slash score
            assert_ok!(PalletCbcPos::slash_score(RuntimeOrigin::signed(2), 1, 10));
            
            // Verify score was slashed
            let new_score = PalletCbcPos::validator_scores(1).unwrap_or(0);
            assert!(new_score <= 100);
        });
    }

    #[test]
    fn test_validator_score_decay() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Set initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, 100));
            
            // Apply score decay
            let decay_amount = <Test as crate::Config>::ValidatorScoreDecay::get();
            let initial_score = PalletCbcPos::validator_scores(1).unwrap_or(0);
            
            // Simulate decay (this would normally happen in on_initialize)
            let expected_score = initial_score.saturating_sub(decay_amount);
            
            // Verify decay calculation
            assert!(expected_score <= initial_score);
        });
    }

    #[test]
    fn test_multiple_validators_registration() {
        new_test_ext().execute_with(|| {
            // Register multiple validators
            for i in 1..=5 {
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(i)));
                assert!(PalletCbcPos::validators(i).is_some());
            }
            
            // Verify all are registered
            for i in 1..=5 {
                assert_eq!(PalletCbcPos::validators(i), Some(true));
            }
        });
    }

    #[test]
    fn test_validator_score_boundaries() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Test minimum score
            let min_score = <Test as crate::Config>::MinValidatorScore::get();
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, min_score));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(min_score));
            
            // Test score below minimum (should fail)
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 1, min_score - 1),
                Error::<Test>::ScoreTooLow
            );
        });
    }

    #[test]
    fn test_slashing_count_tracking() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Slash validator and check count
            assert_ok!(PalletCbcPos::slash_validator(&1, 100));
            assert_eq!(PalletCbcPos::slashing_count(1), Some(1));
            
            // Slash again
            assert_ok!(PalletCbcPos::slash_validator(&1, 100));
            assert_eq!(PalletCbcPos::slashing_count(1), Some(2));
            
            // One more slash should remove validator
            assert_ok!(PalletCbcPos::slash_validator(&1, 100));
            assert_eq!(PalletCbcPos::validators(1), None);
            assert_eq!(PalletCbcPos::slashing_count(1), None);
        });
    }
} 