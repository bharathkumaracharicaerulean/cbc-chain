# CBC PoI Pallet

The CBC PoI (Proof of Inference) pallet implements the PoI consensus mechanism for the CBC blockchain. It allows validators to submit inference results, challenge others' results, and provides a mechanism for rewarding or penalizing based on inference correctness.

## Key Features

1. **Inference Submission**
   - Submit inference results with confidence scores
   - Minimum confidence threshold enforcement
   - Epoch-based result tracking
   - Rewards for valid submissions

2. **Challenge Mechanism**
   - Challenge other validators' results
   - Configurable challenge window
   - Challenge resolution system
   - Rewards for successful challenges

3. **Epoch Management**
   - Track current inference epoch
   - Result age validation
   - Challenge window enforcement
   - Epoch-based updates

4. **Score Integration**
   - Score boosting for correct results
   - Score slashing for invalid results
   - Integration with PoS system
   - Dynamic score adjustments

5. **Reward System**
   - Inference rewards
   - Challenge rewards
   - Penalty system
   - Score adjustments

## Configuration Parameters

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;

    // Minimum confidence threshold for inference (0-100)
    type MinInferenceConfidence: Get<u32>;
    // Maximum age of inference in epochs
    type MaxInferenceAge: Get<u32>;
    // Number of epochs to challenge an inference
    type ChallengeWindow: Get<u32>;
    // Reward for correct inference
    type InferenceReward: Get<u128>;
    // Reward for successful challenge
    type ChallengeReward: Get<u128>;
    // Interface to PoS pallet for boosting/slashing scores
    type PosInterface: PosInterface<Self::AccountId>;
}
```

## Storage Items

### InferenceResults
```rust
#[pallet::storage]
pub type InferenceResults<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (u32, u32), OptionQuery>;
```

### Challenges
```rust
#[pallet::storage]
pub type Challenges<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, u32, u32), OptionQuery>;
```

### Current Epoch
```rust
#[pallet::storage]
pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;
```

## Events

```rust
#[pallet::event]
pub enum Event<T: Config> {
    InferenceSubmitted { who: T::AccountId, result: u32, confidence: u32 },
    InferenceAccepted { validator: T::AccountId, confidence: u32 },
    InferenceRejected { validator: T::AccountId, confidence: u32 },
    InferenceChallenged { challenger: T::AccountId, challenged: T::AccountId, result: u32 },
    ChallengeResolved { challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool },
    ValidatorSlashed { validator: T::AccountId, reason: Vec<u8> },
}
```

## Errors

```rust
#[pallet::error]
pub enum Error<T> {
    InferenceAlreadySubmitted,
    InferenceNotFound,
    InvalidChallenge,
    ConfidenceTooLow,
    ChallengeWindowExpired,
    InferenceTooOld,
}
```

## Dispatchable Functions

### submit_inference
```rust
pub fn submit_inference(
    origin: OriginFor<T>,
    result: u32,
    confidence: u32,
) -> DispatchResult
```
- Submit inference result with confidence
- Validates minimum confidence
- Updates inference results
- Emits `InferenceSubmitted` event

### challenge_inference
```rust
pub fn challenge_inference(
    origin: OriginFor<T>,
    challenged: T::AccountId,
    result: u32,
) -> DispatchResult
```
- Challenge another validator's inference
- Validates challenge window
- Updates challenges
- Emits `InferenceChallenged` event

## Runtime API

```rust
#[decl_runtime_apis]
pub trait PoiApi<AccountId> {
    fn get_inference_result(validator: AccountId) -> Option<(u32, u32)>;
    fn get_challenge(validator: AccountId) -> Option<(AccountId, u32, u32)>;
    fn get_current_epoch() -> u32;
}
```

## Genesis Configuration

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
    pub inference_results: Vec<(T::AccountId, u32)>,
    pub challenges: Vec<(T::AccountId, T::AccountId, u32)>,
    pub current_epoch: u32,
}
```

## Usage Example

```rust
// Submit inference
assert_ok!(PalletCbcPoi::submit_inference(
    Origin::signed(validator_account),
    85, // inference result
    95, // confidence score
));

// Challenge inference
assert_ok!(PalletCbcPoi::challenge_inference(
    Origin::signed(challenger_account),
    validator_account,
    85 // challenged result
));

// Get inference result
let result = PalletCbcPoi::get_inference_result(validator_account);
```

