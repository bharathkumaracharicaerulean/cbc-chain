# pallet-cbc-poi

A Substrate pallet implementing Proof-of-Inference (PoI) consensus for the CBC-Chain, enabling decentralized validation of AI/ML inference results through a challenge-based mechanism.

## Features

### Core Functionality
- **Inference Submission**: Validators submit results with confidence scores
- **Challenge Mechanism**: Dispute resolution for inference results
- **Epoch-based Tracking**: Results and challenges tracked per epoch
- **PoS Integration**: Seamless integration with Proof of Stake system

### Key Components
- **Inference Management**: Store and validate inference submissions
- **Challenge System**: Handle disputes between validators
- **Epoch Control**: Manage epoch transitions and validity windows
- **Reward Distribution**: Incentivize accurate reporting and challenging

## Storage

### Primary Storage
- `InferenceResults`: Maps validators to their latest (result, epoch) tuple
- `Challenges`: Tracks active challenges as (challenged_account, result, epoch)
- `CurrentEpoch`: Current inference epoch counter

## Configuration

### Runtime Configuration
- `MinInferenceConfidence`: Minimum confidence threshold (0-100)
- `MaxInferenceAge`: Maximum age of inferences (in epochs)
- `ChallengeWindow`: Duration for challenging results (in epochs)
- `InferenceReward`: Reward for correct inferences
- `ChallengeReward`: Reward for successful challenges
- `PosInterface`: Hook for PoS integration (score adjustments)

## Extrinsics

### `submit_inference`
- **Purpose**: Submit an inference result with confidence score
- **Parameters**:
  - `result`: The inference result (u32)
  - `confidence`: Confidence score (0-100)
- **Events**: `InferenceSubmitted`, `InferenceAccepted`
- **Errors**: `InferenceAlreadySubmitted`, `ConfidenceTooLow`

### `challenge_inference`
- **Purpose**: Challenge another validator's inference
- **Parameters**:
  - `challenged`: Account being challenged
  - `result`: The disputed result
- **Events**: `InferenceChallenged`, `ChallengeResolved`, `ValidatorSlashed`
- **Errors**: `InferenceNotFound`, `InvalidChallenge`, `ChallengeWindowExpired`, `InferenceTooOld`

## Runtime API

### `get_inference_result(account)`
- Returns: `Option<(u32, u32)>` - (result, epoch) if exists

### `get_challenge(account)`
- Returns: `Option<(AccountId, u32, u32)>` - (challenged, result, epoch) if challenged

### `get_current_epoch()`
- Returns: `u32` - Current epoch number

## Events

### Inference Events
- `InferenceSubmitted { who, result, confidence }`
- `InferenceAccepted { validator, confidence }`
- `InferenceRejected { validator, confidence }`

### Challenge Events
- `InferenceChallenged { challenger, challenged, result }`
- `ChallengeResolved { challenger, challenged, result, success }`
- `ValidatorSlashed { validator, reason }`

## Error Handling

### Validation Errors
- `InferenceAlreadySubmitted`: Validator already submitted this epoch
- `InferenceNotFound`: No inference to challenge
- `InvalidChallenge`: Challenge doesn't match stored result
- `ConfidenceTooLow`: Below minimum threshold
- `ChallengeWindowExpired`: Too late to challenge
- `InferenceTooOld`: Result from previous epoch

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