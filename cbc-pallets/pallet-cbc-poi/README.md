# Pallet CBC-PoI (Proof of Inference)

The `pallet-cbc-poi` implements a **Proof of Inference (PoI)** mechanism for the CBC blockchain. It allows validators to submit inference results, challenge incorrect results, and incentivizes accurate submissions while penalizing invalid ones.

---

## Features

- **Inference Submission**: Validators can submit inference results with confidence scores.
- **Challenges**: Validators can challenge inference results submitted by others.
- **Epoch Management**: Tracks the current epoch to ensure inference results and challenges are valid within the allowed time frame.
- **Rewards**: Validators are rewarded for valid inference submissions, and successful challengers are rewarded for identifying invalid results.

---

## Storage

### **InferenceResults**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, (u32, u32), OptionQuery>`
- **Description**: Stores inference results submitted by validators, along with the epoch in which they were submitted.

### **Challenges**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, u32, u32), OptionQuery>`
- **Description**: Tracks challenges raised against inference results, including the challenger, challenged account, and the result.

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