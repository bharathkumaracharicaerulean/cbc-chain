# Pallet CBC PoI (`pallet-cbc-poi`)

<p align="center">
  <img src="../../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

The **Proof of Inference (PoI)** pallet manages computational validation for AI/ML inference results on the CBC Blockchain. It enables validators to submit ML execution hashes with confidence ratings and processes peer challenges to verify validation accuracy.

---

## Core Responsibilities

1. **Inference Submission**: Validators submit inference results (`submit_inference`) along with confidence percentages.
2. **Challenge Windows**: Opens a 35-block dispute period during which other validators can challenge the validity of a submitted result.
3. **Slashing & Boosting**: Cooperates with the PoS and DCF pallets to boost scores for verified results and slash stakes/scores for incorrect outputs.
4. **Accuracy Auditing**: Processes peer challenge evaluations on-chain to determine whether the original inference was correct.

---

## Configuration Trait (`Config`)

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;
    
    // Limits and thresholds
    type MinInferenceConfidence: Get<u32>;
    type MaxInferenceAge: Get<u32>;
    type ChallengeWindow: Get<u32>;
    
    // Monetary rewards
    type InferenceReward: Get<u128>;
    type ChallengeReward: Get<u128>;
    
    // Interface traits
    type PosInterface: PosInterface<Self::AccountId>;
    type DcfInterface: DcfInterface<Self::AccountId>;
}
```

---

## Storage Layout

* **`InferenceResults`**: Map of validator `AccountId` to their submitted `(result_hash, confidence, block_number)`.
* **`Challenges`**: Map of challenged validator `AccountId` to `(challenger_account, result, epoch_index)` indicating ongoing disputes.
* **`CurrentEpoch`**: Value tracking the current operational PoI epoch.

---

## Dispatchables (`#[pallet::call]`)

### 1. `submit_inference`
* **Access**: Signed Transaction from Validator
* **Logic**: Submits the cryptographic hash of an ML model execution alongside its confidence value. Inserts the entry into `InferenceResults` if the confidence exceeds `MinInferenceConfidence`.

### 2. `challenge_inference`
* **Access**: Signed Transaction
* **Logic**: Challenges another validator's submitted inference. If successful and within the `ChallengeWindow`, it slashes the original author and rewards the challenger.

---

## Testing & Compilation

### Unit Tests
```bash
cargo test -p pallet-cbc-poi
```