//! Comprehensive tests for finality marker correctness validation at scale.
//!
//! This module contains tests that verify the finality marker system works correctly
//! under various conditions, including:
//! - Monotonic advancement validation
//! - Finality regression detection
//! - Bounds checking against best known blocks
//! - Long-run simulation across multiple epochs
//! - Edge cases and error conditions

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, assert_err};

/// Test monotonic advancement validation for finality markers.
/// 
/// This test verifies that finality markers always advance monotonically
/// and that attempts to move finality backward are properly detected and rejected.
#[test]
fn test_finality_monotonic_advancement() {
    new_test_ext().execute_with(|| {
        // Initialize with genesis finality
        assert_eq!(DcfPallet::last_finalized_block(), 1);
        assert_eq!(DcfPallet::previous_finalized_block(), 0);
        
        // Test valid advancement
        assert_ok!(DcfPallet::update_finality_markers(5, 1));
        assert_eq!(DcfPallet::last_finalized_block(), 5);
        assert_eq!(DcfPallet::previous_finalized_block(), 1);
        
        // Test another valid advancement
        assert_ok!(DcfPallet::update_finality_markers(10, 2));
        assert_eq!(DcfPallet::last_finalized_block(), 10);
        assert_eq!(DcfPallet::previous_finalized_block(), 5);
        
        // Test regression attempt (should fail)
        assert_err!(
            DcfPallet::update_finality_markers(8, 3),
            BoundedVec::try_from("Finality regression: 8 < 10".as_bytes().to_vec()).unwrap()
        );
        
        // Verify finality didn't change after failed attempt
        assert_eq!(DcfPallet::last_finalized_block(), 10);
        assert_eq!(DcfPallet::previous_finalized_block(), 5);
        
        // Test same block (should succeed - no regression)
        assert_ok!(DcfPallet::update_finality_markers(10, 3));
        assert_eq!(DcfPallet::last_finalized_block(), 10);
        assert_eq!(DcfPallet::previous_finalized_block(), 10);
    });
}

/// Test finality regression detection with warning events.
/// 
/// This test verifies that finality regression attempts are detected
/// and appropriate warning events are emitted for monitoring systems.
#[test]
fn test_finality_regression_detection() {
    new_test_ext().execute_with(|| {
        // Set up initial finality state
        LastFinalizedBlock::<Test>::put(100);
        PreviousFinalizedBlock::<Test>::put(80);
        System::set_block_number(120);
        
        // Attempt finality regression
        let result = DcfPallet::update_finality_markers(70, 5);
        assert!(result.is_err());
        
        // Check that regression detection event was emitted
        let events = System::events();
        let regression_event = events.iter().find(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::FinalityRegressionDetected { 
                    previous_finalized: 100,
                    attempted_finalized: 70,
                    current_block: 120,
                    epoch: 5,
                })
            )
        });
        assert!(regression_event.is_some(), "FinalityRegressionDetected event should be emitted");
        
        // Verify finality state unchanged
        assert_eq!(DcfPallet::last_finalized_block(), 100);
        assert_eq!(DcfPallet::previous_finalized_block(), 80);
    });
}

/// Test validation that finality never exceeds best known block of prior epoch.
/// 
/// This test ensures that finality cannot advance beyond what was actually
/// produced and validated in the previous epoch.
#[test]
fn test_finality_bounds_validation() {
    new_test_ext().execute_with(|| {
        // Set up scenario where previous epoch best block is known
        LastFinalizedBlock::<Test>::put(50);
        PreviousFinalizedBlock::<Test>::put(30);
        PreviousEpochBestBlock::<Test>::put(80); // Previous epoch ended at block 80
        System::set_block_number(100);
        
        // Test valid advancement within bounds
        assert_ok!(DcfPallet::update_finality_markers(75, 2));
        assert_eq!(DcfPallet::last_finalized_block(), 75);
        
        // Test advancement that exceeds previous epoch best (should fail)
        assert_err!(
            DcfPallet::update_finality_markers(85, 2),
            BoundedVec::try_from("Finality exceeds previous epoch best: 85 > 80".as_bytes().to_vec()).unwrap()
        );
        
        // Check that advancement rejection event was emitted
        let events = System::events();
        let rejection_event = events.iter().find(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::FinalityAdvancementRejected { 
                    attempted_block: 85,
                    current_finalized: 75,
                    best_known_block: 80,
                    epoch: 2,
                    ..
                })
            )
        });
        assert!(rejection_event.is_some(), "FinalityAdvancementRejected event should be emitted");
        
        // Verify finality didn't change
        assert_eq!(DcfPallet::last_finalized_block(), 75);
    });
}

