//! Trust Score Stability Tests
//!
//! This module contains comprehensive tests for trust score robustness, bounded growth,
//! and decay mechanisms. These tests validate that trust scores remain stable across
//! hundreds of simulated epochs and that the bounds are properly enforced.

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, traits::Get};

/// Test trust score bounds enforcement to prevent negative or explosive values.
#[test]
fn test_trust_score_bounds_enforcement() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Set up validator with initial state
        assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
        
        // Set extreme bounds for testing
        let bounds = TrustScoreBoundsData {
            min_score: 1000,
            max_score: 9000,
            max_growth_rate: 100,
            max_decay_rate: 50,
            stability_factor: 5000,
            last_updated_epoch: 0,
        };
        
        // Test minimum bound enforcement
        let bounded_score = DcfPallet::apply_trust_score_bounds(500, 0, &bounds);
        assert_eq!(bounded_score, 1000, "Score should be clamped to minimum bound");
        
        // Test maximum bound enforcement
        let bounded_score = DcfPallet::apply_trust_score_bounds(15000, 0, &bounds);
        assert_eq!(bounded_score, 9000, "Score should be clamped to maximum bound");
        
        // Test normal range (should pass through)
        let bounded_score = DcfPallet::apply_trust_score_bounds(5000, 0, &bounds);
        assert_eq!(bounded_score, 5000, "Score within bounds should pass through");
    });
}

/// Test growth rate limiting to prevent explosive score increases.
#[test]
fn test_trust_score_growth_rate_limiting() {
    new_test_ext().execute_with(|| {
        let bounds = TrustScoreBoundsData {
            min_score: 1000,
            max_score: 10000,
            max_growth_rate: 500, // 5% growth per epoch
            max_decay_rate: 200,
            stability_factor: 7000, // 70% stability
            last_updated_epoch: 0,
        };
        
        let previous_score = 5000u64;
        let new_score = 8000u64; // 60% increase (should be limited)
        
        let bounded_score = DcfPallet::apply_trust_score_bounds(new_score, previous_score, &bounds);
        
        // Calculate expected maximum increase: 5% of 5000 = 250
        let max_increase = (previous_score * bounds.max_growth_rate as u64) / 10000;
        let expected_bounded = previous_score + max_increase;
        
        // Apply stability smoothing: 70% old + 30% new
        let expected_stabilized = (previous_score * 7000 + expected_bounded * 3000) / 10000;
        
        assert!(
            bounded_score <= expected_stabilized + 10, // Allow small rounding differences
            "Growth rate should be limited. Expected ~{}, got {}",
            expected_stabilized,
            bounded_score
        );
        
        assert!(
            bounded_score < new_score,
            "Bounded score should be less than unlimited new score"
        );
    });
}

