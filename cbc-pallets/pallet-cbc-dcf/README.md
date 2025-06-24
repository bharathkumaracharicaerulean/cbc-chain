# pallet-cbc-dcf

The **Dynamic Consensus Framework (DCF)** pallet provides advanced validator management, scoring, and on-chain governance for the CBC-Chain. It enables dynamic validator sets, configurable consensus weights, and robust governance mechanisms for slashing, rewards, and ejection.

## Features

- **Validator Scoring:** Combines Proof-of-Stake (PoS) and Proof-of-Inference (PoI) scores with configurable weights.
- **Epoch Management:** Handles epoch transitions, validator activity tracking, score decay, and validator set updates.
- **Governance:** On-chain proposals for slashing, rewarding, and ejecting validators, with voting and execution logic.
- **Block Authorship Tracking:** Monitors block authorship and missed blocks, applying score boosts or penalties.
- **Runtime APIs:** Exposes APIs for querying validator scores, participation, epoch state, and expected block authors.
- **Sudo Controls:** Governance mode toggling and sudo-only operations for manual intervention and testing.

## Storage

- `ValidatorStates`: State for each validator, including scores and history.
- `PosWeight` / `PoiWeight`: Current weights for PoS and PoI in scoring.
- `ValidatorSet` / `ActiveValidators`: All and currently active validators.
- `EpochConfigStorage`: Current epoch configuration.
- `CurrentEpoch`: Current epoch number.
- `GovernanceModeEnabled`: Whether governance mode is enabled.
- `Proposals`, `ProposalVotes`, `NextProposalId`: Governance proposal tracking.
- `PendingValidatorActions`: Pending join/leave requests.
- `EpochHistories`: Recent epoch analytics.

## Events

- `ValidatorScoreUpdated`, `ValidatorScoreBoosted`, `ValidatorScoreDecayed`
- `EpochStarted`, `ValidatorEjected`, `ValidatorReEntered`
- `ProposalSubmitted`, `ProposalVoted`, `ProposalExecuted`, `ProposalPassed`, `ProposalRejected`
- `GovernanceModeToggled`, `ValidatorJoined`, `ValidatorLeft`

## Errors

- `ValidatorNotFound`, `InvalidWeight`, `InvalidEpochConfig`
- `NotEnoughValidators`, `NotAllowedInGovernanceMode`, `NotValidator`
- `AlreadyVoted`, `ProposalNotApproved`, `ProposalAlreadyExecuted`

## Dispatchable Calls

- `update_validator_stake_score`, `update_validator_inference_score`
- `update_consensus_weights`, `set_governance_mode`, `sudo_advance_epoch`
- `submit_proposal`, `vote_proposal`, `execute_proposal`
- `propose_slash_validator`, `propose_reward_validator`, `propose_eject_validator`
- `join_validator_set`, `leave_validator_set`

## Runtime APIs

See [`DcfApi`](src/lib.rs) for:
- Validator scores, epoch info, participation, expected authors, and more.

## Usage

Integrate the pallet in your runtime and configure the required parameters. Use the extrinsics and APIs to manage validators, epochs, and governance.

## Testing

Run unit tests with:

```sh
cargo test -p pallet-cbc-dcf
```

## References

- [Architecture documentation](../../docs/dcf-architecture.md)
-