/// Test validation that finality doesn't exceed current block number.
/// 
/// This test ensures that finality cannot advance beyond the current
/// block number, preventing impossible finality states.
#[test]
fn test_finality_current_block_bounds() {
    new_test_ext().execute_with(|| {
        System::set_block_number(50);
        LastFinalizedBlock::<Test>::put(30);
        
        // Test valid advancement within current block bounds
        assert_ok!(DcfPallet::update_finality_markers(45, 1));
        assert_eq!(DcfPallet::last_finalized_block(), 45);
        
        // Test advancement beyond current block (should fail)
        assert_err!(
            DcfPallet::update_finality_markers(60, 1),
            BoundedVec::try_from("Finality exceeds current block: 60 > 50".as_bytes().to_vec()).unwrap()
        );
        
        // Test advancement to exactly current block (should succeed)
        assert_ok!(DcfPallet::update_finality_markers(50, 1));
        assert_eq!(DcfPallet::last_finalized_block(), 50);
    });
}

/// Test excessive finality advancement rate limiting.
/// 
/// This test verifies that finality cannot advance by unreasonably
/// large amounts in a single update, preventing potential attacks.
#[test]
fn test_finality_advancement_rate_limiting() {
    new_test_ext().execute_with(|| {
        let epoch_length = 2400u32; // From mock.rs EpochLength = ConstU32<2400>
        let max_advancement = epoch_length * 2; // 2 epochs worth
        
        System::set_block_number(1000);
        LastFinalizedBlock::<Test>::put(100);
        
        // Test reasonable advancement (should succeed)
        let reasonable_target = 100 + max_advancement - 10;
        assert_ok!(DcfPallet::update_finality_markers(reasonable_target, 1));
        assert_eq!(DcfPallet::last_finalized_block(), reasonable_target);
        
        // Test excessive advancement (should fail)
        let excessive_target = reasonable_target + max_advancement + 10;
        System::set_block_number((excessive_target + 100) as u64); // Ensure current block is high enough
        
        let result = DcfPallet::update_finality_markers(excessive_target, 2);
        assert!(result.is_err());
        
        // Verify the error message mentions excessive advancement
        if let Err(reason) = result {
            let reason_str = sp_std::str::from_utf8(&reason).unwrap_or("");
            assert!(reason_str.contains("Excessive finality advancement"));
        }
    });
}

/// Test temporal invariants checking with finality validation.
/// 
/// This test verifies that the temporal invariants checker properly
/// detects finality-related violations during epoch transitions.
#[test]
fn test_temporal_invariants_finality_checking() {
    new_test_ext().execute_with(|| {
        // Test 1: Normal case - no violations
        LastFinalizedBlock::<Test>::put(50);
        PreviousFinalizedBlock::<Test>::put(30);
        System::set_block_number(60);
        
        let violations = DcfPallet::check_temporal_invariants(1, 2);
        assert!(violations.is_empty(), "No violations should be detected in normal case");
        
        // Test 2: Finality regression
        LastFinalizedBlock::<Test>::put(20); // Regressed from 50 to 20
        PreviousFinalizedBlock::<Test>::put(50);
        
        let violations = DcfPallet::check_temporal_invariants(1, 2);
        assert!(!violations.is_empty(), "Should detect finality regression");
        
        let has_finality_violation = violations.iter().any(|v| {
            matches!(v, InvariantViolation::Temporal { 
                violation_type: TemporalViolationType::FinalityRegression { .. }, 
                .. 
            })
        });
        assert!(has_finality_violation, "Should detect finality regression violation");
        
        // Test 3: Finality exceeds current block
        LastFinalizedBlock::<Test>::put(100);
        System::set_block_number(80); // Current block is less than finalized
        
        let violations = DcfPallet::check_temporal_invariants(1, 2);
        assert!(!violations.is_empty(), "Should detect finality exceeding current block");
        
        // Test 4: Finality exceeds previous epoch best
        LastFinalizedBlock::<Test>::put(90);
        PreviousEpochBestBlock::<Test>::put(85);
        System::set_block_number(100);
        
        let violations = DcfPallet::check_temporal_invariants(1, 2);
        assert!(!violations.is_empty(), "Should detect finality exceeding previous epoch best");
    });
}