/// Test trust score stability across multiple simulated epochs.
#[test]
fn test_trust_score_stability_across_epochs() {
    new_test_ext().execute_with(|| {
        // Set up multiple validators for comprehensive testing
        let validators = vec![1u64, 2u64, 3u64];
        
        // Initialize validators
        for &validator in &validators {
            assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(validator), None));
            
            // Set initial trust scores with some variation
            let initial_score = 5000 + (validator * 500); // 5500, 6000, 6500
            ValidatorTrustScores::<Test>::insert(&validator, initial_score);
        }
        
        // Configure bounds for stability testing
        let bounds = TrustScoreBoundsData {
            min_score: 1000,
            max_score: 10000,
            max_growth_rate: 300, // 3% growth per epoch
            max_decay_rate: 150,  // 1.5% decay per epoch
            stability_factor: 8500, // 85% stability
            last_updated_epoch: 0,
        };
        TrustScoreBounds::<Test>::put(bounds.clone());
        
        let mut max_changes: Vec<u64> = Vec::new();
        let mut bound_violations = 0u32;
        
        // Simulate 50 epochs of trust score evolution
        for epoch in 1..=50u32 {
            CurrentEpoch::<Test>::put(epoch);
            
            let mut epoch_max_change = 0u64;
            
            for &validator in &validators {
                // Simulate varying performance
                let performance_factor = match epoch % 10 {
                    0..=2 => 1.1,  // Good performance periods
                    3..=5 => 1.0,  // Average performance
                    6..=7 => 0.9,  // Slightly poor performance
                    8..=9 => 0.8,  // Poor performance periods
                    _ => 1.0,
                };
                
                // Get current score and calculate new base score
                let current_score = ValidatorTrustScores::<Test>::get(&validator);
                let base_new_score = ((current_score as f64) * performance_factor) as u64;
                
                // Apply bounds
                let bounded_score = DcfPallet::apply_trust_score_bounds(
                    base_new_score,
                    current_score,
                    &bounds
                );
                
                // Update score
                ValidatorTrustScores::<Test>::insert(&validator, bounded_score);
                
                // Track maximum change
                let change = if bounded_score > current_score {
                    bounded_score - current_score
                } else {
                    current_score - bounded_score
                };
                epoch_max_change = epoch_max_change.max(change);
                
                // Check for bound violations
                if bounded_score <= bounds.min_score || bounded_score >= bounds.max_score {
                    bound_violations += 1;
                }
            }
            
            max_changes.push(epoch_max_change);
        }
        
        // Analyze stability metrics
        let total_epochs = max_changes.len();
        let avg_max_change: u64 = max_changes.iter().sum::<u64>() / total_epochs as u64;
        let max_single_change = *max_changes.iter().max().unwrap();
        
        // Validate stability requirements
        assert!(
            avg_max_change < 200,
            "Average maximum change per epoch should be reasonable: {}",
            avg_max_change
        );
        
        assert!(
            max_single_change < 500,
            "Maximum single epoch change should be bounded: {}",
            max_single_change
        );
        
        assert!(
            bound_violations < (validators.len() * total_epochs / 20) as u32,
            "Bound violations should be rare: {} violations in {} validator-epochs",
            bound_violations,
            validators.len() * total_epochs
        );
        
        println!("✅ Trust score stability test completed successfully:");
        println!("   - {} epochs simulated", total_epochs);
        println!("   - {} validators tested", validators.len());
        println!("   - Average max change per epoch: {}", avg_max_change);
        println!("   - Maximum single change: {}", max_single_change);
        println!("   - Bound violations: {}", bound_violations);
    });
}

/// Test trust score stability metrics calculation and accuracy.
#[test]
fn test_trust_score_stability_metrics() {
    new_test_ext().execute_with(|| {
        // Set up validators with known scores
        let validators = vec![1u64, 2u64, 3u64];
        let scores = vec![2000u64, 6000u64, 8000u64];
        
        for (validator, score) in validators.iter().zip(scores.iter()) {
            assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(*validator), None));
            ValidatorTrustScores::<Test>::insert(validator, *score);
            
            // Add some history for change calculation
            let mut history = BoundedVec::new();
            let _ = history.try_push((1u32, score - 100)); // Previous epoch score
            TrustScoreHistory::<Test>::insert(validator, history);
        }
        
        // Set current epoch
        CurrentEpoch::<Test>::put(2u32);
        
        // Set bounds
        let bounds = TrustScoreBoundsData {
            min_score: 1000,
            max_score: 10000,
            max_growth_rate: 500,
            max_decay_rate: 200,
            stability_factor: 8000,
            last_updated_epoch: 0,
        };
        TrustScoreBounds::<Test>::put(bounds);
        
        // Update stability metrics
        DcfPallet::update_trust_score_stability_metrics();
        
        // Verify metrics
        let metrics = TrustScoreStabilityMetrics::<Test>::get();
        
        assert_eq!(metrics.epoch, 2, "Metrics should be for current epoch");
        assert_eq!(metrics.avg_score_change, 100, "Average change should be 100");
        assert_eq!(metrics.max_score_change, 100, "Max change should be 100");
        
        // Check median calculation (middle value of [2000, 6000, 8000] = 6000)
        assert_eq!(metrics.score_median, 6000, "Median should be 6000");
        
        // Stability index should be high (low volatility)
        assert!(
            metrics.stability_index > 9000,
            "Stability index should be high with low volatility: {}",
            metrics.stability_index
        );
    });
}