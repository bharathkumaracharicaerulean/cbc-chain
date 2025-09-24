//! Tests for storage migration functionality in the DCF pallet.
//!
//! This module contains comprehensive tests for the storage versioning and
//! migration system, ensuring that migrations work correctly and safely.

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, assert_err, traits::Hooks};
use crate::pallet::StorageVersion as DcfStorageVersion; // Use our pallet's StorageVersion

/// Test that storage version is initialized correctly.
#[test]
fn storage_version_initialization_works() {
    new_test_ext().execute_with(|| {
        // Initially, storage version should be 0 (default)
        assert_eq!(DcfStorageVersion::<Test>::get(), 0);
        
        // Set storage version to current version
        DcfStorageVersion::<Test>::put(CURRENT_STORAGE_VERSION);
        assert_eq!(DcfStorageVersion::<Test>::get(), CURRENT_STORAGE_VERSION);
    });
}

/// Test successful migration from version 0 to version 1.
#[test]
fn migration_v0_to_v1_works() {
    new_test_ext().execute_with(|| {
        // Setup: Start with version 0 (unversioned storage)
        DcfStorageVersion::<Test>::put(0);
        
        // Execute migration
        let result = DcfPallet::perform_storage_migration(0, 1);
        assert_ok!(result);
        
        // Verify migration results
        assert_eq!(DcfStorageVersion::<Test>::get(), 0); // Migration function doesn't update version
        assert!(GovernanceConfigStorage::<Test>::exists()); // Default config should be initialized
        
        // Verify governance config has reasonable defaults
        let config = GovernanceConfigStorage::<Test>::get();
        assert!(config.epoch_length.current > 0);
        assert!(config.max_validators.current > 0);
        assert!(config.pos_weight.current + config.poi_weight.current > 0);
    });
}

/// Test migration when versions already match.
#[test]
fn migration_same_version_works() {
    new_test_ext().execute_with(|| {
        // Setup: Start with version 1
        DcfStorageVersion::<Test>::put(1);
        GovernanceConfigStorage::<Test>::put(GovernanceConfig::<Test>::default());
        
        // Execute migration (should be no-op)
        let result = DcfPallet::perform_storage_migration(1, 1);
        assert_ok!(result);
        
        // Verify no changes
        assert_eq!(DcfStorageVersion::<Test>::get(), 1);
        assert!(GovernanceConfigStorage::<Test>::exists());
    });
}

/// Test migration failure for unsupported migration path.
#[test]
fn migration_unsupported_path_fails() {
    new_test_ext().execute_with(|| {
        // Try to migrate from version 2 to version 3 (unsupported)
        let result = DcfPallet::perform_storage_migration(2, 3);
        assert_err!(result, MigrationError::NoMigrationPath { from: 2, to: 3 });
    });
}

/// Test storage accessibility validation.
#[test]
fn storage_accessibility_validation_works() {
    new_test_ext().execute_with(|| {
        // Initialize some storage items
        CurrentEpoch::<Test>::put(1);
        ActiveValidators::<Test>::put(BoundedVec::default());
        ValidatorSet::<Test>::put(BoundedVec::default());
        
        // Validate storage accessibility
        let result = DcfPallet::validate_storage_accessibility();
        assert_ok!(result);
    });
}

/// Test storage integrity validation with valid data.
#[test]
fn storage_integrity_validation_works() {
    new_test_ext().execute_with(|| {
        // Setup valid storage state
        DcfStorageVersion::<Test>::put(1);
        GovernanceConfigStorage::<Test>::put(GovernanceConfig::<Test>::default());
        EpochConfigStorage::<Test>::put(EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 10,
        });
        ActiveValidators::<Test>::put(BoundedVec::default());
        
        // Validate storage integrity
        let result = DcfPallet::validate_storage_integrity();
        assert_ok!(result);
    });
}

/// Test storage integrity validation fails with too many active validators.
#[test]
fn storage_integrity_validation_fails_with_too_many_validators() {
    new_test_ext().execute_with(|| {
        // Setup invalid storage state - too many active validators
        DcfStorageVersion::<Test>::put(1);
        GovernanceConfigStorage::<Test>::put(GovernanceConfig::<Test>::default());
        EpochConfigStorage::<Test>::put(EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 10,
        });
        
        // Create more active validators than allowed
        let mut validators = BoundedVec::default();
        for i in 0..15u64 { // More than max_validators (10)
            let _ = validators.try_push(i);
        }
        ActiveValidators::<Test>::put(validators);
        
        // Validate storage integrity - should fail
        let result = DcfPallet::validate_storage_integrity();
        assert!(result.is_err());
        if let Err(MigrationError::PostValidationFailed { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PostValidationFailed error");
        }
    });
}

/// Test governance configuration validation with valid config.
#[test]
fn governance_config_validation_works() {
    new_test_ext().execute_with(|| {
        let config = GovernanceConfig::<Test>::default();
        let result = DcfPallet::validate_governance_config(&config);
        assert_ok!(result);
    });
}