/// Test successful finality progression validation events.
/// 
/// This test verifies that successful finality updates emit appropriate
/// validation events for monitoring and audit purposes.
#[test]
fn test_finality_progression_validation_events() {
    new_test_ext().execute_with(|| {
        LastFinalizedBlock::<Test>::put(100);
        System::set_block_number(150);
        
        // Perform valid finality advancement
        assert_ok!(DcfPallet::update_finality_markers(120, 5));
        
        // Check that progression validation event was emitted
        let events = System::events();
        let validation_event = events.iter().find(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::FinalityProgressionValidated { 
                    previous_finalized: 100,
                    new_finalized: 120,
                    advancement: 20,
                    epoch: 5,
                    validation_checks_passed: 4,
                })
            )
        });
        assert!(validation_event.is_some(), "FinalityProgressionValidated event should be emitted");
        
        // Check that block finalized event was also emitted
        let finalized_event = events.iter().find(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::BlockFinalized { 
                    block_number: 120,
                })
            )
        });
        assert!(finalized_event.is_some(), "BlockFinalized event should be emitted");
    });
}

/// Test long-run simulation across multiple epochs for finality progression.
/// 
/// This test simulates finality progression across many epochs to verify
/// that the system maintains correctness over extended periods.
#[test]
fn test_long_run_finality_simulation() {
    new_test_ext().execute_with(|| {
        let epoch_length = 2400u32; // From mock.rs EpochLength = ConstU32<2400>
        let num_epochs = 50; // Simulate 50 epochs
        
        let mut current_block = 1u32;
        let mut current_epoch = 1u32;
        
        // Initialize starting state
        LastFinalizedBlock::<Test>::put(1);
        PreviousFinalizedBlock::<Test>::put(0);
        PreviousEpochBestBlock::<Test>::put(1);
        
        for epoch in 1..=num_epochs {
            // Simulate block production during the epoch
            let epoch_end_block = current_block + epoch_length - 1;
            System::set_block_number(epoch_end_block as u64);
            
            // Update finality at epoch boundary (finalize previous epoch's end)
            let finalize_block = if epoch == 1 {
                1 // Genesis case
            } else {
                current_block - 1 // Previous epoch's last block
            };
            
            // Validate finality advancement
            let result = DcfPallet::update_finality_markers(finalize_block, epoch);
            assert_ok!(result);
            
            // Verify finality progressed correctly
            assert_eq!(DcfPallet::last_finalized_block(), finalize_block);
            
            // Check temporal invariants
            let violations = DcfPallet::check_temporal_invariants(epoch.saturating_sub(1), epoch);
            assert!(violations.is_empty(), "No invariant violations should occur in epoch {}", epoch);
            
            // Advance to next epoch
            current_block = epoch_end_block + 1;
            current_epoch = epoch + 1;
            
            // Log progress every 10 epochs
            if epoch % 10 == 0 {
                println!("Completed epoch {}: finalized block {}, current block {}", 
                        epoch, DcfPallet::last_finalized_block(), current_block - 1);
            }
        }
        
        // Verify final state
        assert!(DcfPallet::last_finalized_block() > 1, "Finality should have advanced significantly");
        assert_eq!(DcfPallet::last_finalized_block(), current_block - epoch_length);
        
        println!("Long-run simulation completed successfully: {} epochs, final finalized block: {}", 
                num_epochs, DcfPallet::last_finalized_block());
    });
}

