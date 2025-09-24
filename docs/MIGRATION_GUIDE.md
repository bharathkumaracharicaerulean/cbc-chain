# DCF Pallet Storage Migration Guide

This guide provides comprehensive instructions for safely performing storage migrations in the DCF (Decentralized Consensus Framework) pallet. Storage migrations are critical operations that must be performed carefully to prevent data loss and maintain system integrity.

## Overview

The DCF pallet implements a robust storage versioning and migration system that:

- Tracks storage schema versions to detect when migrations are needed
- Provides validation mechanisms to ensure data integrity before and after migrations
- Implements safe migration procedures with rollback capabilities
- Maintains audit trails of all migration activities

## Storage Version History

### Version 1 (Current Production Version)
- **Description**: Initial production-ready storage layout
- **Features**: 
  - Comprehensive governance configuration with parameter ranges
  - Economic invariant checking and reporting
  - Validator lifecycle management with cooldown periods
  - Trust score calculation and history tracking
  - Storage version tracking and validation
- **Migration Path**: From version 0 (unversioned) to version 1

## Migration Process

### Automatic Migration During Runtime Upgrade

The DCF pallet automatically handles storage migrations during runtime upgrades through the `on_runtime_upgrade` hook:

1. **Version Detection**: Compares current storage version with expected runtime version
2. **Migration Execution**: Runs appropriate migration steps if versions differ
3. **Validation**: Validates storage integrity before and after migration
4. **Version Update**: Updates storage version to match runtime expectations
5. **Event Emission**: Emits events documenting migration success or failure

### Manual Migration Verification

After a runtime upgrade, verify migration success by:

```bash
# Check storage version matches expected version
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "dcf_getStorageVersion", "params": []}' http://localhost:9933

# Verify governance configuration is accessible
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "dcf_getGovernanceConfig", "params": []}' http://localhost:9933

# Check for migration events in recent blocks
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock", "params": []}' http://localhost:9933
```

## Safe Field Addition Procedures

### Adding New Storage Items

When adding new storage items to the DCF pallet:

1. **Version Planning**: Increment `CURRENT_STORAGE_VERSION` constant
2. **Default Values**: Ensure new storage items have sensible defaults
3. **Migration Logic**: Add migration step to initialize new storage items
4. **Validation**: Add validation rules for new storage items
5. **Documentation**: Update this guide with new version information

#### Example: Adding New Storage Item

```rust
// 1. Add new storage item with default value
#[pallet::storage]
#[pallet::getter(fn new_feature_config)]
pub type NewFeatureConfig<T: Config> = StorageValue<_, NewFeatureConfigStruct, ValueQuery>;

// 2. Update CURRENT_STORAGE_VERSION
pub const CURRENT_STORAGE_VERSION: u32 = 2; // Increment from 1 to 2

// 3. Add migration logic in perform_storage_migration
if from_version == 1 && to_version == 2 {
    // Initialize new storage item with default value
    if !NewFeatureConfig::<T>::exists() {
        NewFeatureConfig::<T>::put(NewFeatureConfigStruct::default());
    }
    // Add validation for new storage item
    weight = weight.saturating_add(Self::validate_new_feature_config()?);
}
```

### Adding Fields to Existing Structures

When adding fields to existing storage structures:

1. **Backward Compatibility**: Ensure new fields are optional or have defaults
2. **Codec Compatibility**: Use `#[codec(skip)]` for fields that shouldn't be encoded
3. **Migration Logic**: Provide migration to populate new fields
4. **Validation**: Validate new fields during migration

#### Example: Adding Field to Existing Struct

```rust
// Before (Version 1)
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct ValidatorMetadata {
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub created_at: u64,
}

// After (Version 2) - Safe addition
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct ValidatorMetadata {
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub created_at: u64,
    // New field with default value
    pub updated_at: Option<u64>, // Optional for backward compatibility
}

// Migration logic to populate new field
fn migrate_validator_metadata_v2() -> Weight {
    let current_timestamp = Self::current_timestamp();
    ValidatorMetadata::<T>::translate(|_key, mut metadata: ValidatorMetadata| {
        metadata.updated_at = Some(current_timestamp);
        Some(metadata)
    });
    // Return appropriate weight
}
```

## Safe Field Removal Procedures

### Removing Storage Items

When removing storage items:

1. **Deprecation Period**: Mark storage items as deprecated before removal
2. **Data Migration**: Migrate important data to new storage items
3. **Cleanup Logic**: Add cleanup logic to remove deprecated storage
4. **Version Increment**: Increment storage version for removal

#### Example: Removing Deprecated Storage Item

```rust
// Version N: Mark as deprecated
#[pallet::storage]
#[deprecated = "Use NewStorageItem instead"]
pub type OldStorageItem<T: Config> = StorageValue<_, OldStruct, OptionQuery>;

// Version N+1: Remove and migrate data
// Remove the storage item definition entirely
// Add migration logic:
if from_version == N && to_version == N+1 {
    // Migrate data from old to new storage
    if let Some(old_data) = OldStorageItem::<T>::take() {
        let new_data = convert_old_to_new(old_data);
        NewStorageItem::<T>::put(new_data);
    }
}
```

### Removing Fields from Existing Structures

When removing fields from structures:

1. **Gradual Removal**: Use `#[codec(skip)]` to stop encoding/decoding
2. **Migration Logic**: Remove field data during migration
3. **Validation**: Ensure removal doesn't break existing functionality

#### Example: Removing Field from Struct

