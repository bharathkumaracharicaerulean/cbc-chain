#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::DispatchError;
    use crate::pallet::Pallet;
    use crate::pallet::CurrentEpoch;
    use crate::pallet::LastEpochUpdate;

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
            assert_noop!(
                PalletCbcPos::register_validator(RuntimeOrigin::root()),
                DispatchError::BadOrigin
            );

            // Try to register with none origin
            assert_noop!(
                PalletCbcPos::register_validator(RuntimeOrigin::none()),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_submit_score() {
        new_test_ext().execute_with(|| {
            // Try to submit score with root origin
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::root(), 1, 75),
                DispatchError::BadOrigin
            );

            // Try to submit score with none origin
            assert_noop!(
                PalletCbcPos::submit_score(RuntimeOrigin::none(), 1, 75),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_slash() {
        new_test_ext().execute_with(|| {
            // Try to slash with root origin
            assert_noop!(
                PalletCbcPos::slash_validator(RuntimeOrigin::root(), 1),
                DispatchError::BadOrigin
            );

            // Try to slash with none origin
            assert_noop!(
                PalletCbcPos::slash_validator(RuntimeOrigin::none(), 1),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_boost_score() {
        new_test_ext().execute_with(|| {
            // Try to boost score with root origin
            assert_noop!(
                PalletCbcPos::boost_score(RuntimeOrigin::root(), 1, b"block".to_vec(), 0),
                DispatchError::BadOrigin
            );

            // Try to boost score with none origin
            assert_noop!(
                PalletCbcPos::boost_score(RuntimeOrigin::none(), 1, b"block".to_vec(), 0),
                DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn test_unauthorized_update_epoch() {
        new_test_ext().execute_with(|| {
            // Try to update epoch with root origin
            assert_noop!(
                PalletCbcPos::update_epoch(RuntimeOrigin::root()),
                DispatchError::BadOrigin
            );

            // Try to update epoch with none origin
            assert_noop!(
                PalletCbcPos::update_epoch(RuntimeOrigin::none()),
                DispatchError::BadOrigin
            );
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
    fn test_score_boost() {
        new_test_ext().execute_with(|| {
            // Register a validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Initial score should be 0
            assert_eq!(PalletCbcPos::validator_scores(1), None);
            
            // Submit initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 100));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(100));
            
            // Boost score for block authorship
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"block".to_vec(), 0));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(150)); // 100 + 50 (BlockAuthorshipBoost)
            
            // Boost score for inference accuracy
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"inference".to_vec(), 0));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(180)); // 150 + 30 (InferenceAccuracyBoost)
            
            // Test invalid boost type
            assert_noop!(
                PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"invalid".to_vec(), 0),
                Error::<Test>::InvalidBoostAmount
            );
        });
    }

    #[test]
    fn test_score_decay() {
        new_test_ext().execute_with(|| {
            // Register a validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Submit initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 1000));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(1000));
            
            // Set initial epoch and last update
            CurrentEpoch::<Test>::put(1);
            LastEpochUpdate::<Test>::put(0);
            
            // Update epoch (10% decay)
            assert_ok!(PalletCbcPos::update_epoch(RuntimeOrigin::signed(1)));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(900)); // 1000 - 10%
            
            // Increment epoch
            CurrentEpoch::<Test>::put(2);
            
            // Update epoch again
            assert_ok!(PalletCbcPos::update_epoch(RuntimeOrigin::signed(1)));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(810)); // 900 - 10%
        });
    }

    #[test]
    fn test_score_history() {
        new_test_ext().execute_with(|| {
            // Register a validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Submit initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 100));
            
            // Set initial epoch and last update
            CurrentEpoch::<Test>::put(1);
            LastEpochUpdate::<Test>::put(0);
            
            // Boost score multiple times
            for i in 1..=5 {
                assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"block".to_vec(), 0));
                CurrentEpoch::<Test>::put(i);
                assert_ok!(PalletCbcPos::update_epoch(RuntimeOrigin::signed(1)));
            }
            
            // Check score history
            let history = PalletCbcPos::validator_score_history(1);
            assert_eq!(history.len(), 5);
            
            // Verify history contains correct epochs and scores
            for (i, (epoch, score)) in history.iter().enumerate() {
                assert_eq!(*epoch, (i + 1) as u32);
                // Score should decrease due to decay
                if i > 0 {
                    assert!(*score < history[i-1].1);
                }
            }
        });
    }

    #[test]
    fn test_validator_selection() {
        new_test_ext().execute_with(|| {
            // Register multiple validators
            for i in 1..=5 {
                assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(i)));
                assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(i), i, 100 * i as u32));
            }
            
            // Get top validators
            let top_validators = PalletCbcPos::get_top_validators(3);
            assert_eq!(top_validators.len(), 3);
            
            // Verify order (should be descending by score)
            assert_eq!(top_validators[0], 5);
            assert_eq!(top_validators[1], 4);
            assert_eq!(top_validators[2], 3);
            
            // Test tiebreaker
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 300));
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(2), 2, 300));
            
            let top_validators = PalletCbcPos::get_top_validators(2);
            assert_eq!(top_validators.len(), 2);
            
            // Verify the order matches our expectations (lower account ID first in case of tie)
            assert_eq!(top_validators[0], 1);
            assert_eq!(top_validators[1], 2);
        });
    }

    #[test]
    fn test_max_score_limit() {
        new_test_ext().execute_with(|| {
            // Register a validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Submit initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 950));
            
            // Try to boost beyond max score
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"block".to_vec(), 0));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(1000)); // Should be capped at MaxValidatorScore
            
            // Try another boost
            assert_ok!(PalletCbcPos::boost_score(RuntimeOrigin::signed(1), 1, b"inference".to_vec(), 0));
            assert_eq!(PalletCbcPos::validator_scores(1), Some(1000)); // Should still be capped
        });
    }

    #[test]
    fn test_invalid_epoch_update() {
        new_test_ext().execute_with(|| {
            // Register a validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));
            
            // Submit initial score
            assert_ok!(PalletCbcPos::submit_score(RuntimeOrigin::signed(1), 1, 100));
            
            // Set initial epoch and last update to same value
            CurrentEpoch::<Test>::put(1);
            LastEpochUpdate::<Test>::put(1);
            
            // Try to update epoch with same epoch number
            assert_noop!(
                PalletCbcPos::update_epoch(RuntimeOrigin::signed(1)),
                Error::<Test>::InvalidEpochUpdate
            );
        });
    }
} 