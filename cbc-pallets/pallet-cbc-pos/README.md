# Pallet CBC-PoS (Proof of Stake)

The `pallet-cbc-pos` implements a **Proof of Stake (PoS)** mechanism for the CBC blockchain. It manages validators, their scores, slashing, and epoch-based updates to ensure a secure and fair consensus process.

---

## Features

- **Validator Registration**: Allows accounts to register as validators, provided the maximum validator limit has not been reached.
- **Score Submission**: Validators can submit scores to indicate their performance or stake.
- **Slashing**: Validators can be slashed for misbehavior, and if their slashing count exceeds the maximum allowed, they are removed from the validator set.
- **Epoch Management**: Tracks the current epoch and applies score decay to validators over time.

---

## Storage

### **Validators**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, bool>`
- **Description**: Tracks registered validators and their active status.

### **ValidatorScores**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, u32>`
- **Description**: Stores the scores of validators.

### **CurrentEpoch**
- **Type**: `StorageValue<_, u32, ValueQuery>`
- **Description**: Tracks the current epoch or round.

### **SlashingCount**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, u32>`
- **Description**: Tracks the number of times a validator has been slashed.

---

## Events

### **ValidatorRegistered**
- **Fields**: `{ validator: T::AccountId }`
- **Description**: Emitted when a new validator is registered.

### **ScoreSubmitted**
- **Fields**: `{ validator: T::AccountId, score: u32 }`
- **Description**: Emitted when a validator submits a score.

### **ValidatorSlashed**
- **Fields**: `{ validator: T::AccountId, slashing_count: u32 }`
- **Description**: Emitted when a validator is slashed.

### **ValidatorRemoved**
- **Fields**: `{ validator: T::AccountId, reason: Vec<u8> }`
- **Description**: Emitted when a validator is removed due to exceeding the maximum slashing count.

---

## Errors

- **`ValidatorAlreadyRegistered`**: The account is already registered as a validator.
- **`ValidatorNotRegistered`**: The account is not registered as a validator.
- **`ScoreTooLow`**: The submitted score is below the minimum threshold.
- **`TooManyValidators`**: The maximum number of validators has been reached.
- **`MaxSlashingCountReached`**: The validator has reached the maximum slashing count and is removed.

---

## Extrinsics

### **register_validator**
- **Description**: Allows an account to register as a validator.
- **Weight**: `10_000`

### **submit_score**
- **Description**: Allows a validator to submit a score.
- **Parameters**:
  - `score`: The score to be submitted.
- **Weight**: `15_000`

### **slash_validator**
- **Description**: Allows slashing of a validator for misbehavior.
- **Parameters**:
  - `validator`: The account ID of the validator to be slashed.
- **Weight**: `20_000`

---

## Testing

The `tests.rs` file includes unit tests to verify the functionality of the pallet. Key tests include:
- **Validator Registration**:
  - Valid registrations.
  - Rejection of duplicate registrations.
- **Score Submission**:
  - Valid score submissions.
  - Rejection of scores below the minimum threshold.
  - Rejection of scores for non-registered validators.
- **Slashing**:
  - Valid slashing.
  - Removal of validators after reaching the maximum slashing count.
  - Rejection of slashing for non-registered validators.

Run the tests using:

```bash
#To build files use the below command
cargo build -p pallet-cbc-pos

#To perform unit testing on pallet run below command
cargo test -p pallet-cbc-pos

#TO perform benchmarking test on the pallet use this command
cargo bench -p pallet-cbc-pos