# CBC Runtime Overview

The CBC runtime implements a blockchain system with a combination of Proof of Stake (PoS) and Proof of Inference (PoI) consensus mechanisms. This document provides an overview of the runtime configuration, pallets, and key parameters.

---

## 1. Runtime Version

- **Spec Name**: `cbc-runtime`
- **Impl Name**: `cbc-runtime`
- **Spec Version**: 100
- **Impl Version**: 1
- **Transaction Version**: 1
- **System Version**: 1

---

## 2. Block Parameters

- **Block Time**: 6 seconds (6000 milliseconds)
- **Slot Duration**: 6 seconds
- **Block Hash Count**: 2400 blocks (recent blocks stored)

---

## 3. Balance Parameters

- **Base Unit**: 1,000,000,000,000 (10^12)
- **Milli Unit**: 1,000,000,000 (10^9)
- **Micro Unit**: 1,000,000 (10^6)
- **Existential Deposit**: 1,000,000,000 (Milli Unit)
- **DOLLARS**: 1,000,000,000,000 (Base Unit)

---

## 4. Pallets and Their Purpose

### **System (`frame_system`)**
- Manages the basic blockchain system, including accounts, transactions, and block execution.
- Pallet Index: 0

### **Timestamp (`pallet_timestamp`)**
- Sets and validates the timestamp for each block.
- Used for time-dependent logic in the runtime.
- Pallet Index: 1

### **Balances (`pallet_balances`)**
- Manages account balances, token transfers, and existential deposits.
- Supports reserving, freezing, and slashing balances.
- Pallet Index: 2

### **Transaction Payment (`pallet_transaction_payment`)**
- Handles transaction fees and their payment.
- Supports weight-based fee calculation.
- Pallet Index: 3

### **Sudo (`pallet_sudo`)**
- Allows a privileged account (sudo key) to execute administrative tasks.
- Useful for runtime upgrades and testing.
- Pallet Index: 4

### **Pallet-cbc-PoI**
- Implements Proof of Inference consensus mechanism.
- Manages inference results and challenges.
- Pallet Index: 6

### **Pallet-cbc-PoS**
- Implements Proof of Stake consensus mechanism.
- Manages validator stakes and scores.
- Pallet Index: 7

### **Pallet-cbc-Dcf**
- Combines PoS and PoI scores with configurable weights.
- Provides governance mechanisms for validator management.
- Pallet Index: 8

---

## 5. Key Constants

### **Time Constants**
- **MINUTES**: 10 blocks (60 seconds)
- **HOURS**: 600 blocks (3600 seconds)
- **DAYS**: 14400 blocks (86400 seconds)

### **Balance Constants**
- **UNIT**: 1,000,000,000,000
- **MILLI_UNIT**: 1,000,000,000
- **MICRO_UNIT**: 1,000,000
- **EXISTENTIAL_DEPOSIT**: 1,000,000,000
- **DOLLARS**: 1,000,000,000,000

### **Blockchain Constants**
- **Block Hash Count**: 2400 blocks
- **Block Time**: 6 seconds
- **Slot Duration**: 6 seconds

---

## 6. Runtime Types

### **Basic Types**
- **Signature**: MultiSignature
- **AccountId**: Account identifier
- **Balance**: u128
- **Nonce**: u32
- **Hash**: sp_core::H256
- **BlockNumber**: u32
- **Address**: MultiAddress
- **Header**: generic::Header
- **Block**: generic::Block

### **Transaction Types**
- **TxExtension**: Transaction extensions
- **UncheckedExtrinsic**: Raw transaction type
- **SignedPayload**: Signed transaction payload

---

## 7. Session Keys

The runtime uses ed25519 keys for DCF consensus:
- **DcfPublic**: ed25519 application-specific public key

---

## 8. Runtime APIs

The runtime exposes several APIs for external interaction:
- Runtime API versions defined in `apis::RUNTIME_API_VERSIONS`
- Used by tools like Polkadot-JS Apps to interact with the chain

---

## 9. Runtime Executive

The runtime uses FRAME's Executive module for:
- Dispatching calls to appropriate pallets
- Handling block execution
- Managing runtime upgrades
- Processing transactions

---

## 10. Genesis Configuration

The runtime supports genesis configuration presets for:
- Initial validator set
- Initial balances
- Initial PoS/PoI parameters
- Initial DCF configuration

---

## 11. Transaction Processing

The runtime uses the following transaction extensions:
- CheckNonZeroSender
- CheckSpecVersion
- CheckTxVersion
- CheckGenesis
- CheckEra
- CheckNonce
- CheckWeight
- ChargeTransactionPayment
- CheckMetadataHash
- WeightReclaim

---

## 12. Security Considerations

1. **Block Time**: Fixed at 6 seconds to ensure consistent block production
2. **Balance Parameters**: Secure existential deposit to prevent dust accounts
3. **Transaction Processing**: Multiple checks to prevent replay attacks
4. **Consensus Security**: Combined PoS/PoI mechanism for enhanced security
5. **Key Management**: Secure ed25519 keys for consensus

---

## 13. Future Enhancements

1. **Runtime Upgrades**: Support for runtime versioning
2. **Consensus Improvements**: Potential for additional consensus mechanisms
3. **Transaction Processing**: Enhanced transaction validation
4. **API Expansion**: Additional runtime APIs for external interaction
5. **Configuration**: More flexible runtime configuration options

---

## 14. Conclusion

The CBC runtime implements a robust blockchain system with a unique combination of PoS and PoI consensus mechanisms. It provides a secure and efficient platform for blockchain operations while maintaining flexibility through its modular pallet architecture and configurable parameters.
  
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
  - Alice 
- **Endowed Accounts**:
  - Alice: Pre-funded with a large balance.
  - Bob: Pre-funded with a large balance.
- **Sudo Key**:
  - Alice: Assigned as the sudo (root) key.

### **Local Testnet Configuration**
- **Authorities**:
  - Alice and Bob 
- **Endowed Accounts**:
  - All keyring accounts except "One" and "Two" are pre-funded.
- **Sudo Key**:
  - Alice: Assigned as the sudo (root) key.

---

This document summarizes the runtime's structure and configuration. For more details, refer to the source code in the `cbc-runtime` directory.