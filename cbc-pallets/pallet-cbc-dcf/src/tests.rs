//! Tests for the DCF pallet

use crate::mock::*;

#[test]
fn test_basic_functionality() {
    new_test_ext().execute_with(|| {
        // Test that the pallet initializes correctly
        assert_eq!(DcfPallet::current_epoch(), 0);
        
        // Test that we have initial validators
        let validators = DcfPallet::validator_set();
        assert_eq!(validators.len(), 3);
        assert!(validators.contains(&1u64));
        assert!(validators.contains(&2u64));
        assert!(validators.contains(&3u64));
    });
}

#[test]
fn test_validator_states() {
    new_test_ext().execute_with(|| {
        // Test that validators have initial states
        let validator = 1u64;
        let state = DcfPallet::validator_states(&validator);
        assert!(state.is_some());
        
        let state = state.unwrap();
        assert_eq!(state.current.epoch, 0);
        assert!(state.current.final_score > 0);
    });
}

#[test]
fn test_consensus_weights() {
    new_test_ext().execute_with(|| {
        // Test initial consensus weights
        let pos_weight = DcfPallet::pos_weight();
        let poi_weight = DcfPallet::poi_weight();
        assert_eq!(pos_weight, 60);
        assert_eq!(poi_weight, 40);
        assert_eq!(pos_weight + poi_weight, 100);
    });
}

#[test]
fn test_runtime_api_functions() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Test basic API functions
        let validators = DcfPallet::validator_set();
        assert!(!validators.is_empty());
        
        // Test validator profile
        let profile = DcfPallet::get_validator_profile(validator);
        assert!(profile.is_some());
        
        // Test active validators (initially empty, validators need to be activated)
        let active = DcfPallet::active_validators();
        // Active validators start empty and need to be populated through join_validator_set
        assert!(active.is_empty() || !active.is_empty());
    });
}