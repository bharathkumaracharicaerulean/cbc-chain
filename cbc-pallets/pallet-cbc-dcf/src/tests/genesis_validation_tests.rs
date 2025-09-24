//! Tests for genesis validation and dry-run capabilities.
//!
//! This module contains comprehensive tests for the genesis validation functionality
//! implemented as part of task 11: "Implement genesis validation and dry-run capabilities".
//!
//! The tests cover:
//! - Genesis configuration validation for duplicate validators and invalid stakes
//! - Active set size validation not exceeding MaxValidators
//! - Dry-run function that builds genesis and asserts all invariants
//! - Comprehensive error handling and edge cases

use super::*;
use crate::mock::*;
use frame_support::{
    assert_err, assert_ok,
    traits::{Currency, ReservableCurrency},
};
use sp_runtime::traits::Zero;

/// Test genesis validation with valid configuration.
/// 
/// This test verifies that a properly configured genesis passes all validation checks.
/// 
/// Requirements Coverage:
/// - 11.1: Validates that valid configurations pass duplicate and stake checks
/// - 11.2: Validates that active set size within limits passes validation
#[test]
fn test_genesis_validation_valid_config() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert_ok!(result);

        // Test dispatchable validation
        assert_ok!(DcfPallet::validate_genesis_configuration(
            RuntimeOrigin::root(),
            validators,
            stakes,
            epoch_config,
        ));
    });
}

/// Test genesis validation with duplicate validators.
/// 
/// This test verifies that duplicate validators are properly detected and rejected.
/// 
/// Requirements Coverage:
/// - 11.1: Validates detection of duplicate validators
#[test]
fn test_genesis_validation_duplicate_validators() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 1u64]; // Duplicate validator 1
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate validator"));

        // Test dispatchable validation
        assert_err!(
            DcfPallet::validate_genesis_configuration(
                RuntimeOrigin::root(),
                validators,
                stakes,
                epoch_config,
            ),
            Error::<Test>::GenesisValidationFailed
        );
    });
}

/// Test genesis validation with invalid stakes.
/// 
/// This test verifies that stakes below minimum requirements are properly detected.
/// 
/// Requirements Coverage:
/// - 11.1: Validates detection of invalid stakes
#[test]
fn test_genesis_validation_invalid_stakes() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 500_000u128, 3_000_000u128]; // Second stake below minimum
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("below MinStake"));

        // Test dispatchable validation
        assert_err!(
            DcfPallet::validate_genesis_configuration(
                RuntimeOrigin::root(),
                validators,
                stakes,
                epoch_config,
            ),
            Error::<Test>::GenesisValidationFailed
        );
    });
}

/// Test genesis validation with too many validators.
/// 
/// This test verifies that active set size limits are properly enforced.
/// 
/// Requirements Coverage:
/// - 11.2: Validates active set size not exceeding MaxValidators
#[test]
fn test_genesis_validation_too_many_validators() {
    new_test_ext().execute_with(|| {
        // Create more validators than MaxValidators (which is 100 in mock)
        let validators: Vec<u64> = (1..=101).collect(); // 101 validators
        let stakes = vec![1_000_000u128; 101];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds MaxValidators"));

        // Test dispatchable validation
        assert_err!(
            DcfPallet::validate_genesis_configuration(
                RuntimeOrigin::root(),
                validators,
                stakes,
                epoch_config,
            ),
            Error::<Test>::GenesisValidationFailed
        );
    });
}

/// Test genesis validation with mismatched array lengths.
/// 
/// This test verifies that array length mismatches are properly detected.
#[test]
fn test_genesis_validation_mismatched_arrays() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 2_000_000u128]; // One less stake than validators
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match validators length"));
    });
}

/// Test genesis dry-run with valid configuration.
/// 
/// This test verifies that the dry-run function works correctly with valid configurations.
/// 
/// Requirements Coverage:
/// - 11.3: Tests dry-run function that builds genesis and asserts all invariants
/// - 11.4: Validates comprehensive validation without side effects
#[test]
fn test_genesis_dry_run_valid_config() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test dry-run function
        let result = DcfPallet::dry_run_genesis_build(&genesis_config);
        assert_ok!(&result);

        let report = result.unwrap();
        assert_eq!(report.validator_count, 3);
        assert_eq!(report.total_stake, 6_000_000u128);
        assert_eq!(report.average_stake, 2_000_000u128);
        assert!(report.validation_passed);
        assert!(!report.invariant_checks.is_empty());

        // Verify min/max stake validators
        assert!(report.min_stake_validator.is_some());
        assert!(report.max_stake_validator.is_some());
        let (min_validator, min_stake) = report.min_stake_validator.unwrap();
        let (max_validator, max_stake) = report.max_stake_validator.unwrap();
        assert_eq!(min_validator, 1u64);
        assert_eq!(min_stake, 1_000_000u128);
        assert_eq!(max_validator, 3u64);
        assert_eq!(max_stake, 3_000_000u128);

        // Test dispatchable dry-run
        assert_ok!(DcfPallet::dry_run_genesis_configuration(
            RuntimeOrigin::root(),
            validators,
            stakes,
            epoch_config,
        ));
    });
}

