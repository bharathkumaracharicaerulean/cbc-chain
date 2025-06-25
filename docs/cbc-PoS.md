# CBC-PoS Pallet Documentation

The `cbc-pos` pallet implements a **Proof of Stake (PoS)** mechanism for the CBC blockchain. It manages validators, their scores, slashing, stake management, and epoch-based updates to ensure a secure and fair consensus process.

---

## 1. Purpose

The `cbc-pos` pallet is responsible for:
- Managing the validator set with stake requirements.
- Allowing validators to register and submit scores.
- Penalizing misbehaving validators through slashing.
- Removing validators who exceed the maximum slashing count.
- Tracking validator stakes and applying score decay over time.
- Integrating with PoI system for score boosting/slashes.

---

## 2. Key Features

### **Validator Management**
- Validators can register to participate in block production.
- A maximum number of validators is enforced.
- Validators must maintain minimum stake requirements.

### **Stake Management**
- Validators can bond and unbond stake.
- Minimum stake amount requirement.
- Stake tracking per validator.

### **Score Management**
- Validators can submit scores to indicate their performance.
- Scores below a minimum threshold are rejected.
- Score decay applied per epoch.
- Score boosting/slashes via PoI integration.

### **Slashing**
- Validators can be slashed for misbehavior.
- Validators are removed if their slashing count exceeds the maximum allowed.
- Slashing count tracked per validator.

### **Epoch Management**
- Tracks the current epoch.
- Applies score decay to validators at the end of each epoch.

---

## 3. Key Storage

### **Validators**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, bool>`
- **Description**: Tracks registered validators and their active status.

### **ValidatorScores**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, u32>`
- **Description**: Stores the current score for each validator.

### **Stake**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>`
- **Description**: Tracks the bonded stake amount for each validator.

### **CurrentEpoch**
- **Type**: `StorageValue<_, u32, ValueQuery>`
- **Description**: Tracks the current epoch number.

### **SlashingCount**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, u32>`
- **Description**: Tracks the number of times a validator has been slashed.

---

## 4. Key Events

### **ValidatorRegistered**
- **Fields**: `{ validator: T::AccountId }`
- **Description**: Emitted when a validator successfully registers.

### **ScoreSubmitted**
- **Fields**: `{ validator: T::AccountId, score: u32 }`
- **Description**: Emitted when a validator submits a score.

### **ValidatorSlashed**
- **Fields**: `{ validator: T::AccountId, slashing_count: u32 }`
- **Description**: Emitted when a validator is slashed.

### **ValidatorRemoved**
- **Fields**: `{ validator: T::AccountId, reason: Vec<u8> }`
- **Description**: Emitted when a validator is removed from the set.

### **StakeBonded**
- **Fields**: `{ validator: T::AccountId, amount: BalanceOf<T> }`
- **Description**: Emitted when a validator bonds stake.

### **StakeUnbonded**
- **Fields**: `{ validator: T::AccountId, amount: BalanceOf<T> }`
- **Description**: Emitted when a validator unbonds stake.

---

## 5. Key Errors

### **ValidatorAlreadyRegistered**
- **Description**: Validator is already registered.

### **ValidatorNotRegistered**
- **Description**: Validator is not registered.

### **InvalidScore**
- **Description**: Score is invalid.

### **TooManyValidators**
- **Description**: Maximum number of validators reached.

### **ScoreTooLow**
- **Description**: Score is below minimum threshold.

### **MaxSlashingCountReached**
- **Description**: Validator has been slashed too many times.

### **InsufficientStake**
- **Description**: Validator does not meet minimum stake requirements.

### **InvalidStakeAmount**
- **Description**: Invalid stake amount specified.

---

## 6. Tests

The `tests.rs` file includes unit tests to verify the functionality of the pallet. Key tests include:

### **Validator Registration**
- **Test**: `test_register_validator_success`
  - Verifies that a validator can register successfully.
