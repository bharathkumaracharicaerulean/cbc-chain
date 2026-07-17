# Pallet CBC DVF (`pallet-cbc-dvf`)

<p align="center">
  <img src="../../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

The **Dynamic Voting Framework (DVF)** pallet is responsible for fast BFT-style finality validation on the CBC Blockchain. It handles validator vote submissions, tallies active weights, and processes BFT justification proofs to mutate the finalized block state on-chain.

---

## Core Responsibilities

1. **Vote Verification**: Validates block vote signatures, verifying they originate from active validators.
2. **Weight Computation**: Calculates each validator's voting weight based on their PoS stake combined with their DCF performance ratings:
   $$\text{Voting Weight} = (\text{Stake} \times \text{StakeWeightFactor}) + \text{min}(\text{Score} \times \text{ScoreWeightFactor}, \text{ScoreBoostCap})$$
3. **Threshold Checking**: Tallies incoming votes; once a candidate block hash reaches a $\ge 67\%$ (two-thirds) threshold of the total voting weight, a finality trigger is fired.
4. **Justification Finalization**: Validates unsigned justification payloads submitted from network aggregators and updates the canonical finalized block height state.
5. **Validator Registry**: Co-manages validator registration and balance reserves.

---

## Configuration Trait (`Config`)

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_cbc_pos::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    type Signer: Parameter + Member + IdentifyAccount<AccountId = Self::AccountId>;
    type Signature: Parameter + Member + Verify<Signer = Self::Signer> + MaxEncodedLen;
    
    // Weight parameters
    type StakeWeightFactor: Get<u128>;
    type ScoreWeightFactor: Get<u128>;
    type ScoreBoostCap: Get<u128>;
    
    // Finality parameters
    type FinalityThreshold: Get<Perbill>; // e.g. 66.66%
    type FinalityCheckpointInterval: Get<BlockNumberFor<Self>>;
    type VoteRetentionRounds: Get<u32>;
    type MaxValidators: Get<u32>;
}
```

---

## Storage Layout

* **`EpochVotingWeight`**: Map of `AccountId` to current epoch's voting weight configuration.
* **`TotalVotingWeight`**: Total sum of voting weights distributed in the active epoch.
* **`VoteRecords`**: Maps a voting round and validator `AccountId` to their submitted `DvfVote` record.
* **`VoteTallies`**: Maps candidate block hashes to current accumulated voting weights.
* **`FinalizedBlockNumber`**: Cache indicating the highest finalized block index.
* **`FinalizedBlockHash`**: Cache indicating the block hash of the highest finalized block.

---

## Dispatchables (`#[pallet::call]`)

### 1. `submit_dvf_vote`
* **Access**: Signed Transaction from Validator
* **Parameters**: `vote` structure containing block number, hash, index, epoch ID, and signature.
* **Flow**:
  1. Validates signature on payload.
  2. Ensures target block is a checkpoint block and has not already been finalized.
  3. Increments the block candidate's vote tally by the voter's calculated weight.
  4. Triggers block finalization if the accumulated weight meets or exceeds the 67% threshold.

### 2. `submit_justification`
* **Access**: Unsigned Transaction (`None` origin)
* **Parameters**: `justification` payload containing validator signatures.
* **Flow**: Verifies that the signatures meet the BFT quorum requirement and advances the finalized chain state.

### 3. `join_validators`
* **Access**: Signed Transaction
* **Flow**: Registers the account in the validator registry, locking the minimum stake requirement (`MinStake`) in reserves.

---

## Testing & Compilation

### Unit Tests
```bash
cargo test -p pallet-cbc-dvf
```
