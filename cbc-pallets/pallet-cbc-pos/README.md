# Pallet CBC PoS (`pallet-cbc-pos`)

<p align="center">
  <img src="../../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

The **Proof of Stake (PoS)** pallet implements staking logic, validator registrations, locked token pools, slashing counts, and performance-based staking score ratings for the CBC Blockchain. It is a dependency for the consensus/DCF calculation layer.

---

## Core Responsibilities

1. **Validator Registrations**: Tracks register eligibility, enforces validator count limits (`MaxValidators`), and monitors validator status.
2. **Stake Management**: Enforces a minimum staking threshold (`MinStake`) and tracks locked tokens in reserves via the currency trait.
3. **Slashing Invariant Tracking**: Records validator slashes; ejects a validator if the maximum slashing limit is exceeded.
4. **Performance Rating Updates**: Updates validator ratings and stake scores dynamically; provides reward multipliers for top-performing nodes.

---

## Configuration Trait (`Config`)

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;
    type Currency: ReservableCurrency<Self::AccountId>;
    
    // Limits
    type MinValidatorScore: Get<u32>;
    type MinActiveValidators: Get<u32>;
    type MaxValidators: Get<u32>;
    type ValidatorScoreDecay: Get<u32>;
    type MaxSlashingCount: Get<u32>;
    
    // Staking economics
    type MinStake: Get<BalanceOf<Self>>;
}
```

---

## Storage Layout

* **`Validators`**: Map of `AccountId` to active status bool.
* **`ValidatorScores`**: Map of `AccountId` to current staking score.
* **`SlashingCount`**: Map of `AccountId` to slash frequency accumulator.
* **`Stake`**: Map of `AccountId` to locked token reserve quantity.
* **`ValidatorJoinTime`**: Map of `AccountId` to the block number when the validator joined.
* **`PerformanceHistory`**: Map of `AccountId` to a list of historical performance records.

---

## Pallet Dispatchables (`#[pallet::call]`)

### 1. `register_validator`
* **Access**: Signed Transaction
* **Logic**: Locks `MinStake` in reserves and registers the account in the validator set.

### 2. `submit_score`
* **Access**: Root / Governance
* **Logic**: Submits a new validator performance score.

### 3. `slash_validator`
* **Access**: Root / Governance
* **Logic**: Slashes a validator's stake; triggers ejection if the slash threshold is met.

### 4. `bond_stake` / `unbond_stake`
* **Access**: Signed Transaction from Validator
* **Logic**: Locks additional funds or releases locked funds (enforcing the minimum staking requirements).

### 5. `boost_score` / `decay_score`
* **Access**: Root / DCF Pallet Callback
* **Logic**: Programmatically scales the staking score rating up or down based on block authoring success or missed slots.

---

## Testing & Compilation

### Unit Tests
```bash
cargo test -p pallet-cbc-pos
```
