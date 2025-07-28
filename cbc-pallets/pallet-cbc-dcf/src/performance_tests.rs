//! Performance tests for the DCF pallet

#![cfg(test)]

use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use std::time::Instant;

/// Helper function to create multiple test validators
fn create_test_validators(count: u32) -> Vec<u64> {
    let mut validators = Vec::new();
    for i in 0..count {
        let validator = (i + 100) as u64; // Start from 100 to avoid conflicts with genesis validators
        
        let validator_state = ValidatorState {
            last_active_epoch: 0,
            current: EpochStats {
                epoch: 0,
                stake_score: 1000 + (i * 10) as u64,
                inference_score: 800 + (i * 5) as u64,
                final_score: 900 + (i * 7) as u64,
                authored_blocks: 0,
                missed_blocks: 0,
            },
            history: BoundedVec::new(),
            uptime: 0,
            inference_success_count: 0,
            participation_rate: 100,
        };
        
        ValidatorStates::<Test>::insert(validator, validator_state);
        validators.push(validator);
    }
    
    validators
}

/// Test batch score updates performance
#[test]
fn test_batch_score_update_performance() {
    new_test_ext().execute_with(|| {
        let validator_count = 10;
        let validators = create_test_validators(validator_count);
        
        // Measure single updates
        let start_time = Instant::now();
        for validator in &validators {
            assert_ok!(DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(*validator),
                *validator
            ));
        }
        let single_update_time = start_time.elapsed();
        
        // Measure batch updates
        let start_time = Instant::now();
        let batch_result = DcfPallet::batch_update_validator_scores(
            &validators,
            true,  // update PoS
            false, // skip PoI for simplicity
        );
        let batch_update_time = start_time.elapsed();
        
        assert!(batch_result.is_ok());
        let successful_count = batch_result.unwrap();
        assert_eq!(successful_count, validator_count);
        
        println!("Single updates time: {:?}", single_update_time);
        println!("Batch updates time: {:?}", batch_update_time);
    });
}

/// Test validator ranking performance
#[test]
fn test_validator_ranking_performance() {
    new_test_ext().execute_with(|| {
        let validator_count = 20;
        let validators = create_test_validators(validator_count);
        
        // Add validators to active set
        let bounded_validators = BoundedVec::try_from(validators.clone())
            .expect("Too many validators for test");
        ActiveValidators::<Test>::put(bounded_validators);
        
        // Measure ranking performance
        let start_time = Instant::now();
        let rankings = DcfPallet::get_validator_rankings_cached();
        let ranking_time = start_time.elapsed();
        
        println!("Validator ranking time for {} validators: {:?}", validator_count, ranking_time);
        
        // Verify rankings are correct
        assert_eq!(rankings.len(), validator_count as usize);
        
        // Verify rankings are sorted (descending by score)
        for i in 1..rankings.len() {
            assert!(rankings[i-1].1 >= rankings[i].1);
        }
    });
}

/// Test optimized author selection performance
#[test]
fn test_optimized_author_selection_performance() {
    new_test_ext().execute_with(|| {
        let validator_count = 15;
        let validators = create_test_validators(validator_count);
        
        // Add validators to active set
        let bounded_validators = BoundedVec::try_from(validators.clone())
            .expect("Too many validators for test");
        ActiveValidators::<Test>::put(bounded_validators);
        
        // Measure author selection performance for multiple blocks
        let start_time = Instant::now();
        let mut selected_authors = Vec::new();
        
        for block_number in 1..=50 {
            if let Some(author) = DcfPallet::optimized_select_author(block_number) {
                selected_authors.push(author);
            }
        }
        
        let selection_time = start_time.elapsed();
        
        println!("Author selection time for 50 blocks: {:?}", selection_time);
        println!("Average time per selection: {:?}", selection_time / 50);
        
        // Verify authors were selected
        assert_eq!(selected_authors.len(), 50);
        
        // Verify all selected authors are valid validators
        for author in &selected_authors {
            assert!(validators.contains(author));
        }
    });
}

/// Test memory usage with validator sets
#[test]
fn test_memory_efficiency_validator_set() {
    new_test_ext().execute_with(|| {
        let validator_count = 30;
        let validators = create_test_validators(validator_count);
        
        // Test batch state queries
        let start_time = Instant::now();
        let states = DcfPallet::get_validator_states_batch(&validators);
        let query_time = start_time.elapsed();
        
        println!("Batch state query time for {} validators: {:?}", validator_count, query_time);
        
        // Verify all states were retrieved
        assert_eq!(states.len(), validator_count as usize);
        for state in &states {
            assert!(state.is_some());
        }
    });
}

/// Comprehensive performance benchmark
#[test]
fn test_comprehensive_performance_benchmark() {
    new_test_ext().execute_with(|| {
        println!("\\n=== DCF Performance Benchmark ===");
        
        let validator_count = 20;
        let validators = create_test_validators(validator_count);
        
        // Add validators to active set
        let bounded_validators = BoundedVec::try_from(validators.clone())
            .expect("Too many validators for test");
        ActiveValidators::<Test>::put(bounded_validators);
        
        // 1. Score Update Performance
        let start_time = Instant::now();
        for validator in &validators[0..5] {
            assert_ok!(DcfPallet::update_validator_stake_score(
                RuntimeOrigin::signed(*validator),
                *validator
            ));
        }
        let score_update_time = start_time.elapsed();
        println!("Score updates (5 validators): {:?}", score_update_time);
        
        // 2. Validator Queries Performance
        let start_time = Instant::now();
        for validator in &validators[0..10] {
            let _ = DcfPallet::validator_states(validator);
            let _ = DcfPallet::get_validator_profile(*validator);
        }
        let query_time = start_time.elapsed();
        println!("Validator queries (10 validators): {:?}", query_time);
        
        // 3. Runtime API Performance
        let start_time = Instant::now();
        let _ = DcfPallet::current_epoch();
        let _ = DcfPallet::active_validators();
        let _ = DcfPallet::pos_weight();
        let _ = DcfPallet::poi_weight();
        let api_time = start_time.elapsed();
        println!("Runtime API calls: {:?}", api_time);
        
        println!("=== Benchmark Complete ===");
    });
}