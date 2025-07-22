# pallet-cbc-dcf

The **Dynamic Consensus Framework (DCF)** pallet manages validators, scoring, and governance for the CBC-Chain using a hybrid PoS/PoI system.

## Key Features

### Validator Management
- Dynamic validator set with opt-in/out functionality
- Dual scoring: Combines PoS (stake) and PoI (inference)
- Comprehensive state tracking and statistics
- Score adjustments with decay and boost mechanisms

### Epoch System
- Configurable epoch duration
- Automatic validator set rotation
- Historical data tracking (24 epochs)
- Block production monitoring

### Governance
- On-chain proposal system
- Stake-weighted voting
- Emergency governance mode
- Validator management actions
- **Slashing**: Penalize malicious behavior
- **Ejection**: Remove underperforming validators
- **Score Management**: Decay for inactivity, boosts for good performance
- **Missed Block Handling**: Tracks and penalizes missed blocks

## Core Functionality

### Validator Operations
- Join/leave validator set
- Update stake (PoS) and inference (PoI) scores
- Track performance and participation

### Governance
- Submit and vote on proposals
- Execute approved actions
- Emergency governance controls
- Validator management (slash/reward/eject)

### Runtime API
- Validator scores and states
- Epoch information and configuration
- Governance status
- Historical data access

## Integration

The DCF pallet integrates with other CBC-Chain components:
- `pallet-cbc-pos`: Proof of Stake functionality
- `pallet-cbc-poi`: Proof of Inference scoring
- `frame-system`: Core blockchain functionality

## Development

### Building

```bash
# Build with all features
cargo build --release --features runtime-benchmarks
```

### Testing

Run the test suite:
```bash
cargo test
```

### Benchmarking

To generate weights and benchmarks:
```bash
cargo test --features runtime-benchmarks --bench benchmarking
```

## Storage

### Key Storage
- Validator states and statistics
- Active validator set
- Epoch configuration and history
- Governance proposals and votes
- Pending validator actions

## Configuration

### Main Parameters
- Validator limits and thresholds
- Score calculation weights
- Governance controls
- Epoch settings
- Block production parameters

## Security Considerations

- **Validator Rotation**: The active validator set can change at epoch boundaries
- **Score Manipulation**: The score decay mechanism prevents score inflation
- **Governance Attacks**: The voting mechanism is protected by stake weighting
- **Front-running**: Critical operations are protected against front-running

## Future Improvements

- On-chain parameter adjustment through governance
- More sophisticated score calculation algorithms
- Enhanced validator set selection mechanisms
- Cross-chain governance capabilities
   - Governance mode

