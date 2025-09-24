# DCF Runtime API Contract

## Version 1.0

This document defines the comprehensive contract for the DCF (Decentralized Consensus Framework) Runtime API. This contract guarantees stability and backward compatibility within the same API version.

## API Version Management

### Current Version
- **API Version**: 1
- **Contract Version**: 1.0
- **Last Updated**: Production Readiness Implementation

### Version History
- **Version 1**: Initial production-ready API contract with comprehensive governance, invariant checking, and validator lifecycle management

### Breaking Change Policy
Breaking changes that require version increment:
- Changing method signatures (name, parameters, return types)
- Removing existing methods
- Changing the semantics of existing methods
- Modifying data structures used in API responses

Non-breaking changes that do NOT require version increment:
- Adding new methods
- Adding optional fields to existing structures (with proper defaults)
- Improving documentation
- Internal implementation changes that don't affect the API contract

## API Methods

### Version Management

#### `get_api_version() -> u32`
Returns the current DCF Runtime API version.

**Returns:**
- `u32`: Current API version number

**Example:**
```rust
let version = runtime_api.get_api_version();
assert_eq!(version, 1);
```

**Guarantees:**
- This method will always be available in all API versions
- Return type will never change
- Method signature is guaranteed stable

### Validator Information

#### `get_validator_scores() -> Vec<(AccountId, u64)>`
Returns scores for all validators in the system.

**Returns:**
- `Vec<(AccountId, u64)>`: List of (validator_account, final_score) pairs

**Example:**
```rust
let scores = runtime_api.get_validator_scores();
for (validator, score) in scores {
    println!("Validator {:?} has score {}", validator, score);
}
```

**Error Cases:**
- Returns empty vector if no validators exist
- Returns 0 score for validators without state

#### `get_validator_stake_score(validator: AccountId) -> u64`
Returns the Proof-of-Stake score for a specific validator.

**Parameters:**
- `validator`: The validator account to query

**Returns:**
- `u64`: Current PoS score based on stake amount

**Example:**
```rust
let alice_pos_score = runtime_api.get_validator_stake_score(alice_account);
```

**Error Cases:**
- Returns 0 if validator doesn't exist
- Returns 0 if validator has no stake

#### `get_validator_inference_score(validator: AccountId) -> u64`
Returns the Proof-of-Inference score for a specific validator.

**Parameters:**
- `validator`: The validator account to query

**Returns:**
- `u64`: Current PoI score based on inference performance

**Example:**
```rust
let alice_poi_score = runtime_api.get_validator_inference_score(alice_account);
```

**Error Cases:**
- Returns 0 if validator doesn't exist
- Returns 0 if validator has no inference history

#### `is_validator_active(validator: AccountId) -> bool`
Checks if a validator is currently active in the consensus.

**Parameters:**
- `validator`: The validator account to check

**Returns:**
- `bool`: True if validator is active, false otherwise

**Example:**
```rust
if runtime_api.is_validator_active(alice_account) {
    println!("Alice is actively participating in consensus");
}
```

**Error Cases:**
- Returns false if validator doesn't exist
- Returns false if validator is in cooldown

#### `get_active_validators() -> Vec<AccountId>`
Returns the list of all currently active validators.

**Returns:**
- `Vec<AccountId>`: List of active validator accounts

**Example:**
```rust
let active_validators = runtime_api.get_active_validators();
println!("Currently {} validators are active", active_validators.len());
```

**Error Cases:**
- Returns empty vector if no validators are active

#### `get_validator_profile(account_id: AccountId) -> Option<ValidatorProfile<AccountId, Balance, BlockNumber>>`
Returns comprehensive profile information for a validator.

**Parameters:**
- `account_id`: The validator account to query

**Returns:**
- `Some(ValidatorProfile)`: Complete validator profile if found
- `None`: If validator doesn't exist

