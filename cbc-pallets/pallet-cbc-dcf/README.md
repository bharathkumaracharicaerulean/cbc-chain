# Pallet CBC DCF (`pallet-cbc-dcf`)

<p align="center">
  <img src="../../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

The **Dynamic Consensus Framework (DCF)** pallet coordinates the core consensus scoring, epoch transitions, and validator active-set configuration for the CBC Blockchain. It computes combined multi-factor trust scores and interfaces with the governance pallet to apply validator set updates (slashings, ejections, additions).

---

## Core Responsibilities

1. **Combined Scoring Engine**: Merges stake-based PoS metrics (65% weight) with computational PoI accuracy metrics (35% weight) into a single unified trust score (scaled up to `10000` BP).
2. **Epoch State Transition**: Coordinates block height epochs (100 blocks), validator rotation, performance rating decay, and historical metrics snapshotting.
3. **External Governance Callback Provider**: Implements `ProposalExecutor` and `ValidatorProvider` traits, allowing the decoupled `pallet-cbc-governance` to perform safe, authorized admin actions (e.g. slashing stakes, ejecting underperforming nodes).
4. **Safety Rails & Invariants**: Enforces boundaries on validator counts, validator cooldowns, leave restrictions, and parameter boundaries.

---

## Configuration Trait (`Config`)

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pos::Config + poi::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    // Limits on validator sets
    type MaxValidators: Get<u32>;
    type MinActiveValidators: Get<u32>;
    type MaxEpochHistory: Get<u32>;
    
    // Staking vs Computational weighting parameters
    type DefaultPosWeight: Get<u64>;
    type DefaultPoiWeight: Get<u64>;
    type PercentagePrecision: Get<u32>;
    
    // Performance scoring limits
    type MinValidatorScore: Get<u32>;
    type MaxValidatorScore: Get<u64>;
    type ValidatorScoreDecay: Get<u32>;
    
    // Operational interval configurations
    type ScoreDecayInterval: Get<u32>;
    type ParticipationUpdateInterval: Get<u32>;
    type UnderperformanceCheckInterval: Get<u32>;
}
```

---

## Storage Architecture

* **`ValidatorStates`**: Map of `AccountId` to `ValidatorState`, containing current stake score, inference score, combined trust score, total blocks produced, and missed block history.
* **`CurrentEpoch`**: Tracks the running consensus epoch sequence index.
* **`EpochHistory`**: Bounded historical log of past epochs and corresponding active validator sets.
* **`PosWeight` / `PoiWeight`**: System-wide configuration weights for combining consensus scoring factors.

---

## Pallet Extrinsics (`#[pallet::call]`)

### 1. `update_validator_stake_score`
* **Access**: Root / Authorized Signer
* **Logic**: Queries current locked stakes from the PoS pallet and updates the validator's `stake_score` caching, triggering a recomputation of the combined trust score.

### 2. `update_consensus_weights`
* **Access**: Root / Governance Proposal Only
* **Logic**: Updates the active system configuration weights (`PosWeight` and `PoiWeight`) dynamically, enforcing that the sum remains equal to the maximum precision value (`10000`).

### 3. `update_dcf_parameter`
* **Access**: Root / Governance Proposal Only
* **Logic**: Safely updates operational parameters (e.g. decay intervals, performance check intervals, score decay values) after verifying input values against pre-defined safety rails.

---

## Integrated Traits (Decoupled Governance Interface)

To ensure strict modular separation, the DCF pallet implements the following interfaces which are consumed by the governance pallet:

```rust
pub trait ProposalExecutor<AccountId, Balance> {
    fn execute_slash(validator: &AccountId, amount: Balance) -> DispatchResult;
    fn execute_ejection(validator: &AccountId) -> DispatchResult;
    fn execute_reward(validator: &AccountId, amount: Balance) -> DispatchResult;
}

pub trait ValidatorProvider<AccountId> {
    fn active_validators() -> Vec<AccountId>;
    fn is_active(validator: &AccountId) -> bool;
}
```

---

## Testing & Benchmarks

### Running Pallet Unit Tests
```bash
cargo test -p pallet-cbc-dcf
```

### Pallet Benchmarking
Generate transaction weights based on actual hardware execution profiles:
```bash
# Build node with benchmarking features
cargo build --release --features runtime-benchmarks

# Run benchmarks for all extrinsics in the pallet
./target/release/cbc-node benchmark pallet \
  --pallet pallet_cbc_dcf \
  --extrinsic "*" \
  --steps 50 \
  --repeat 20 \
  --output ./cbc-pallets/pallet-cbc-dcf/src/weights.rs
```