- **Test**: `test_register_validator_already_registered`
  - Ensures that duplicate registrations are rejected.

### **Score Submission**
- **Test**: `test_submit_score_success`
  - Verifies that a validator can submit a valid score.
- **Test**: `test_submit_score_not_registered`
  - Ensures that non-registered validators cannot submit scores.
- **Test**: `test_submit_score_too_low`
  - Ensures that scores below the minimum threshold are rejected.

### **Slashing**
- **Test**: `test_slash_validator_success`
  - Verifies that a validator can be slashed successfully.
- **Test**: `test_slash_validator_removal`
  - Ensures that a validator is removed after reaching the maximum slashing count.
- **Test**: `test_slash_validator_not_registered`
  - Ensures that non-registered validators cannot be slashed.

### **Unauthorized Actions**
- **Test**: `test_unauthorized_register`
  - Ensures that only signed origins can register validators.
- **Test**: `test_unauthorized_submit_score`
  - Ensures that only signed origins can submit scores.
- **Test**: `test_unauthorized_slash`
  - Ensures that only signed origins can slash validators.

---

## 6. Runtime API

The pallet exposes the following runtime API:

```rust
trait PosApi<AccountId, Balance> {
    fn get_validator_stake(validator: AccountId) -> Balance;
    fn get_validator_score(validator: AccountId) -> u32;
    fn get_active_validators() -> Vec<AccountId>;
    fn get_slashing_count(validator: AccountId) -> u32;
}
```

## 7. Mock Runtime

The `mock.rs` file defines a mock runtime for testing the pallet. Key configurations include:

### **System Configuration**
- Implements `frame_system::Config` with mock types for accounts, blocks, and events.

### **Pallet Configuration**
- Implements `pallet_cbc_pos::Config` with the following constants:
  - `MinValidatorScore`: Minimum score required for a validator to be considered active.
  - `MinActiveValidators`: Minimum number of active validators required.
  - `MaxValidators`: Maximum number of validators allowed.
  - `ValidatorScoreDecay`: Score decay per epoch.
  - `MaxSlashingCount`: Maximum slashing count before removal.
  - `MinStake`: Minimum stake amount required for validators.

### **Genesis Storage**
- Initializes the mock runtime with default storage values for validators, scores, current epoch, slashing counts, and stakes.

---

## 8. Benchmarking

The `benchmarking.rs` file provides benchmarks for the pallet's extrinsics. Key benchmarks include:

### **Register Validator**
- **Setup**: Creates a whitelisted caller.
- **Action**: Registers the caller as a validator.
- **Verification**: Ensures the validator is added to the `Validators` storage.

### **Submit Score**
- **Setup**: Registers a validator and sets a valid score.
- **Action**: Submits the score for the validator.
- **Verification**: Ensures the score is stored in the `ValidatorScores` storage.

### **Slash Validator**
- **Setup**: Registers a validator and sets an initial slashing count.
- **Action**: Slashes the validator until the maximum slashing count is reached.
- **Verification**: Ensures the validator is removed from the `Validators` storage.

---

## 9. Weights

The `weights.rs` file defines the weights for the pallet's extrinsics. Key weights include:

### **Register Validator**
- **Weight**: `10_000`
- **Description**: Includes the cost of writing to the `Validators` storage.

### **Submit Score**
- **Weight**: `15_000`
- **Description**: Includes the cost of writing to the `ValidatorScores` storage.

### **Slash Validator**
- **Weight**: `20_000`
- **Description**: Includes the cost of updating the `SlashingCount` storage and removing the validator if necessary.

---

## 10. Conclusion

The `cbc-pos` pallet provides a  Proof of Stake mechanism for managing validators, their scores, and slashing. It includes comprehensive tests, mocks, and benchmarks to ensure reliability and performance. For more details, refer to the source code in the `cbc-pallets/pallet-cbc-pos` directory.