**Example:**
```rust
if let Some(profile) = runtime_api.get_validator_profile(alice_account) {
    println!("Validator name: {:?}", profile.name);
    println!("Trust score: {}", profile.trust_score);
}
```

**ValidatorProfile Structure:**
```rust
pub struct ValidatorProfile<AccountId, Balance, BlockNumber> {
    pub account_id: AccountId,
    pub name: Option<Vec<u8>>,
    pub stake: Balance,
    pub trust_score: u64,
    pub uptime_stats: UptimeStats,
    pub performance_metrics: PerformanceMetrics,
    pub last_active_epoch: u32,
    pub cooldown_status: Option<BlockNumber>,
}
```

### Consensus Information

#### `get_current_epoch() -> u32`
Returns the current epoch number.

**Returns:**
- `u32`: Current epoch number (starts from 0)

**Example:**
```rust
let current_epoch = runtime_api.get_current_epoch();
```

#### `get_consensus_weights() -> (u64, u64)`
Returns the current PoS and PoI weights used in score calculation.

**Returns:**
- `(u64, u64)`: Tuple of (pos_weight, poi_weight)

**Example:**
```rust
let (pos_weight, poi_weight) = runtime_api.get_consensus_weights();
println!("PoS weight: {}, PoI weight: {}", pos_weight, poi_weight);
```

**Guarantees:**
- Weights always sum to precision factor (typically 10000)

#### `get_expected_author(block_number: u32) -> Option<AccountId>`
Returns the expected block author for a given block number.

**Parameters:**
- `block_number`: Block number to query

**Returns:**
- `Some(AccountId)`: Expected author if deterministic sequence exists
- `None`: If block number is outside current epoch or no sequence available

**Example:**
```rust
if let Some(expected_author) = runtime_api.get_expected_author(12345) {
    println!("Block 12345 should be authored by {:?}", expected_author);
}
```

### Governance Configuration

#### `get_governance_config() -> Vec<u8>`
Returns the complete governance configuration with parameter ranges.

**Returns:**
- `Vec<u8>`: Encoded GovernanceConfig structure

**Example:**
```rust
let config_bytes = runtime_api.get_governance_config();
let config: GovernanceConfig<Runtime> = Decode::decode(&mut &config_bytes[..]).unwrap();
```

**GovernanceConfig Structure:**
```rust
pub struct GovernanceConfig<T: Config> {
    pub epoch_length: ParameterRange<u32>,
    pub min_stake: ParameterRange<T::Balance>,
    pub min_score_threshold: ParameterRange<u64>,
    pub max_validators: ParameterRange<u32>,
    pub pos_weight: ParameterRange<u64>,
    pub poi_weight: ParameterRange<u64>,
    // ... additional parameters
}
```

#### `get_parameter_value(parameter: Vec<u8>) -> Option<Vec<u8>>`
Returns the current value of a specific DCF parameter.

**Parameters:**
- `parameter`: Encoded ParameterType to query

**Returns:**
- `Some(Vec<u8>)`: Encoded current value if parameter exists
- `None`: If parameter type is invalid or not found

**Example:**
```rust
let param_type = ParameterType::EpochLength.encode();
if let Some(value_bytes) = runtime_api.get_parameter_value(param_type) {
    let epoch_length: u32 = Decode::decode(&mut &value_bytes[..]).unwrap();
}
```

#### `validate_parameter_value(parameter: Vec<u8>, value: Vec<u8>) -> bool`
Validates if a parameter value would be accepted by governance.

**Parameters:**
- `parameter`: Encoded ParameterType to validate
- `value`: Encoded proposed new value

**Returns:**
- `bool`: True if value would be accepted, false if rejected

**Example:**
```rust
let param_type = ParameterType::MaxValidators.encode();
let new_value = 150u32.encode();
let is_valid = runtime_api.validate_parameter_value(param_type, new_value);
```

### Invariant Monitoring

#### `get_latest_invariant_report() -> Option<Vec<u8>>`
Returns the most recent invariant validation report.