```rust
// Version N: Mark field for removal
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct ValidatorMetadata {
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub created_at: u64,
    #[codec(skip)] // Stop encoding/decoding this field
    pub deprecated_field: Option<u64>,
}

// Version N+1: Remove field entirely
#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct ValidatorMetadata {
    pub name: BoundedVec<u8, ConstU32<32>>,
    pub created_at: u64,
    // deprecated_field removed
}
```

## Migration Testing

### Pre-Migration Testing

Before deploying migrations:

1. **Unit Tests**: Test migration logic with various data scenarios
2. **Integration Tests**: Test complete migration process end-to-end
3. **Validation Tests**: Verify all validation rules work correctly
4. **Rollback Tests**: Test migration failure and rollback scenarios

#### Example Migration Test

```rust
#[test]
fn test_storage_migration_v0_to_v1() {
    new_test_ext().execute_with(|| {
        // Setup: Simulate version 0 storage state
        StorageVersion::<Test>::put(0);
        
        // Execute migration
        let weight = Pallet::<Test>::perform_storage_migration(0, 1).unwrap();
        
        // Verify: Check migration results
        assert_eq!(StorageVersion::<Test>::get(), 1);
        assert!(GovernanceConfigStorage::<Test>::exists());
        
        // Verify weight calculation
        assert!(weight > Weight::zero());
    });
}
```

### Post-Migration Validation

After migration deployment:

1. **Storage Integrity**: Verify all storage items are accessible
2. **Data Consistency**: Check that migrated data maintains consistency
3. **Functionality Tests**: Verify all pallet functions work correctly
4. **Performance Tests**: Ensure migration didn't degrade performance

## Troubleshooting

### Common Migration Issues

#### Storage Version Mismatch
**Symptoms**: Runtime upgrade fails with storage version error
**Solution**: 
1. Check current storage version: `StorageVersion::<T>::get()`
2. Verify migration path exists for current → target version
3. Add missing migration logic if needed

#### Migration Validation Failure
**Symptoms**: Migration fails during validation phase
**Solution**:
1. Check validation error details in logs
2. Verify storage data integrity
3. Fix data inconsistencies before retry

#### Insufficient Migration Weight
**Symptoms**: Migration times out or fails due to weight limits
**Solution**:
1. Optimize migration logic for better performance
2. Split large migrations into smaller chunks
3. Increase block weight limits temporarily if needed

### Recovery Procedures

#### Migration Failure Recovery
If migration fails:

1. **Stop Network**: Halt block production to prevent further issues
2. **Analyze Logs**: Review migration failure logs for root cause
3. **Fix Issues**: Address underlying problems (data corruption, logic errors)
4. **Retry Migration**: Restart network with fixed migration logic
5. **Validate Results**: Thoroughly test after successful migration

#### Data Corruption Recovery
If data corruption is detected:

1. **Backup Current State**: Create snapshot of current storage
2. **Restore from Backup**: Restore from last known good backup
3. **Replay Transactions**: Replay transactions since backup if possible
4. **Validate Integrity**: Verify restored data integrity
5. **Resume Operations**: Resume normal network operations

## Best Practices

### Development Best Practices

1. **Version Everything**: Always increment version for breaking changes
2. **Test Thoroughly**: Test migrations with realistic data volumes
3. **Document Changes**: Maintain detailed migration documentation
4. **Validate Early**: Implement validation at every migration step
5. **Plan Rollbacks**: Always have rollback procedures ready

### Deployment Best Practices

1. **Staged Rollout**: Deploy migrations to testnets first
2. **Monitor Closely**: Monitor migration progress and system health
3. **Have Backups**: Maintain recent backups before migrations
4. **Communication**: Communicate migration plans to stakeholders
5. **Emergency Procedures**: Have emergency rollback procedures ready

### Maintenance Best Practices

1. **Regular Audits**: Regularly audit storage for consistency
2. **Performance Monitoring**: Monitor storage performance over time
3. **Cleanup Old Data**: Remove deprecated data after safe periods
4. **Update Documentation**: Keep migration guide current
5. **Version Planning**: Plan future versions and migration paths

## Migration Checklist

### Pre-Migration Checklist

- [ ] Storage version incremented in code
- [ ] Migration logic implemented and tested
- [ ] Validation rules added for new/changed storage
- [ ] Unit tests written and passing
- [ ] Integration tests written and passing
- [ ] Documentation updated
- [ ] Backup procedures verified
- [ ] Rollback procedures tested
- [ ] Stakeholders notified

### Post-Migration Checklist

- [ ] Storage version matches expected version
- [ ] All storage items accessible
- [ ] Governance configuration valid
- [ ] Validator data integrity verified
- [ ] All pallet functions working
- [ ] Performance metrics normal
- [ ] No error events emitted
- [ ] Migration events properly emitted
- [ ] Documentation updated with results
- [ ] Lessons learned documented

## Support and Resources

### Getting Help

If you encounter issues with DCF storage migrations:

1. **Check Logs**: Review runtime logs for detailed error information
2. **Consult Documentation**: Review this guide and related documentation
3. **Test Environment**: Reproduce issues in test environment
4. **Community Support**: Reach out to the development community
5. **Professional Support**: Contact professional support if available

### Additional Resources

- [Substrate Storage Migration Guide](https://docs.substrate.io/reference/how-to-guides/storage-migrations/)
- [FRAME Storage Documentation](https://docs.substrate.io/build/runtime-storage/)
- [DCF Pallet Documentation](./README.md)
- [DCF Runtime Integration Guide](../../cbc-runtime/README.md)

---

**Important**: Storage migrations are critical operations that can cause data loss if performed incorrectly. Always test thoroughly in development environments before deploying to production networks.