/// Test finality validation during epoch transitions.
/// 
/// This test verifies that finality validation is properly integrated
/// into the epoch transition process and handles edge cases correctly.
#[test]
fn test_finality_validation_in_epoch_transitions() {
    new_test_ext().execute_with(|| {
        let epoch_length = 2400u32; // From mock.rs EpochLength = ConstU32<2400>
        
        // Set up initial state
        System::set_block_number(epoch_length as u64);
        CurrentEpoch::<Test>::put(1);
        LastFinalizedBlock::<Test>::put(1);
        
        // Trigger epoch transition which should update finality
        let weight = DcfPallet::handle_epoch_transition();
        assert!(weight.ref_time() > 0, "Epoch transition should consume weight");
        
        // Verify finality was updated during transition
        let new_finalized = DcfPallet::last_finalized_block();
        assert!(new_finalized > 1, "Finality should advance during epoch transition");
        
        // Verify epoch advanced
        assert_eq!(DcfPallet::current_epoch(), 2);
        
        // Check that no invariant violations occurred
        let violations = DcfPallet::check_temporal_invariants(1, 2);
        assert!(violations.is_empty(), "No violations should occur during normal epoch transition");
    });
}

/// Test edge cases in finality validation.
/// 
/// This test covers various edge cases and boundary conditions
/// in the finality validation system.
#[test]
fn test_finality_validation_edge_cases() {
    new_test_ext().execute_with(|| {
        // Test 1: Zero advancement (same block)
        LastFinalizedBlock::<Test>::put(50);
        System::set_block_number(100);
        
        assert_ok!(DcfPallet::update_finality_markers(50, 1));
        assert_eq!(DcfPallet::last_finalized_block(), 50);
        
        // Test 2: Single block advancement
        assert_ok!(DcfPallet::update_finality_markers(51, 1));
        assert_eq!(DcfPallet::last_finalized_block(), 51);
        
        // Test 3: Maximum reasonable advancement
        let epoch_length = 2400u32; // From mock.rs EpochLength = ConstU32<2400>
        let max_advancement = epoch_length * 2;
        let target = 51 + max_advancement;
        System::set_block_number((target + 10) as u64);
        
        assert_ok!(DcfPallet::update_finality_markers(target, 1));
        assert_eq!(DcfPallet::last_finalized_block(), target);
        
        // Test 4: Previous epoch best block is zero (initial case)
        PreviousEpochBestBlock::<Test>::put(0);
        let new_target = target + 10;
        System::set_block_number((new_target + 10) as u64);
        
        // Should succeed when previous epoch best is 0 (no constraint)
        assert_ok!(DcfPallet::update_finality_markers(new_target, 1));
        assert_eq!(DcfPallet::last_finalized_block(), new_target);
    });
}

/// Test finality validation with multiple validators and complex scenarios.
/// 
/// This test verifies finality validation works correctly in complex
/// multi-validator scenarios with various network conditions.
#[test]
fn test_finality_validation_multi_validator_scenarios() {
    new_test_ext().execute_with(|| {
        // Set up multiple validators using the test account type
        let validators = vec![
            1u64, // Alice equivalent
            2u64, // Bob equivalent  
            3u64, // Charlie equivalent
        ];
        
        // Initialize validator set
        ValidatorSet::<Test>::put(
            BoundedVec::try_from(validators.clone()).unwrap()
        );
        ActiveValidators::<Test>::put(
            BoundedVec::try_from(validators.clone()).unwrap()
        );
        
        // Simulate finality progression with validator participation
        System::set_block_number(100);
        LastFinalizedBlock::<Test>::put(50);
        
        // Update finality with validator context
        assert_ok!(DcfPallet::update_finality_markers(80, 5));
        
        // Verify finality marker event includes validator information
        let events = System::events();
        let finality_marker_event = events.iter().find(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::FinalityMarker { 
                    block_number: 80,
                    total_validators: 3,
                    ..
                })
            )
        });
        assert!(finality_marker_event.is_some(), "FinalityMarker event should include validator info");
        
        // Test finality validation with validator set changes
        // Remove one validator
        let reduced_validators = vec![validators[0].clone(), validators[1].clone()];
        ActiveValidators::<Test>::put(
            BoundedVec::try_from(reduced_validators).unwrap()
        );
        
        // Finality should still work with changed validator set
        assert_ok!(DcfPallet::update_finality_markers(90, 6));
        assert_eq!(DcfPallet::last_finalized_block(), 90);
    });
}