**Returns:**
- `Some(Vec<u8>)`: Encoded InvariantReport if available
- `None`: If no report has been generated yet

**Example:**
```rust
if let Some(report_bytes) = runtime_api.get_latest_invariant_report() {
    let report: InvariantReport = Decode::decode(&mut &report_bytes[..]).unwrap();
    if !report.violations.is_empty() {
        println!("Warning: {} invariant violations detected", report.violations.len());
    }
}
```

**InvariantReport Structure:**
```rust
pub struct InvariantReport {
    pub epoch: u32,
    pub block_number: u32,
    pub violations: Vec<InvariantViolation>,
    pub severity: InvariantSeverity,
    pub timestamp: u64,
}
```

#### `get_invariant_report_for_epoch(epoch: u32) -> Option<Vec<u8>>`
Returns the invariant report for a specific epoch.

**Parameters:**
- `epoch`: Epoch number to query

**Returns:**
- `Some(Vec<u8>)`: Encoded InvariantReport if found
- `None`: If no report exists for the specified epoch

**Example:**
```rust
if let Some(report_bytes) = runtime_api.get_invariant_report_for_epoch(100) {
    let report: InvariantReport = Decode::decode(&mut &report_bytes[..]).unwrap();
}
```

#### `has_invariant_violations() -> bool`
Quick health check for active invariant violations.

**Returns:**
- `bool`: True if there are active violations, false otherwise

**Example:**
```rust
if runtime_api.has_invariant_violations() {
    println!("ALERT: System has active invariant violations!");
}
```

### System Metrics

#### `get_system_constants() -> SystemConstants<Balance, BlockNumber>`
Returns system-wide constants and configuration.

**Returns:**
- `SystemConstants`: Structure containing all system constants

**Example:**
```rust
let constants = runtime_api.get_system_constants();
println!("Min stake: {}", constants.min_stake);
println!("Max validators: {}", constants.max_validators);
```

#### `get_epoch_config() -> EpochConfig`
Returns the current epoch configuration.

**Returns:**
- `EpochConfig`: Current epoch configuration

**Example:**
```rust
let config = runtime_api.get_epoch_config();
println!("Blocks per epoch: {}", config.blocks_per_epoch);
```

#### `get_validator_set_info() -> (u32, u32, u32)`
Returns validator set statistics.

**Returns:**
- `(u32, u32, u32)`: Tuple of (total_validators, active_validators, cooldown_validators)

**Example:**
```rust
let (total, active, cooldown) = runtime_api.get_validator_set_info();
println!("Validators: {} total, {} active, {} in cooldown", total, active, cooldown);
```

### Finality Information

#### `get_last_finalized_block() -> u32`
Returns the last finalized block number.

**Returns:**
- `u32`: Block number of the last finalized block

**Example:**
```rust
let finalized = runtime_api.get_last_finalized_block();
```

#### `is_block_finalized(block_number: u32) -> bool`
Checks if a specific block has been finalized.

**Parameters:**
- `block_number`: Block number to check

**Returns:**
- `bool`: True if block is finalized, false otherwise

**Example:**
```rust
if runtime_api.is_block_finalized(12345) {
    println!("Block 12345 is finalized");
}
```

#### `get_finality_info() -> (u32, u32)`
Returns comprehensive finality information.

**Returns:**
- `(u32, u32)`: Tuple of (last_finalized_block, current_block)

**Example:**
```rust
let (finalized, current) = runtime_api.get_finality_info();
let lag = current - finalized;
println!("Finality lag: {} blocks", lag);
```

## Data Structures

### Core Types

#### ParameterType
Enumeration of all governable DCF parameters:

```rust
pub enum ParameterType {
    EpochLength,
    MinStake,
    MaxValidators,
    MinScoreThreshold,
    PosWeight,
    PoiWeight,
    RewardRatio,
    SlashRatio,
    LeaveCooldown,
    // ... additional parameters
}
```