## Security Considerations

1. **Inference Submission**
   - Minimum confidence requirement
   - Epoch validation
   - Duplicate prevention

2. **Challenge System**
   - Challenge window enforcement
   - Result age validation
   - Validity checks

3. **Score Management**
   - Score boosting for correct results
   - Score slashing for invalid results
   - Integration with PoS

4. **Reward System**
   - Reward validation
   - Penalty enforcement
   - Score adjustments

## Performance Optimization

1. **Storage Efficiency**
   - Compact storage types
   - Efficient indexing
   - Value queries

2. **Weight Management**
   - Weight benchmarks
   - Weight tracking
   - Optimization

3. **Event Logging**
   - Event optimization
   - Event filtering
   - Event batching

## Testing

The pallet includes comprehensive tests in the `mock` and `tests` modules, covering:

- Inference submission
- Challenge mechanism
- Epoch management
- Reward system
- Genesis configuration

## Benchmarking

The pallet includes benchmarking support for:

- Weight calculation
- Storage benchmarks
- Runtime API benchmarks
- Event benchmarks

## Integration

The PoI pallet integrates with:

1. **Runtime**
   - FRAME system
   - Event system
   - Storage system
   - Weight system

2. **Other Pallets**
   - PoS (via PosInterface)
   - DCF (Dynamic Consensus Framework)
   - Balances

3. **Runtime APIs**
   - Inference queries
   - Challenge queries
   - Epoch queries

## Future Enhancements

1. **Inference System**
   - Advanced inference algorithms
   - Result validation
   - Confidence scoring

2. **Challenge System**
   - Progressive challenges
   - Challenge rewards
   - Challenge penalties

3. **Score Management**
   - Advanced score algorithms
   - Score normalization
   - Score decay

4. **Reward System**
   - Dynamic rewards
   - Reward optimization
   - Penalty system


### **CurrentEpoch**
- **Type**: `StorageValue<_, u32, ValueQuery>`
- **Description**: Tracks the current epoch or round.

---

## Events

### **InferenceSubmitted**
- **Fields**: `{ who: T::AccountId, result: u32, confidence: u32 }`
- **Description**: Emitted when a validator submits an inference result.

### **InferenceChallenged**
- **Fields**: `{ challenger: T::AccountId, challenged: T::AccountId, result: u32 }`
- **Description**: Emitted when a validator challenges an inference result.

### **ChallengeResolved**
- **Fields**: `{ challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool }`
- **Description**: Emitted when a challenge is resolved, indicating whether it was successful or not.

---

## Errors

- **`InferenceAlreadySubmitted`**: The inference result has already been submitted by the validator.
- **`InferenceNotFound`**: The inference result being challenged does not exist.
- **`ConfidenceTooLow`**: The confidence level of the submitted inference is below the minimum threshold.
- **`ChallengeWindowExpired`**: The challenge was submitted after the allowed challenge window.
- **`InferenceTooOld`**: The inference result is too old to be challenged.
- **`InvalidChallenge`**: The challenge is invalid because the result does not match the stored inference.

---

## Extrinsics

### **submit_inference**
- **Description**: Allows a validator to submit an inference result with a confidence score.
- **Parameters**:
  - `result`: The inference result.
  - `confidence`: The confidence score (0-100).
- **Weight**: `10_000`

### **challenge_inference**
- **Description**: Allows a validator to challenge an inference result submitted by another validator.
- **Parameters**:
  - `challenged`: The account ID of the validator whose inference result is being challenged.
  - `result`: The result being challenged.
- **Weight**: `20_000`

---

## Testing

The `tests.rs` file includes unit tests to verify the functionality of the pallet. Key tests include:
- **Inference Submission**:
  - Valid submissions.
  - Rejection of duplicate submissions.
  - Rejection of submissions with low confidence.
- **Challenges**:
  - Valid challenges.
  - Rejection of challenges against non-existent or invalid results.
  - Rejection of challenges submitted after the challenge window.

Run the tests using:

```bash
#To run the build files use the below command
cargo build -p pallet-cbc-poi

#To perform unit testing on pallet run below command
cargo test -p pallet-cbc-poi

#TO perform benchmarking test on the pallet use this command
cargo bench -p pallet-cbc-poi