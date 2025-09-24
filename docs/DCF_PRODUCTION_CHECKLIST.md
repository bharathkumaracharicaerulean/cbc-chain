# DCF Production Checklist

This document provides a comprehensive checklist for deploying and operating the Dynamic Consensus Framework (DCF) in production environments.

## Table of Contents

1. [Pre-Deployment Configuration](#pre-deployment-configuration)
2. [Parameter Ranges and Validation](#parameter-ranges-and-validation)
3. [System Invariants](#system-invariants)
4. [Upgrade Procedures](#upgrade-procedures)
5. [Migration Rules](#migration-rules)
6. [Operator Procedures](#operator-procedures)
7. [Monitoring and Alerting](#monitoring-and-alerting)
8. [Emergency Procedures](#emergency-procedures)

## Pre-Deployment Configuration

### Genesis Configuration Validation

Before deploying DCF, ensure your genesis configuration meets these requirements:

#### Validator Set Configuration
- [ ] **Validator Count**: Initial validator count ≤ `MaxValidators` (default: 100)
- [ ] **Minimum Active**: At least `MinActiveValidators` validators (recommended: 3-7)
- [ ] **No Duplicates**: All validator accounts are unique
- [ ] **Sufficient Stakes**: All validators meet `MinStake` requirement (default: 1,000,000 units)

#### Epoch Configuration
- [ ] **Epoch Length**: `blocks_per_epoch` between 50-1000 blocks (default: 100)
- [ ] **Reasonable Duration**: Epoch duration appropriate for network (6s blocks = 10min epochs)
- [ ] **Min Stake**: `min_stake` sufficient for economic security
- [ ] **Max Validators**: `max_validators` appropriate for network capacity

#### Economic Parameters
- [ ] **Stake Requirements**: Minimum stake provides adequate economic security
- [ ] **Slashing Rates**: Slashing percentages are reasonable (10-30% typical)
- [ ] **Reward Rates**: Reward amounts sustainable long-term
- [ ] **Cooldown Periods**: Leave cooldown prevents gaming (1000-10000 blocks typical)

### Runtime Configuration Constants

Verify these constants are appropriate for your network:

```rust
// Validator Set Limits
MaxValidators: 100,              // Maximum active validators
MinActiveValidators: 3,          // Minimum for network security
MinStake: 1_000_000,            // Minimum stake requirement

// Scoring Parameters
MaxValidatorScore: 10_000,       // Maximum possible score
MinValidatorScore: 30,           // Minimum score for participation
DefaultPosWeight: 5000,          // PoS weight (50%)
DefaultPoiWeight: 5000,          // PoI weight (50%)

// Timing Parameters
EpochLength: 100,                // Blocks per epoch
LeaveCooldown: 1000,            // Cooldown after leaving
ValidatorCooldownPeriod: 5000,   // Re-entry cooldown

// Trust Score Configuration
TrustScoreUptimeWeight: 40,      // Uptime component weight
TrustScoreInferenceWeight: 35,   // Inference component weight
TrustScoreSlashingWeight: 25,    // Slashing penalty weight
MaxTrustScore: 10_000,          // Maximum trust score

// Economic Bounds
ValidatorReward: 10_000,         // Default reward amount
SlashPercent: 20,               // Default slashing percentage
BaseRewardPercentage: 60,        // Base reward allocation
PerformanceRewardPercentage: 25, // Performance bonus allocation
TopPerformerRewardPercentage: 15, // Top performer bonus

// Metadata Limits
MaxValidatorNameLength: 32,      // Validator name size limit
MaxValidatorHistoryLength: 24,   // Performance history epochs
MaxEpochHistory: 168,           // Network epoch history
MaxEvidenceLength: 1024,        // Misbehavior evidence size

// Rate Limiting
MaxProposalsPerBlock: 5,         // DoS protection
MaxJoinsPerBlock: 3,            // Join rate limiting
MaxLeavesPerBlock: 3,           // Leave rate limiting
MaxVotesPerBlock: 20,           // Voting rate limiting
```

## Parameter Ranges and Validation

### Economic Parameters

| Parameter | Minimum | Maximum | Default | Units | Description |
|-----------|---------|---------|---------|-------|-------------|
| `MinStake` | 100,000 | 1,000,000,000 | 1,000,000 | Balance units | Minimum validator stake |
| `ValidatorReward` | 1,000 | 100,000 | 10,000 | Balance units | Default reward amount |
| `SlashPercent` | 1 | 50 | 20 | Percentage | Default slashing rate |
| `BaseRewardPercentage` | 30 | 80 | 60 | Percentage | Base reward pool |
| `PerformanceRewardPercentage` | 10 | 40 | 25 | Percentage | Performance rewards |
| `TopPerformerRewardPercentage` | 5 | 30 | 15 | Percentage | Top performer bonus |

### Scoring Parameters

| Parameter | Minimum | Maximum | Default | Units | Description |
|-----------|---------|---------|---------|-------|-------------|
| `MaxValidatorScore` | 1,000 | 100,000 | 10,000 | Score units | Maximum validator score |
| `MinValidatorScore` | 1 | 1,000 | 30 | Score units | Minimum active score |
| `DefaultPosWeight` | 1,000 | 9,000 | 5,000 | Basis points | PoS weight (out of 10,000) |
| `DefaultPoiWeight` | 1,000 | 9,000 | 5,000 | Basis points | PoI weight (out of 10,000) |
| `MaxTrustScore` | 1,000 | 100,000 | 10,000 | Score units | Maximum trust score |

### Timing Parameters

| Parameter | Minimum | Maximum | Default | Units | Description |
|-----------|---------|---------|---------|-------|-------------|
| `EpochLength` | 50 | 1,000 | 100 | Blocks | Blocks per epoch |
| `LeaveCooldown` | 100 | 50,000 | 1,000 | Blocks | Leave cooldown period |
| `ValidatorCooldownPeriod` | 500 | 100,000 | 5,000 | Blocks | Re-entry cooldown |
| `MinProposalInterval` | 1 | 1,000 | 20 | Blocks | Min proposal interval |
| `MinValidatorStatusInterval` | 10 | 5,000 | 50 | Blocks | Min status change interval |

### Validator Set Parameters

| Parameter | Minimum | Maximum | Default | Units | Description |
|-----------|---------|---------|---------|-------|-------------|
| `MaxValidators` | 3 | 1,000 | 100 | Count | Maximum active validators |
| `MinActiveValidators` | 1 | 10 | 3 | Count | Minimum for security |
| `MaxValidatorNameLength` | 8 | 128 | 32 | Bytes | Validator name limit |
| `MaxValidatorHistoryLength` | 12 | 100 | 24 | Epochs | Performance history |
| `MaxEpochHistory` | 50 | 1,000 | 168 | Epochs | Network history |

## System Invariants

The DCF system maintains critical invariants that must never be violated:

### Economic Invariants
- [ ] **Reserved Balance Non-Negative**: `reserved_balance >= 0` for all validators
- [ ] **Reserved Greater Than Slashed**: `reserved_balance >= total_slashed_amount`
- [ ] **Minimum Stake Enforcement**: Active validators maintain `stake >= MinStake`
- [ ] **No Arithmetic Overflow**: All balance operations use checked arithmetic
- [ ] **Slashing Bounds**: Per-epoch slashing ≤ configured maximum

### Validator Set Invariants
- [ ] **Active Set Size Limit**: `active_validators.len() <= MaxValidators`
- [ ] **Minimum Active Validators**: `active_validators.len() >= MinActiveValidators`
- [ ] **Cooldown Enforcement**: Validators in cooldown cannot rejoin
- [ ] **No Duplicate Validators**: Each account appears at most once in validator set
- [ ] **Score Consistency**: Active validators have scores ≥ `MinValidatorScore`

### Score Invariants
- [ ] **Score Bounds**: All scores within `[0, MaxValidatorScore]`
- [ ] **Trust Score Bounds**: Trust scores within `[0, MaxTrustScore]`
- [ ] **Weight Consistency**: `pos_weight + poi_weight > 0`
- [ ] **Deterministic Calculation**: Score calculations are deterministic
- [ ] **Bounded Growth**: Trust score changes respect growth/decay limits

### Temporal Invariants
- [ ] **Epoch Progression**: Epochs advance monotonically
- [ ] **Cooldown Respect**: Leave cooldown periods always respected
- [ ] **Block Finality**: `finalized_block <= current_block`
- [ ] **Finality Progression**: Finality marker advances or stays constant
- [ ] **Author Sequence**: Block authors follow deterministic sequence

## Upgrade Procedures

### Pre-Upgrade Checklist
- [ ] **Backup Storage**: Complete storage backup before upgrade
- [ ] **Test Migration**: Test migration on staging environment
- [ ] **Validator Notification**: Notify validators of upgrade schedule
- [ ] **Monitoring Setup**: Ensure monitoring systems are ready
- [ ] **Rollback Plan**: Prepare rollback procedures if needed

### Storage Migration Process
1. **Version Check**: Verify current storage version matches expected
2. **Migration Execution**: Run migration with proper error handling
3. **Validation**: Validate migrated data integrity
4. **Invariant Check**: Verify all system invariants after migration
5. **Event Emission**: Emit migration completion events

### Post-Upgrade Validation
- [ ] **Storage Version**: Verify storage version updated correctly
- [ ] **Invariant Check**: Run comprehensive invariant validation
- [ ] **Validator Set**: Verify validator set integrity
- [ ] **Score Consistency**: Verify score calculations work correctly
- [ ] **API Compatibility**: Test all runtime APIs function properly

## Migration Rules

### Safe Field Additions
- Add new optional fields with sensible defaults
- Use `Option<T>` for truly optional data
- Provide migration logic for existing records
- Test with existing production data

### Safe Field Removals
- Mark fields as deprecated first
- Ensure no active code references removed fields
- Clean up storage in migration
- Document breaking changes

### Unsafe Operations
- ❌ Changing field types without migration
- ❌ Removing required fields without migration
- ❌ Changing storage key structures
- ❌ Modifying enum variant discriminants

### Migration Best Practices
- Always increment storage version
- Provide comprehensive migration tests
- Handle migration failures gracefully
- Log migration progress and results
- Validate data integrity after migration

## Operator Procedures

### Validator Management

#### Adding New Validators
1. **Stake Verification**: Ensure validator has sufficient stake
2. **Join Process**: Validator calls `join_validators()`
3. **Validation**: System validates stake and cooldown status
4. **Activation**: Validator added to set if requirements met
5. **Monitoring**: Monitor validator performance after joining

#### Removing Validators
1. **Leave Request**: Validator calls `leave_validators()`
2. **Cooldown Period**: Validator enters cooldown (stake remains reserved)
3. **Automatic Removal**: System removes validator after cooldown
4. **Stake Release**: Reserved stake returned to validator
5. **Re-entry Cooldown**: Additional cooldown before rejoining allowed

#### Emergency Validator Ejection
1. **Evidence Collection**: Gather misbehavior evidence
2. **Proposal Submission**: Submit ejection proposal with evidence
3. **Voting Process**: Validators vote on ejection proposal
4. **Execution**: Execute approved ejection proposal
5. **Slashing**: Apply appropriate slashing if warranted

### Slashing Procedures

#### Automated Slashing
- **Threshold Detection**: System detects misbehavior threshold
- **Automatic Execution**: Slashing applied automatically
- **Event Emission**: Slashing events emitted for monitoring
- **Balance Update**: Validator balance updated immediately

#### Manual Slashing
1. **Evidence Review**: Review misbehavior evidence
2. **Slashing Proposal**: Create slashing proposal with amount
3. **Validator Voting**: Validators vote on slashing proposal
4. **Execution**: Execute approved slashing
5. **Documentation**: Document slashing reason and amount

### Reward Distribution

#### Epoch Rewards
- **Automatic Distribution**: Rewards distributed at epoch end
- **Performance Basis**: Based on block authorship and scores
- **Tier System**: Base, performance, and top performer tiers
- **Event Tracking**: Reward events for transparency

#### Manual Rewards
1. **Performance Review**: Review validator performance
2. **Reward Proposal**: Create reward proposal with amount
3. **Approval Process**: Get proposal approved by validators
4. **Distribution**: Execute approved reward distribution
5. **Record Keeping**: Maintain reward distribution records

### Epoch Management

#### Normal Epoch Transitions
- **Automatic Progression**: Epochs advance automatically
- **Score Updates**: Validator scores updated
- **Set Rebalancing**: Active set rebalanced based on scores
- **Invariant Validation**: System invariants checked

#### Manual Epoch Advancement
1. **Governance Mode**: Enable governance mode if needed
2. **Manual Trigger**: Call `sudo_advance_epoch()` with root
3. **Validation**: Verify epoch transition completed correctly
4. **Monitoring**: Monitor system state after transition
5. **Mode Reset**: Disable governance mode if appropriate

## Monitoring and Alerting

### Critical Metrics to Monitor

#### System Health
- [ ] **Invariant Violations**: Any invariant violation events
- [ ] **Finality Progression**: Finality marker advancement
- [ ] **Epoch Transitions**: Successful epoch transitions
- [ ] **Storage Version**: Correct storage version
- [ ] **Migration Status**: Migration completion and success

#### Validator Metrics
- [ ] **Active Validator Count**: Within expected range
- [ ] **Validator Scores**: Score distribution and outliers
- [ ] **Cooldown Status**: Validators in cooldown periods
- [ ] **Join/Leave Rates**: Validator churn rates
- [ ] **Performance Metrics**: Block authorship and missed blocks

#### Economic Metrics
- [ ] **Total Staked**: Total stake in the system
- [ ] **Reserved Balances**: Validator reserved balances
- [ ] **Slashing Events**: Frequency and amounts
- [ ] **Reward Distribution**: Reward amounts and recipients
- [ ] **Balance Consistency**: Economic invariant compliance

#### Performance Metrics
- [ ] **Block Production**: Block authorship distribution
- [ ] **Score Calculations**: Score computation performance
- [ ] **API Response Times**: Runtime API performance
- [ ] **Storage Usage**: Storage growth and efficiency
- [ ] **Weight Consumption**: Dispatchable weight usage

### Alert Thresholds

#### Critical Alerts (Immediate Response)
- Any invariant violation event
- Finality regression or stall
- Storage version mismatch
- Migration failure
- Active validator count below minimum

#### Warning Alerts (Monitor Closely)
- Validator score distribution anomalies
- High validator churn rates
- Excessive slashing events
- API response time degradation
- Storage usage growth rate

#### Info Alerts (Routine Monitoring)
- Epoch transition events
- Validator join/leave events
- Reward distribution events
- Parameter update events
- Performance milestone events

## Emergency Procedures

### Invariant Violation Response
1. **Immediate Assessment**: Determine violation severity
2. **System Halt**: Consider halting block production if critical
3. **Root Cause Analysis**: Investigate violation cause
4. **Corrective Action**: Apply fixes or rollback if needed
5. **Validation**: Verify invariants restored
6. **Documentation**: Document incident and resolution

### Finality Stall Response
1. **Detection**: Monitor finality progression
2. **Validator Check**: Verify validator set health
3. **Network Analysis**: Check network connectivity
4. **Manual Intervention**: Consider manual epoch advancement
5. **Recovery Validation**: Verify finality resumption

### Mass Validator Exit
1. **Threshold Monitoring**: Monitor active validator count
2. **Emergency Recruitment**: Fast-track new validator onboarding
3. **Cooldown Adjustment**: Consider temporary cooldown reduction
4. **Network Security**: Ensure minimum security threshold
5. **Stability Restoration**: Restore normal validator levels

### Storage Corruption
1. **Backup Restoration**: Restore from known good backup
2. **Data Validation**: Validate restored data integrity
3. **Invariant Check**: Verify all invariants after restoration
4. **Incremental Recovery**: Apply missing transactions if possible
5. **System Restart**: Restart with validated state

### Parameter Governance Emergency
1. **Parameter Validation**: Check parameter ranges and consistency
2. **Emergency Override**: Use root access for critical fixes
3. **Rollback Capability**: Revert to previous known good parameters
4. **Validation Testing**: Test parameter changes thoroughly
5. **Gradual Deployment**: Phase in parameter changes if possible

## Deployment Checklist

### Pre-Production Testing
- [ ] **Unit Tests**: All unit tests pass
- [ ] **Integration Tests**: Runtime API tests pass
- [ ] **Property Tests**: Fuzz testing with multiple seeds
- [ ] **Load Testing**: Multi-epoch scenarios with max validators
- [ ] **Migration Testing**: Storage migration on production-like data

### Production Deployment
- [ ] **Genesis Validation**: Genesis configuration validated
- [ ] **Parameter Review**: All parameters within safe ranges
- [ ] **Monitoring Setup**: Monitoring and alerting configured
- [ ] **Backup Strategy**: Backup and recovery procedures ready
- [ ] **Operator Training**: Operators trained on procedures

### Post-Deployment Validation
- [ ] **System Health**: All invariants satisfied
- [ ] **Validator Set**: Validator set functioning correctly
- [ ] **Score Calculations**: Scores computed correctly
- [ ] **API Functionality**: All runtime APIs working
- [ ] **Event Emission**: Events emitted correctly

## Maintenance Procedures

### Regular Health Checks
- **Daily**: Monitor critical metrics and alerts
- **Weekly**: Review validator performance and churn
- **Monthly**: Analyze long-term trends and capacity
- **Quarterly**: Review parameter settings and optimization

### Capacity Planning
- Monitor validator set growth trends
- Plan for MaxValidators limit increases
- Assess storage growth and optimization needs
- Evaluate performance scaling requirements

### Security Reviews
- Regular audit of validator set composition
- Review of slashing events and patterns
- Analysis of governance proposal patterns
- Assessment of economic security parameters

This checklist should be customized for your specific network requirements and operational procedures. Regular review and updates ensure continued production readiness as the network evolves.