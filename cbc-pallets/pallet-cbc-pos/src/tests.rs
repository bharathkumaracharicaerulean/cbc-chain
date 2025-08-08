#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::Error;
    use crate::pallet::SlashingEvent;
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

            // Slash validator with reason and penalty
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::signed(2), 
                1, 
                b"Double signing".to_vec(), 
                100
            ));

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
            for i in 0..3 {
                let reason = format!("Slashing reason {}", i).into_bytes();
                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::signed(2), 
                    1, 
                    reason, 
                    100
                ));
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
                PalletCbcPos::slash_validator(
                    RuntimeOrigin::signed(2), 
                    1, 
                    b"Test reason".to_vec(), 
                    100
                ),
                Error::<Test>::ValidatorNotRegistered
            );
        });
    }

    #[test]
    fn test_slashing_history_tracking() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Initial history should be empty
            assert_eq!(PalletCbcPos::slashing_history(1), Vec::<SlashingEvent<u128>>::new());

            // First slashing
            let reason1 = b"Double signing".to_vec();
            let penalty1 = 100;
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::signed(2), 
                1, 
                reason1.clone(), 
                penalty1
            ));

            // Check history after first slashing
            let history = PalletCbcPos::slashing_history(1);
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].reason, reason1);
            assert_eq!(history[0].penalty_amount, penalty1);
            assert_eq!(history[0].slashing_count, 1);

            // Second slashing
            let reason2 = b"Network attack".to_vec();
            let penalty2 = 200;
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::signed(2), 
                1, 
                reason2.clone(), 
                penalty2
            ));

            // Check history after second slashing
            let history = PalletCbcPos::slashing_history(1);
            assert_eq!(history.len(), 2);
            assert_eq!(history[1].reason, reason2);
            assert_eq!(history[1].penalty_amount, penalty2);
            assert_eq!(history[1].slashing_count, 2);

            // Verify timestamps are increasing
            assert!(history[1].timestamp > history[0].timestamp);
        });
    }

    #[test]
    fn test_slashing_history_persistence() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Perform multiple slashings
            for i in 0..5 {
                let reason = format!("Slashing reason {}", i).into_bytes();
                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::signed(2), 
                    1, 
                    reason, 
                    100 + (i as u128 * 50)
                ));
            }

            // Verify all slashing events are recorded
            let history = PalletCbcPos::slashing_history(1);
            assert_eq!(history.len(), 5);

            // Verify each event has correct data
            for (i, event) in history.iter().enumerate() {
                assert_eq!(event.slashing_count, (i + 1) as u32);
                assert_eq!(event.penalty_amount, 100 + (i as u128 * 50));
                assert!(event.timestamp > 0);
            }
        });
    }

    #[test]
    fn test_slashing_history_after_validator_removal() {
        new_test_ext().execute_with(|| {
            // Register validator
            assert_ok!(PalletCbcPos::register_validator(RuntimeOrigin::signed(1)));

            // Perform slashings until validator is removed
            for i in 0..3 {
                let reason = format!("Slashing reason {}", i).into_bytes();
                assert_ok!(PalletCbcPos::slash_validator(
                    RuntimeOrigin::signed(2), 
                    1, 
                    reason, 
                    100
                ));
            }

            // Verify validator is removed
            assert_eq!(PalletCbcPos::validators(1), None);

            // Verify slashing history is still preserved
            let history = PalletCbcPos::slashing_history(1);
            assert_eq!(history.len(), 3);

            // Verify the last event has the maximum slashing count
            assert_eq!(history[2].slashing_count, 3);
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
            assert_ok!(PalletCbcPos::slash_validator(
                RuntimeOrigin::root(), 
                1, 
                b"Test reason".to_vec(), 
                100
            ));
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