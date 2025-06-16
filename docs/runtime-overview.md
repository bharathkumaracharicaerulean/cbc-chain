# CBC Runtime Overview

This document provides an overview of the CBC runtime, including the purpose of each pallet, key constants, and the genesis default accounts.

---

## 1. Pallets and Their Purpose

### **System (`frame_system`)**
- Manages the basic blockchain system, including accounts, transactions, and block execution.

### **Timestamp (`pallet_timestamp`)**
- Sets and validates the timestamp for each block.
- Used for time-dependent logic in the runtime.

### **Aura (`pallet_aura`)**
- Provides the Aura consensus mechanism for block production.

### **Grandpa (`pallet_grandpa`)**
- Provides the Grandpa finality mechanism for finalizing blocks.

### **Balances (`pallet_balances`)**
- Manages account balances, token transfers, and existential deposits.
- Supports reserving, freezing, and slashing balances.

### **Transaction Payment (`pallet_transaction_payment`)**
- Handles transaction fees and their payment.
- Supports weight-based fee calculation.

### **Sudo (`pallet_sudo`)**
- Allows a privileged account (sudo key) to execute administrative tasks.
- Useful for runtime upgrades and testing.


### **Pallet-cbc-PoS**
- **Purpose**:  
  The `pallet-cbc-pos` (Proof of Stake) manages validators and their scores in the CBC blockchain. It ensures that only eligible validators participate in block production and finality, and it handles slashing and removal of validators for misbehavior.
  
- **Key Features**:
  - **Validator Registration**: Allows accounts to register as validators, provided the maximum validator limit has not been reached.
  - **Score Submission**: Validators can submit their scores, which are used to determine their eligibility for block production.
  - **Slashing**: Validators can be slashed for misbehavior, and if their slashing count exceeds the maximum allowed, they are removed from the validator set.
  - **Epoch Management**: Tracks the current epoch and applies score decay to validators over time.

- **Key Storage**:
  - `Validators`: Tracks registered validators and their active status.
  - `ValidatorScores`: Stores the scores of validators.
  - `CurrentEpoch`: Tracks the current epoch or round.
  - `SlashingCount`: Tracks the number of times a validator has been slashed.

- **Key Events**:
  - `ValidatorRegistered`: Emitted when a new validator is registered.
  - `ScoreSubmitted`: Emitted when a validator submits a score.
  - `ValidatorSlashed`: Emitted when a validator is slashed.
  - `ValidatorRemoved`: Emitted when a validator is removed due to exceeding the maximum slashing count.

---

### **Pallet-cbc-PoI**
- **Purpose**:  
  The `pallet-cbc-poi` (Proof of Inference) manages inference results submitted by validators and allows challenges to be raised against these results. It incentivizes validators to submit accurate inference results and penalizes invalid submissions.

- **Key Features**:
  - **Inference Submission**: Validators can submit inference results along with a confidence score. The results are stored with the current epoch.
  - **Challenges**: Validators can challenge the inference results of others within a specified challenge window.
  - **Epoch Management**: Tracks the current epoch and ensures that inference results and challenges are valid within the allowed time frame.

- **Key Storage**:
  - `InferenceResults`: Stores inference results submitted by validators, along with the epoch in which they were submitted.
  - `Challenges`: Tracks challenges raised against inference results, including the challenger, challenged account, and the result.
  - `CurrentEpoch`: Tracks the current epoch or round.

- **Key Events**:
  - `InferenceSubmitted`: Emitted when a validator submits an inference result.
  - `InferenceChallenged`: Emitted when a validator challenges an inference result.
  - `ChallengeResolved`: Emitted when a challenge is resolved, indicating whether it was successful or not.

---

## 2. Key Constants

### **Time Constants**
- `MINUTES`: Number of blocks in a minute (`60_000 / MILLI_SECS_PER_BLOCK`).
- `HOURS`: Number of blocks in an hour (`MINUTES * 60`).
- `DAYS`: Number of blocks in a day (`HOURS * 24`).

### **Balance Constants**
- `UNIT`: Base unit for balances (`1_000_000_000_000`).
- `MILLI_UNIT`: Milli unit for balances (`1_000_000_000`).
- `MICRO_UNIT`: Micro unit for balances (`1_000_000`).
- `EXISTENTIAL_DEPOSIT`: Minimum balance required to keep an account alive (`MILLI_UNIT`).

### **Blockchain Parameters**
- `BLOCK_HASH_COUNT`: Number of recent blocks to store in the block hash map (`2400`).

---

## 3. Genesis Default Accounts

### **Development Configuration**
- **Authorities**:
  - Alice (Aura and Grandpa authority).
- **Endowed Accounts**:
  - Alice: Pre-funded with a large balance.
  - Bob: Pre-funded with a large balance.
- **Sudo Key**:
  - Alice: Assigned as the sudo (root) key.

### **Local Testnet Configuration**
- **Authorities**:
  - Alice and Bob (Aura and Grandpa authorities).
- **Endowed Accounts**:
  - All keyring accounts except "One" and "Two" are pre-funded.
- **Sudo Key**:
  - Alice: Assigned as the sudo (root) key.

---

This document summarizes the runtime's structure and configuration. For more details, refer to the source code in the `cbc-runtime` directory.