## Configuration Parameters

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pos::Config + poi::Config + TypeInfo + fmt::Debug {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    #[pallet::constant]
    type MaxValidators: Get<u32>;
    type EpochConfig: Get<EpochConfig>;
    type ScoreBoostReason: Get<ScoreBoostReason>;
    type EjectionReason: Get<EjectionReason>;
    type InferenceErrorSeverity: Get<u32>;
    type ScoreDecayRate: Get<u32>;
    type MinValidatorScore: Get<u64>;
}
```

## Storage Items

### ValidatorStates
```rust
#[pallet::storage]
pub type ValidatorStates<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorState, OptionQuery>;
```

### CurrentEpoch
```rust
#[pallet::storage]
pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;
```

### EpochHistory
```rust
#[pallet::storage]
pub type EpochHistory<T: Config> = StorageValue<_, BoundedVec<EpochHistory<T>, ConstU32<24>>, ValueQuery>;
```

## Events

### Validator Events
- Score updates and adjustments
- Epoch transitions
- Validator status changes (ejected/re-entered)
- Invalid block detection

### Governance Events
- Proposal submission and voting
- Governance mode changes
- Proposal execution results

## Error Cases

### Validation Errors
- Validator not found
- Invalid configurations
- Insufficient validators

### Governance Errors
- Unauthorized actions
- Invalid proposals
- Voting violations

### System Errors
- Epoch transition failures
- State inconsistencies
- Invalid operations

## Dispatchable Functions

### Validator Management
```rust
pub fn join_validator_set(origin: OriginFor<T>) -> DispatchResult
{{ ... }}
pub fn leave_validator_set(origin: OriginFor<T>) -> DispatchResult
```

### Score Management
```rust
pub fn update_validator_stake_score(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult
pub fn update_validator_inference_score(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult
pub fn update_consensus_weights(origin: OriginFor<T>, pos_weight: u64, poi_weight: u64) -> DispatchResult
```

### Governance
```rust
pub fn submit_proposal(origin: OriginFor<T>, action: ProposalAction<T>) -> DispatchResult
pub fn vote_proposal(origin: OriginFor<T>, proposal_id: u32, approve: bool) -> DispatchResult
pub fn execute_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult
```

### Sudo Operations
```rust
pub fn set_governance_mode(origin: OriginFor<T>, enabled: bool) -> DispatchResult
pub fn sudo_advance_epoch(origin: OriginFor<T>) -> DispatchResult

// Sudo proposal functions
pub fn propose_slash_validator(
    origin: OriginFor<T>,
    proposer: T::AccountId,
    validator: T::AccountId,
    amount: <T as pallet::Config>::Balance,
) -> DispatchResult

pub fn propose_reward_validator(
    origin: OriginFor<T>,
    proposer: T::AccountId,
    validator: T::AccountId,
    amount: <T as pallet::Config>::Balance,
) -> DispatchResult

pub fn propose_eject_validator(
    origin: OriginFor<T>,
    proposer: T::AccountId,
    validator: T::AccountId,
    reason: EjectionReason,
) -> DispatchResult
```

## Runtime API

```rust
#[decl_runtime_apis]
pub trait DcfApi<AccountId> {
    fn get_validator_scores() -> Vec<(AccountId, u64)>;
    fn get_current_epoch() -> u32;
    fn get_validator_stake_score(validator: AccountId) -> u64;
    fn get_validator_inference_score(validator: AccountId) -> u64;
    fn get_consensus_weights() -> (u64, u64);
    fn is_validator_active(validator: AccountId) -> bool;
    fn get_expected_author(block_number: u32) -> Option<AccountId>;
    fn get_validator_score_history(validator: AccountId) -> Vec<u64>;
    fn get_validator_participation(validator: AccountId) -> (u32, u32);
    fn get_active_validators() -> Vec<AccountId>;
    fn get_validator_last_active(validator: AccountId) -> u32;
    fn validate_block_author(block_number: u32, author: AccountId);
    fn get_validator_profile(account_id: AccountId) -> Option<(u64, u32, u32, u32, u32)>;
    fn get_inference_result(account_id: AccountId) -> Option<u64>;
    fn get_epoch_history(epoch_number: u32) -> Option<RuntimeEpochHistory<AccountId>>;
    fn get_recent_epochs(n: u32) -> Vec<RuntimeEpochHistory<AccountId>>;
}
```

## Genesis Configuration

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub initial_validators: Vec<T::AccountId>,
    pub initial_scores: Vec<(T::AccountId, u64)>,
    pub initial_epoch_config: EpochConfig,
    pub initial_consensus_weights: (u64, u64),
    pub initial_governance_mode: bool,
}
```

## Benchmarking

The pallet includes benchmarking support for:

- Weight calculation
- Storage benchmarks
- Runtime API benchmarks
- Event benchmarks

## Integration

The DCF pallet integrates with:

1. **Runtime**
   - FRAME system
   - Event system
   - Storage system
   - Weight system

2. **Other Pallets**
   - PoS (via pos::Config)
   - PoI (via poi::Config)
   - Balances
   - Timestamp

3. **Runtime APIs**
   - Validator queries
   - Score queries
   - Epoch queries
   - Governance queries

## Usage Example

```rust
// Join validator set
assert_ok!(PalletCbcDcf::join_validator_set(Origin::signed(validator_account)));

// Submit governance proposal
assert_ok!(PalletCbcDcf::submit_proposal(
    Origin::signed(proposer_account),
    ProposalAction::Reward {
        validator: validator_account,
        amount: reward_amount
    }
));

// Vote on proposal
assert_ok!(PalletCbcDcf::vote_proposal(
    Origin::signed(voter_account),
    proposal_id,
    true // approve
));

// Get validator score
let score = PalletCbcDcf::get_validator_stake_score(validator_account);
```

## Testing

The pallet includes comprehensive tests in the `tests` module, covering:

- Validator management
- Score calculations
- Governance system
- Epoch transitions
- Runtime API
- Genesis configuration

Run unit tests with:

```sh
cargo test -p pallet-cbc-dcf
```

## References

- [Architecture documentation](../../docs/dcf-architecture.md)
- [PoS Pallet](../pallet-cbc-pos/README.md)
- [PoI Pallet](../pallet-cbc-poi/README.md)
- [Runtime Overview](../../docs/runtime-overview.md)
-
# CBC DCF Pallet

The CBC DCF (Dynamic Consensus Framework) pallet implements the core consensus mechanism for CBC-Chain, combining Proof of Stake (PoS) and Proof of Inference (PoI) to create a dynamic validator selection and scoring system. It provides advanced features for validator management, score calculation, and on-chain governance.

## Key Features

1. **Dynamic Validator Management**
   - Validator joining/leaving system
   - Score-based validator selection
   - Epoch-based validator rotation
   - Automatic score adjustments

2. **Score Calculation**
   - Combined PoS and PoI scoring
   - Configurable weight distribution
   - Score decay mechanism
   - Block authorship tracking

3. **On-Chain Governance**
   - Proposal submission system
   - Voting mechanism
   - Proposal execution
   - Sudo controls

4. **Epoch Management**
   - Automatic epoch transitions
   - Validator set updates
   - Score snapshots
   - Historical tracking

5. **Security Features**
   - Validator slashing
   - Score penalties
   - Ejection mechanism
   - Governance mode

## Configuration Parameters

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pos::Config + poi::Config + TypeInfo + fmt::Debug {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    #[pallet::constant]
    type MaxValidators: Get<u32>;
    type EpochConfig: Get<EpochConfig>;
    type ScoreBoostReason: Get<ScoreBoostReason>;
    type EjectionReason: Get<EjectionReason>;
    type InferenceErrorSeverity: Get<u32>;
    type ScoreDecayRate: Get<u32>;
    type MinValidatorScore: Get<u64>;
}
```

## Storage Items

### ValidatorStates
```rust
#[pallet::storage]
pub type ValidatorStates<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorState, OptionQuery>;
```

### CurrentEpoch
```rust
#[pallet::storage]
pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;
```

### EpochHistory
```rust
#[pallet::storage]
pub type EpochHistory<T: Config> = StorageValue<_, BoundedVec<EpochHistory<T>, ConstU32<24>>, ValueQuery>;
```

## Events

```rust
#[pallet::event]
pub enum Event<T: Config> {
    ValidatorScoreUpdated { validator: T::AccountId, score: u64 },
    ValidatorJoined { validator: T::AccountId },
    ValidatorLeft { validator: T::AccountId },
    ProposalSubmitted { proposal_id: u32, proposer: T::AccountId },
    ProposalVoted { proposal_id: u32, voter: T::AccountId, approve: bool },
    ProposalExecuted { proposal_id: u32 },
    ValidatorSlashed { validator: T::AccountId, amount: BalanceOf<T> },
    ValidatorRewarded { validator: T::AccountId, amount: BalanceOf<T> },
    ValidatorEjected { validator: T::AccountId, reason: EjectionReason },
    ScoreBoosted { validator: T::AccountId, amount: u64, reason: ScoreBoostReason },
    ScoreDecayed { validator: T::AccountId, amount: u64 },
}
```

## Errors

```rust
#[pallet::error]
pub enum Error<T> {
    ValidatorNotFound,
    InvalidWeight,
    InvalidEpochConfig,
    NotEnoughValidators,
    ProposalNotFound,
    InvalidVote,
    NotAuthorized,
    GovernanceModeDisabled,
    EpochTransitionFailed,
    ScoreTooLow,
    ValidatorInactive,
    InvalidScoreBoost,
    InvalidEjectionReason,
    InvalidInferenceError,
}
```

## Dispatchable Functions

### Validator Management
```rust
pub fn join_validator_set(origin: OriginFor<T>) -> DispatchResult
pub fn leave_validator_set(origin: OriginFor<T>) -> DispatchResult
```

### Score Management
```rust
pub fn update_validator_stake_score(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult
pub fn update_validator_inference_score(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult
pub fn update_consensus_weights(origin: OriginFor<T>, pos_weight: u64, poi_weight: u64) -> DispatchResult
```

### Governance
```rust
pub fn submit_proposal(origin: OriginFor<T>, action: ProposalAction<T>) -> DispatchResult
pub fn vote_proposal(origin: OriginFor<T>, proposal_id: u32, approve: bool) -> DispatchResult
pub fn execute_proposal(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult
```

### Sudo Operations
```rust
pub fn set_governance_mode(origin: OriginFor<T>, enabled: bool) -> DispatchResult
pub fn sudo_advance_epoch(origin: OriginFor<T>) -> DispatchResult
```

## Runtime API

```rust
#[decl_runtime_apis]
pub trait DcfApi<AccountId> {
    fn get_validator_scores() -> Vec<(AccountId, u64)>;
    fn get_current_epoch() -> u32;
    fn get_validator_stake_score(validator: AccountId) -> u64;
    fn get_validator_inference_score(validator: AccountId) -> u64;
    fn get_consensus_weights() -> (u64, u64);
    fn is_validator_active(validator: AccountId) -> bool;
    fn get_expected_author(block_number: u32) -> Option<AccountId>;
    fn get_validator_score_history(validator: AccountId) -> Vec<u64>;
    fn get_validator_participation(validator: AccountId) -> (u32, u32);
    fn get_active_validators() -> Vec<AccountId>;
    fn get_validator_last_active(validator: AccountId) -> u32;
    fn validate_block_author(block_number: u32, author: AccountId);
    fn get_validator_profile(account_id: AccountId) -> Option<(u64, u32, u32, u32, u32)>;
    fn get_inference_result(account_id: AccountId) -> Option<u64>;
    fn get_epoch_history(epoch_number: u32) -> Option<RuntimeEpochHistory<AccountId>>;
    fn get_recent_epochs(n: u32) -> Vec<RuntimeEpochHistory<AccountId>>;
}
```

## Genesis Configuration

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub initial_validators: Vec<T::AccountId>,
    pub initial_scores: Vec<(T::AccountId, u64)>,
    pub initial_epoch_config: EpochConfig,
    pub initial_consensus_weights: (u64, u64),
    pub initial_governance_mode: bool,
}
```

## Usage Example

```rust
// Join validator set
assert_ok!(PalletCbcDcf::join_validator_set(Origin::signed(validator_account)));

// Submit governance proposal
assert_ok!(PalletCbcDcf::submit_proposal(
    Origin::signed(proposer_account),
    ProposalAction::Reward {
        validator: validator_account,
        amount: reward_amount
    }
));

// Vote on proposal
assert_ok!(PalletCbcDcf::vote_proposal(
    Origin::signed(voter_account),
    proposal_id,
    true // approve
));

// Get validator score
let score = PalletCbcDcf::get_validator_stake_score(validator_account);
```

## Security Considerations

1. **Validator Management**
   - Score thresholds for joining
   - Maximum validator limit
   - Ejection mechanism

2. **Score System**
   - Score decay prevention
   - Minimum score requirements
   - Score normalization

3. **Governance**
   - Proposal quorum requirements
   - Voting period enforcement
   - Execution validation

4. **Epoch Management**
   - Validator rotation security
   - Score snapshot integrity
   - Historical tracking

## Performance Optimization

1. **Storage Efficiency**
   - BoundedVec usage
   - Compact storage types
   - Efficient indexing

2. **Weight Management**
   - Weight benchmarks
   - Weight tracking
   - Optimization

3. **Event Logging**
   - Event optimization
   - Event filtering
   - Event batching

## Testing

The pallet includes comprehensive tests in the `tests` module, covering:

- Validator management
- Score calculations
- Governance system
- Epoch transitions
- Runtime API
- Genesis configuration

## Benchmarking

The pallet includes benchmarking support for:

- Weight calculation
- Storage benchmarks
- Runtime API benchmarks
- Event benchmarks

## Integration

The DCF pallet integrates with:

1. **Runtime**
   - FRAME system
   - Event system
   - Storage system
   - Weight system

2. **Other Pallets**
   - PoS (via pos::Config)
   - PoI (via poi::Config)
   - Balances
   - Timestamp

3. **Runtime APIs**
   - Validator queries
   - Score queries
   - Epoch queries
   - Governance queries

## Future Enhancements

1. **Score System**
   - Advanced scoring algorithms
   - Dynamic weight adjustments
   - Score normalization

2. **Governance**
   - Progressive proposals
   - Voting optimizations
   - Proposal categories

3. **Epoch Management**
   - Adaptive epoch length
   - Dynamic validator selection
   - Score decay optimization

4. **Security**
   - Advanced slashing conditions
   - Score manipulation prevention
   - Validator rotation security