/// Test genesis dry-run with configuration that generates warnings.
/// 
/// This test verifies that the dry-run function properly generates warnings for
/// potentially problematic configurations.
#[test]
fn test_genesis_dry_run_with_warnings() {
    new_test_ext().execute_with(|| {
        // Small validator set (should generate warning)
        let validators = vec![1u64, 2u64];
        // High stake inequality (should generate warning)
        let stakes = vec![1_000_000u128, 20_000_000u128]; // 20x difference
        let epoch_config = EpochConfig {
            blocks_per_epoch: 50, // Short epoch (should generate warning)
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test dry-run function
        let result = DcfPallet::dry_run_genesis_build(&genesis_config);
        assert_ok!(&result);

        let report = result.unwrap();
        assert!(report.validation_passed);
        assert!(!report.warnings.is_empty());

        // Should have warnings for small validator set, stake inequality, and short epoch
        assert!(report.warnings.len() >= 2);
        
        // Check for specific warnings
        let warnings_text = report.warnings.join(" ");
        assert!(warnings_text.contains("Small validator set") || warnings_text.contains("Short epoch"));
    });
}

/// Test genesis dry-run with invalid configuration.
/// 
/// This test verifies that the dry-run function properly fails with invalid configurations.
#[test]
fn test_genesis_dry_run_invalid_config() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 1u64]; // Duplicate validator
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test dry-run function
        let result = DcfPallet::dry_run_genesis_build(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate validator"));

        // Test dispatchable dry-run
        assert_err!(
            DcfPallet::dry_run_genesis_configuration(
                RuntimeOrigin::root(),
                validators,
                stakes,
                epoch_config,
            ),
            Error::<Test>::GenesisValidationFailed
        );
    });
}

/// Test current system invariant validation.
/// 
/// This test verifies that the current system state can be validated against invariants.
#[test]
fn test_validate_current_invariants() {
    new_test_ext().execute_with(|| {
        // Initialize some test state
        let validator = 1u64;
        let stake = 1_000_000u128;
        
        // Add validator to active set
        let mut active_validators = BoundedVec::new();
        active_validators.try_push(validator).unwrap();
        ActiveValidators::<Test>::put(active_validators);
        
        // Set validator stake
        ValidatorStake::<Test>::insert(&validator, stake);
        
        // Set trust score within bounds
        ValidatorTrustScores::<Test>::insert(&validator, 5000u64);
        
        // Set consensus weights
        PosWeight::<Test>::put(5000u64);
        PoiWeight::<Test>::put(5000u64);
        
        // Set finality markers
        LastFinalizedBlock::<Test>::put(10u32);
        PreviousFinalizedBlock::<Test>::put(5u32);

        // Test invariant validation
        let result = DcfPallet::validate_current_invariants();
        assert_ok!(result);
    });
}

/// Test current system invariant validation with violations.
/// 
/// This test verifies that invariant violations are properly detected.
#[test]
fn test_validate_current_invariants_with_violations() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Add validator to active set
        let mut active_validators = BoundedVec::new();
        active_validators.try_push(validator).unwrap();
        ActiveValidators::<Test>::put(active_validators);
        
        // Set stake below minimum (violation)
        ValidatorStake::<Test>::insert(&validator, 500_000u128); // Below MinStake
        
        // Set trust score outside bounds (violation)
        ValidatorTrustScores::<Test>::insert(&validator, 15000u64); // Above MaxTrustScore
        
        // Set invalid consensus weights (violation)
        PosWeight::<Test>::put(0u64);
        PoiWeight::<Test>::put(0u64);
        
        // Set invalid finality markers (violation)
        LastFinalizedBlock::<Test>::put(5u32);
        PreviousFinalizedBlock::<Test>::put(10u32); // Previous > current

        // Test invariant validation
        let result = DcfPallet::validate_current_invariants();
        assert!(result.is_err());
        
        let violations = result.unwrap_err();
        assert!(!violations.is_empty());
        
        // Should have multiple violations
        assert!(violations.len() >= 3);
        
        // Check for specific violations
        let violations_text = violations.join(" ");
        assert!(violations_text.contains("stake") && violations_text.contains("below MinStake"));
        assert!(violations_text.contains("trust score") && violations_text.contains("outside bounds"));
        assert!(violations_text.contains("PoS") && violations_text.contains("PoI"));
    });
}

/// Test genesis validation with empty validator set.
/// 
/// This test verifies that empty validator sets are properly rejected.
#[test]
fn test_genesis_validation_empty_validators() {
    new_test_ext().execute_with(|| {
        let validators = vec![]; // Empty validator set
        let stakes = vec![];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least one validator"));
    });
}

/// Test genesis validation with invalid epoch configuration.
/// 
/// This test verifies that invalid epoch configurations are properly rejected.
#[test]
fn test_genesis_validation_invalid_epoch_config() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 0, // Invalid: zero epoch length
            min_stake: 1_000_000,
            max_validators: 100,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: vec![],
            validator_stakes: stakes.clone(),
            current_epoch: 0,
            epoch_config: epoch_config.clone(),
            validator_names: vec![],
            strict_validation: true,
        };

        // Test comprehensive validation
        let result = DcfPallet::validate_genesis_config(&genesis_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Epoch length cannot be zero"));
    });
}

/// Test authorization for genesis validation dispatchables.
/// 
/// This test verifies that only root can call genesis validation functions.
#[test]
fn test_genesis_validation_authorization() {
    new_test_ext().execute_with(|| {
        let validators = vec![1u64, 2u64, 3u64];
        let stakes = vec![1_000_000u128, 2_000_000u128, 3_000_000u128];
        let epoch_config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1_000_000,
            max_validators: 100,
        };

        // Test that non-root origin is rejected
        assert_err!(
            DcfPallet::validate_genesis_configuration(
                RuntimeOrigin::signed(1),
                validators.clone(),
                stakes.clone(),
                epoch_config.clone(),
            ),
            sp_runtime::DispatchError::BadOrigin
        );

        assert_err!(
            DcfPallet::dry_run_genesis_configuration(
                RuntimeOrigin::signed(1),
                validators,
                stakes,
                epoch_config,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}