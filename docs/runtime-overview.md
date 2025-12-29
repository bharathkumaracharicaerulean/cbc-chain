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

### **PalletCbcPoi (`pallet_cbc_poi`)**
- Implements Proof of Inference consensus mechanism.
- Manages inference results and challenges.
- Pallet Index: 6

### **PalletCbcPos (`pallet_cbc_pos`)**
- Implements Proof of Stake consensus mechanism.
- Manages validator stakes and scores.
- Pallet Index: 7

### **Dcf (`pallet_cbc_dcf`)**
- Combines PoS and PoI scores with configurable weights.
- Provides governance mechanisms for validator management.
- Pallet Index: 8

---

## 5. Detailed Pallet Descriptions

### **PalletCbcPoi (`pallet_cbc_poi`)**
- **Purpose**: The Proof of Inference pallet manages inference results submitted by validators and allows challenges to be raised against these results. It incentivizes validators to submit accurate inference results and penalizes invalid submissions.

- **Key Parameters**:
  - **MinInferenceConfidence**: 80 (minimum confidence score required)
  - **MaxInferenceAge**: 10 blocks (maximum age for inference results)
  - **ChallengeWindow**: 5 blocks (time window for challenges)
  - **InferenceReward**: 1000 units (reward for valid inference)
  - **ChallengeReward**: 500 units (reward for successful challenge)

### **PalletCbcPos (`pallet_cbc_pos`)**
- **Purpose**: The Proof of Stake pallet manages validator registration, stake management, and score calculation based on staked tokens.

- **Key Parameters**:
  - **MinValidatorScore**: 50 (minimum score to remain active)
  - **MinActiveValidators**: 3 (minimum number of active validators)
  - **MaxValidators**: 100 (maximum number of validators)
  - **ValidatorScoreDecay**: 10 (score decay per epoch)
  - **MaxSlashingCount**: 3 (maximum slashes before removal)
  - **MinStake**: 1000 DOLLARS (minimum stake required)

### **Dcf (`pallet_cbc_dcf`)**
- **Purpose**: The Dynamic Consensus Framework combines PoS and PoI scores to create a hybrid consensus mechanism. It manages validator selection, epoch transitions, and consensus weight configuration.

- **Key Parameters**:
  - **DcfMaxValidators**: 100 (maximum validators in DCF)
  - **DefaultPosWeight**: 60% (weight assigned to PoS scores)
  - **DefaultPoiWeight**: 40% (weight assigned to PoI scores)
  - **MaxValidatorsPerEpoch**: 50 (maximum validators per epoch)
  - **MaxValidatorScore**: 100 (maximum possible validator score)
  - **BlockAuthorshipBoost**: 10 (score boost for authoring blocks)
  - **MissedBlockPenalty**: 5 (penalty for missing block production)
  - **InferenceBoostLow**: 2 (low confidence inference boost)
  - **InferenceBoostMedium**: 5 (medium confidence inference boost)
  - **InferenceBoostHigh**: 10 (high confidence inference boost)

---

## 6. Key Constants

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

## 7. Runtime Types

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

## 8. Session Keys

The runtime uses ed25519 keys for DCF consensus:
- **DcfPublic**: ed25519 application-specific public key

---

## 9. Runtime APIs

The runtime exposes several APIs for external interaction:
- Runtime API versions defined in `apis::RUNTIME_API_VERSIONS`
- Used by tools like Polkadot-JS Apps to interact with the chain

---

## 10. Runtime Executive

The runtime uses FRAME's Executive module for:
- Dispatching calls to appropriate pallets
- Handling block execution
- Managing runtime upgrades
- Processing transactions

---

## 11. Genesis Configuration

The runtime supports genesis configuration presets for:
- Initial validator set
- Initial balances
- Initial PoS/PoI parameters
- Initial DCF configuration

---

## 12. Transaction Processing

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

## 13. Security Considerations

1. **Block Time**: Fixed at 6 seconds to ensure consistent block production
2. **Balance Parameters**: Secure existential deposit to prevent dust accounts
3. **Transaction Processing**: Multiple checks to prevent replay attacks
4. **Consensus Security**: Combined PoS/PoI mechanism for enhanced security
5. **Key Management**: Secure ed25519 keys for consensus

---

## 14. Future Enhancements

1. **Runtime Upgrades**: Support for runtime versioning
2. **Consensus Improvements**: Potential for additional consensus mechanisms
3. **Transaction Processing**: Enhanced transaction validation
4. **API Expansion**: Additional runtime APIs for external interaction
5. **Configuration**: More flexible runtime configuration options

---

## 15. Conclusion

The CBC runtime implements a robust blockchain system with a unique combination of PoS and PoI consensus mechanisms. It provides a secure and efficient platform for blockchain operations while maintaining flexibility through its modular pallet architecture and configurable parameters.

This document summarizes the runtime's structure and configuration. For more details, refer to the source code in the `cbc-runtime` directory.