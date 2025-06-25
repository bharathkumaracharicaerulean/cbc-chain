# CBC-PoI Pallet Documentation

The `cbc-poi` pallet implements a **Proof of Inference (PoI)** mechanism for the CBC blockchain. It allows validators to submit inference results, challenge incorrect results, and incentivizes accurate submissions while penalizing invalid ones.

---

## 1. Purpose

The `cbc-poi` pallet is responsible for:
- Allowing validators to submit inference results with confidence scores.
- Enabling validators to challenge inference results submitted by others.
- Managing rewards for valid inference submissions and successful challenges.
- Ensuring inference results and challenges are valid within the allowed time frame.
- Integrating with PoS system for score boosting and slashing.

---

## 2. Key Features

### **Inference Submission**
- Validators can submit inference results along with a confidence score.
- Results are stored with the current epoch for validation.

### **Challenges**
- Validators can challenge inference results submitted by others.
- Challenges must be submitted within a specified challenge window.

### **Epoch Management**
- Tracks the current epoch to ensure inference results and challenges are valid within the allowed time frame.

### **Rewards**
- Validators are rewarded for submitting valid inference results.
- Successful challengers are rewarded for identifying invalid inference results.

---

## 3. Key Storage

### **InferenceResults**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, (u32, u32), OptionQuery>`
- **Description**: Stores inference results submitted by validators, along with the epoch in which they were submitted.

### **Challenges**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, u32, u32), OptionQuery>`
- **Description**: Tracks challenges raised against inference results, including the challenger, challenged account, and the result.

### **CurrentEpoch**
- **Type**: `StorageValue<_, u32, ValueQuery>`
- **Description**: Tracks the current inference epoch or round.

---

## 4. Key Events

### **InferenceSubmitted**
- **Fields**: `{ who: T::AccountId, result: u32, confidence: u32 }`
- **Description**: Emitted when a validator submits an inference result.

### **InferenceAccepted**
- **Fields**: `{ validator: T::AccountId, confidence: u32 }`
- **Description**: Emitted when an inference result is accepted.

### **InferenceRejected**
- **Fields**: `{ validator: T::AccountId, confidence: u32 }`
- **Description**: Emitted when an inference result is rejected.

### **InferenceChallenged**
- **Fields**: `{ challenger: T::AccountId, challenged: T::AccountId, result: u32 }`
- **Description**: Emitted when a validator challenges an inference result.

### **ChallengeResolved**
- **Fields**: `{ challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool }`
- **Description**: Emitted when a challenge is resolved, indicating whether it was successful or not.

### **ValidatorSlashed**
- **Fields**: `{ validator: T::AccountId, reason: Vec<u8> }`
- **Description**: Emitted when a validator is slashed for submitting invalid inference results.

---

## 5. Key Errors

### **InferenceAlreadySubmitted**
- **Description**: The inference result already exists for this validator.

### **InferenceNotFound**
- **Description**: The inference result does not exist.

### **InvalidChallenge**
- **Description**: The challenge is invalid (e.g., wrong result).

### **ConfidenceTooLow**
- **Description**: The confidence level is too low.

### **ChallengeWindowExpired**
- **Description**: The challenge window has expired.

### **InferenceTooOld**
- **Description**: The inference is too old to be challenged.

---

## 6. Tests

The `tests.rs` file includes unit tests to verify the functionality of the pallet. Key tests include:

### **Inference Submission**
- **Test**: `test_submit_inference_success`
  - Verifies that a validator can successfully submit an inference result.
- **Test**: `test_submit_inference_confidence_too_low`
  - Ensures that inference submissions with confidence below the threshold are rejected.
- **Test**: `test_submit_inference_already_submitted`
  - Ensures that duplicate inference submissions by the same validator are rejected.

### **Challenges**
- **Test**: `test_challenge_inference_success`
  - Verifies that a valid challenge can be submitted successfully.
- **Test**: `test_challenge_inference_not_found`
  - Ensures that challenges against non-existent inference results are rejected.
- **Test**: `test_challenge_inference_invalid_result`
  - Ensures that challenges with incorrect results are rejected.
- **Test**: `test_challenge_inference_too_old`
  - Ensures that challenges against inference results that are too old are rejected.

### **Unauthorized Actions**
- **Test**: `test_unauthorized_submit`
  - Ensures that only signed origins can submit inference results.
- **Test**: `test_unauthorized_challenge`
  - Ensures that only signed origins can submit challenges.

---

## 7. Runtime API

The pallet exposes the following runtime API:

```rust
trait PoiApi<AccountId> {
    fn get_inference_result(validator: AccountId) -> Option<(u32, u32)>;
    fn get_challenge(validator: AccountId) -> Option<(AccountId, u32, u32)>;
    fn get_current_epoch() -> u32;
}
```

## 8. Mock Runtime

The `mock.rs` file defines a mock runtime for testing the pallet. Key configurations include:

### **System Configuration**
- Implements `frame_system::Config` with mock types for accounts, blocks, and events.

### **Pallet Configuration**
- Implements `pallet_cbc_poi::Config` with the following constants:
  - `MinInferenceConfidence`: Minimum confidence threshold for inference (0-100).
  - `MaxInferenceAge`: Maximum age of inference in epochs.
  - `ChallengeWindow`: Number of epochs to challenge an inference.
  - `InferenceReward`: Reward for correct inference.
  - `ChallengeReward`: Reward for successful challenge.
  - `PosInterface`: Interface to PoS pallet for boosting/slashing scores

### **Genesis Storage**
- Initializes the mock runtime with default storage values for inference results, challenges, and the current epoch.

---

## 8. Benchmarking

The `benchmarking.rs` file provides benchmarks for the pallet's extrinsics. Key benchmarks include:

### **Submit Inference**
- **Setup**: Creates a whitelisted caller and sets a valid confidence score.
- **Action**: Submits an inference result.
- **Verification**: Ensures the inference result is stored in the `InferenceResults` storage.

### **Challenge Inference**
- **Setup**: Creates a whitelisted caller and a valid inference result to challenge.
- **Action**: Submits a challenge against the inference result.
- **Verification**: Ensures the challenge is stored in the `Challenges` storage.

---

## 9. Weights

The pallet uses a dedicated weights module (`weights.rs`) that defines weight information for all extrinsics. The weights are automatically benchmarked and include:

- Weight information for `submit_inference` extrinsic
- Weight information for `challenge_inference` extrinsic

The actual weight values are determined through runtime benchmarks and can be found in the weights module.

---

## 10. Conclusion

The `cbc-poi` pallet provides a  Proof of Inference mechanism for managing inference submissions, challenges, and rewards. It includes comprehensive tests, mocks, and benchmarks to ensure reliability and performance. For more details, refer to the source code in the `cbc-pallets/pallet-cbc-poi` directory.