#### InvariantViolation
Structure representing a detected invariant violation:

```rust
pub enum InvariantViolation<T: Config> {
    Economic { violation: EconomicViolationType<T> },
    Validator { violation: ValidatorViolationType<T> },
    Temporal { violation: TemporalViolationType },
}
```

#### InvariantSeverity
Severity levels for invariant violations:

```rust
pub enum InvariantSeverity {
    Low,      // System can continue operating
    Medium,   // Attention required but not critical
    High,     // Immediate attention required
    Critical, // System integrity compromised
}
```

## Error Handling

### Common Error Patterns

1. **Missing Data**: Methods return `None` or empty collections when data doesn't exist
2. **Invalid Parameters**: Methods return default values (0, false, empty) for invalid inputs
3. **System Errors**: Internal errors are handled gracefully without panicking

### Error Response Examples

```rust
// Validator doesn't exist
let score = runtime_api.get_validator_stake_score(invalid_account); // Returns 0

// Epoch doesn't exist
let report = runtime_api.get_invariant_report_for_epoch(999999); // Returns None

// Block not in current epoch
let author = runtime_api.get_expected_author(999999); // Returns None
```

## Performance Considerations

### Query Optimization
- All methods are optimized for runtime query performance
- Complex calculations are cached when possible
- Large data sets are paginated or limited in size

### Resource Limits
- Event payloads are bounded to prevent excessive memory usage
- Vector returns are limited to reasonable sizes
- Encoded data structures use efficient serialization

## Integration Guidelines

### Version Compatibility
Always check API version before making calls:

```rust
let api_version = runtime_api.get_api_version();
if api_version != EXPECTED_VERSION {
    // Handle version mismatch
    return Err("Incompatible API version");
}
```

### Error Handling
Handle optional returns gracefully:

```rust
match runtime_api.get_validator_profile(account) {
    Some(profile) => {
        // Process profile data
    },
    None => {
        // Handle missing validator
    }
}
```

### Event Monitoring
Monitor for API version changes:

```rust
// Subscribe to ApiVersionChanged events
if let Some(event) = events.find_first::<ApiVersionChanged>() {
    println!("API version changed from {} to {}", event.old_version, event.new_version);
    // Update client code accordingly
}
```

## Testing and Validation

### Integration Test Examples
The API contract includes comprehensive integration tests that validate:

1. **Method Signatures**: All methods can be called with correct parameters
2. **Return Types**: All methods return expected data types
3. **Error Handling**: All error cases are handled gracefully
4. **Data Consistency**: Related methods return consistent data
5. **Performance**: All methods complete within acceptable time limits

### Test Scenarios
- Valid validator queries with expected data
- Invalid validator queries with graceful error handling
- Epoch boundary conditions and transitions
- Parameter validation with edge cases
- Invariant violation detection and reporting
- API version compatibility checks

## Compliance and Auditing

### Requirements Coverage
This API contract fulfills the following requirements:

- **12.1**: All runtime API method signatures, argument types, and return shapes are frozen
- **12.2**: API versioning constant and breaking change event emission are implemented
- **12.3**: Comprehensive API contract document with example payloads and error cases
- **12.4**: Integration tests for all runtime API methods with various input scenarios

### Audit Trail
All API interactions can be monitored through:
- Event emission for state changes
- Parameter change tracking
- Invariant violation reporting
- Version change notifications

## Support and Maintenance

### Documentation Updates
This contract document will be updated for:
- New API methods (non-breaking)
- Parameter additions (non-breaking)
- Version increments (breaking changes)
- Usage examples and best practices

### Backward Compatibility
Within the same API version:
- Existing methods will continue to work as documented
- Method signatures will not change
- Return types will remain stable
- Error handling behavior will be consistent

### Migration Guide
When API versions change, migration guides will be provided covering:
- Breaking changes and their impact
- Code update requirements
- Testing and validation procedures
- Timeline for deprecation of old versions