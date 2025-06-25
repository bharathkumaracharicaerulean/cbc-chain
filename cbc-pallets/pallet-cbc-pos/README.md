# CBC PoS Pallet

The CBC PoS (Proof of Stake) pallet implements the PoS consensus mechanism for the CBC blockchain. It manages validator stakes, scores, and slashing mechanisms.

## Key Features

1. **Validator Management**
   - Registration and deregistration
   - Maximum validator limit
   - Score tracking
   - Stake management

2. **Score System**
   - Score submission and validation
   - Score decay per epoch
   - Score boosting and slashing
   - Minimum score requirements

3. **Stake Management**
   - Bonding and unbonding
   - Minimum stake requirements
   - Stake validation
   - Stake slashing

4. **Slashing Mechanism**
   - Validator slashing
   - Maximum slashing count
   - Automatic removal
   - Slashing penalties

5. **Runtime API**
   - Validator queries
   - Score queries
   - Stake queries
   - Slashing queries

## Configuration Parameters

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;

    // Minimum score required for a validator to be considered active
    type MinValidatorScore: Get<u32>;
    // Minimum number of active validators required
    type MinActiveValidators: Get<u32>;
    // Maximum number of validators allowed
    type MaxValidators: Get<u32>;
    // Score decay per epoch
    type ValidatorScoreDecay: Get<u32>;
    // Maximum slashing count before removal
    type MaxSlashingCount: Get<u32>;
    // Minimum stake amount required for validators
    type MinStake: Get<BalanceOf<Self>>;
    // The balance type
    type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
}
```

## Storage Items

### Validators
```rust
#[pallet::storage]
pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;
```

### Validator Scores
```rust
#[pallet::storage]
pub type ValidatorScores<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;
```

### Current Epoch
```rust
#[pallet::storage]
pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;
```

### Slashing Count
```rust
#[pallet::storage]
pub type SlashingCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;
```

### Stake
```rust
#[pallet::storage]
pub type Stake<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;
```

## Events

```rust
#[pallet::event]
pub enum Event<T: Config> {
    ValidatorRegistered { validator: T::AccountId },
    ScoreSubmitted { validator: T::AccountId, score: u32 },
    ValidatorSlashed { validator: T::AccountId, slashing_count: u32 },
    ValidatorRemoved { validator: T::AccountId, reason: Vec<u8> },
    StakeBonded { validator: T::AccountId, amount: BalanceOf<T> },
    StakeUnbonded { validator: T::AccountId, amount: BalanceOf<T> },
}
```

## Errors

```rust
#[pallet::error]
pub enum Error<T> {
    ValidatorAlreadyRegistered,
    ValidatorNotRegistered,
    InvalidScore,
    TooManyValidators,
    ScoreTooLow,
    MaxSlashingCountReached,
    InsufficientStake,
    InvalidStakeAmount,
}
```

## Dispatchable Functions

### register_validator
```rust
pub fn register_validator(origin: OriginFor<T>) -> DispatchResult
```
- Registers a new validator
- Requires minimum stake
- Enforces maximum validator limit
- Emits `ValidatorRegistered` event

### submit_score
```rust
pub fn submit_score(origin: OriginFor<T>, validator: T::AccountId, score: u32) -> DispatchResult
```
- Submits validator score
- Validates minimum score
- Updates validator score
- Emits `ScoreSubmitted` event

### slash_validator
```rust
pub fn slash_validator(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult
```
- Slashes validator
- Increments slashing count
- Removes validator if max count reached
- Emits `ValidatorSlashed` or `ValidatorRemoved` event

### bond_stake
```rust
pub fn bond_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult
```
- Bonds stake to validator
- Validates minimum stake
- Updates stake amount
- Emits `StakeBonded` event

### unbond_stake
```rust
pub fn unbond_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult
```
- Unbonds stake from validator
- Validates stake amount
- Updates stake amount
- Emits `StakeUnbonded` event

### boost_score
```rust
pub fn boost_score(origin: OriginFor<T>, validator: T::AccountId, weight: u32) -> DispatchResult
```
- Boosts validator score
- Validates validator existence
- Updates score
- Emits `ScoreSubmitted` event

### slash_score
```rust
pub fn slash_score(origin: OriginFor<T>, validator: T::AccountId, weight: u32) -> DispatchResult
```
- Slashes validator score
- Validates validator existence
- Updates score
- Emits `ScoreSubmitted` event

## Runtime API

```rust
#[decl_runtime_apis]
pub trait PosApi<AccountId, Balance> {
    fn get_validator_stake(validator: AccountId) -> Balance;
    fn get_validator_score(validator: AccountId) -> u32;
    fn get_active_validators() -> Vec<AccountId>;
    fn get_slashing_count(validator: AccountId) -> u32;
}
```

## Genesis Configuration

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub validators: Vec<T::AccountId>,
    pub validator_scores: Vec<u32>,
    pub current_epoch: u32,
    pub slashing_count: Vec<(T::AccountId, u32)>,
}
```

## Usage Example

```rust
// Register a validator
assert_ok!(PalletCbcPos::register_validator(Origin::signed(validator_account)));

// Bond stake
assert_ok!(PalletCbcPos::bond_stake(Origin::signed(validator_account), 1000));

// Submit score
assert_ok!(PalletCbcPos::submit_score(Origin::signed(validator_account), validator_account, 100));

// Get validator stake
let stake = PalletCbcPos::get_validator_stake(validator_account);
```

## Security Considerations

1. **Validator Registration**
   - Limited by maximum validators (`MaxValidators`)
   - Requires minimum stake (`MinStake`)
   - Enforces minimum score (`MinValidatorScore`)

2. **Score Management**
   - Score decay per epoch
   - Minimum score requirement
   - Score boosting and slashing

3. **Slashing**
   - Maximum slashing count
   - Automatic removal after max slashing
   - Stake slashing

4. **Stake Management**
   - Minimum stake requirement
   - Bonding/unbonding validation
   - Stake slashing on misbehavior

## Performance Optimization

1. **Storage Efficiency**
   - Compact storage types
   - Efficient indexing
   - Value queries for default values

2. **Weight Management**
   - Weight benchmarks
   - Weight tracking
   - Weight optimization

3. **Event Logging**
   - Event deposit optimization
   - Event filtering
   - Event batching

## Testing

The pallet includes comprehensive tests in the `mock` and `tests` modules, covering:

- Validator registration
- Score submission
- Slashing
- Stake management
- Runtime API
- Genesis configuration

## Benchmarking

The pallet includes benchmarking support for:

- Weight calculation
- Storage benchmarks
- Runtime API benchmarks
- Event benchmarks

## Integration

The PoS pallet integrates with:

1. **Runtime**
   - FRAME system
   - Event system
   - Storage system
   - Weight system

2. **Other Pallets**
   - PoI (Proof of Inference)
   - DCF (Dynamic Consensus Framework)
   - Balances

3. **Runtime APIs**
   - Validator queries
   - Score queries
   - Stake queries
   - Slashing queries

## Future Enhancements

1. **Validator Set Management**
   - Dynamic validator set
   - Validator rotation
   - Validator selection

2. **Score System**
   - Advanced score algorithms
   - Score decay optimization
   - Score normalization

3. **Stake Management**
   - Stake delegation
   - Stake unbonding periods
   - Stake rewards

4. **Slashing**
   - Progressive slashing
   - Slashing penalties
   - Slashing rewards