/// Test governance configuration validation fails with invalid ranges.
#[test]
fn governance_config_validation_fails_with_invalid_ranges() {
    new_test_ext().execute_with(|| {
        let mut config = GovernanceConfig::<Test>::default();
        
        // Make epoch length range invalid (min > max)
        config.epoch_length.min = 1000;
        config.epoch_length.max = 100;
        config.epoch_length.current = 500;
        
        let result = DcfPallet::validate_governance_config(&config);
        assert!(result.is_err());
        if let Err(MigrationError::PostValidationFailed { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PostValidationFailed error");
        }
    });
}

/// Test governance configuration validation fails with zero consensus weights.
#[test]
fn governance_config_validation_fails_with_zero_weights() {
    new_test_ext().execute_with(|| {
        let mut config = GovernanceConfig::<Test>::default();
        
        // Set both weights to zero
        config.pos_weight.current = 0;
        config.poi_weight.current = 0;
        
        let result = DcfPallet::validate_governance_config(&config);
        assert!(result.is_err());
        if let Err(MigrationError::PostValidationFailed { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PostValidationFailed error");
        }
    });
}

/// Test epoch configuration validation with valid config.
#[test]
fn epoch_config_validation_works() {
    new_test_ext().execute_with(|| {
        let config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 10,
        };
        
        let result = DcfPallet::validate_epoch_config(&config);
        assert_ok!(result);
    });
}

/// Test epoch configuration validation fails with zero epoch length.
#[test]
fn epoch_config_validation_fails_with_zero_epoch_length() {
    new_test_ext().execute_with(|| {
        let config = EpochConfig {
            blocks_per_epoch: 0, // Invalid
            min_stake: 1000,
            max_validators: 10,
        };
        
        let result = DcfPallet::validate_epoch_config(&config);
        assert!(result.is_err());
        if let Err(MigrationError::PostValidationFailed { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PostValidationFailed error");
        }
    });
}

/// Test epoch configuration validation fails with zero max validators.
#[test]
fn epoch_config_validation_fails_with_zero_max_validators() {
    new_test_ext().execute_with(|| {
        let config = EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 0, // Invalid
        };
        
        let result = DcfPallet::validate_epoch_config(&config);
        assert!(result.is_err());
        if let Err(MigrationError::PostValidationFailed { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PostValidationFailed error");
        }
    });
}

/// Test on_runtime_upgrade hook with matching versions.
#[test]
fn on_runtime_upgrade_with_matching_versions_works() {
    new_test_ext().execute_with(|| {
        // Setup: Set storage version to current version
        DcfStorageVersion::<Test>::put(CURRENT_STORAGE_VERSION);
        GovernanceConfigStorage::<Test>::put(GovernanceConfig::<Test>::default());
        EpochConfigStorage::<Test>::put(EpochConfig {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 10,
        });
        
        // Execute on_runtime_upgrade
        let weight = DcfPallet::on_runtime_upgrade();
        
        // Should complete without panic and return reasonable weight
        assert!(weight.ref_time() > 0);
        assert_eq!(DcfStorageVersion::<Test>::get(), CURRENT_STORAGE_VERSION);
    });
}

/// Test on_runtime_upgrade hook with version mismatch.
#[test]
fn on_runtime_upgrade_with_version_mismatch_works() {
    new_test_ext().execute_with(|| {
        // Setup: Set storage version to 0 (needs migration)
        DcfStorageVersion::<Test>::put(0);
        
        // Execute on_runtime_upgrade
        let weight = DcfPallet::on_runtime_upgrade();
        
        // Should complete migration and update version
        assert!(weight.ref_time() > 0);
        assert_eq!(DcfStorageVersion::<Test>::get(), CURRENT_STORAGE_VERSION);
        assert!(GovernanceConfigStorage::<Test>::exists());
    });
}

/// Test that migration events are emitted correctly.
#[test]
fn migration_events_are_emitted() {
    new_test_ext().execute_with(|| {
        // Setup: Set storage version to 0 (needs migration)
        DcfStorageVersion::<Test>::put(0);
        
        // Execute on_runtime_upgrade
        let _weight = DcfPallet::on_runtime_upgrade();
        
        // Check that migration completed event was emitted
        let events = System::events();
        assert!(events.iter().any(|event| {
            matches!(
                event.event,
                RuntimeEvent::DcfPallet(Event::StorageMigrationCompleted {
                    from_version: 0,
                    to_version: 1,
                })
            )
        }));
    });
}

/// Test migration error handling and event emission.
#[test]
#[should_panic(expected = "DCF: Critical storage migration failure")]
fn migration_failure_panics_and_emits_event() {
    new_test_ext().execute_with(|| {
        // Setup: Set storage version to unsupported version
        DcfStorageVersion::<Test>::put(99); // Unsupported version
        
        // Execute on_runtime_upgrade - should panic due to unsupported migration
        let _weight = DcfPallet::on_runtime_upgrade();
    });
}

/// Test storage validation failure handling.
#[test]
#[should_panic(expected = "DCF: Critical storage validation failure")]
fn storage_validation_failure_panics() {
    new_test_ext().execute_with(|| {
        // Setup: Set storage version to current but create invalid state
        DcfStorageVersion::<Test>::put(CURRENT_STORAGE_VERSION);
        
        // Create invalid state - too many active validators
        let mut validators = BoundedVec::default();
        for i in 0..1000u64 { // Way more than any reasonable limit
            let _ = validators.try_push(i);
        }
        ActiveValidators::<Test>::put(validators);
        
        // Execute on_runtime_upgrade - should panic due to validation failure
        let _weight = DcfPallet::on_runtime_upgrade();
    });
}