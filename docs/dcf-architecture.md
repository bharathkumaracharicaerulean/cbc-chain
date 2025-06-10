### Documentation for DCF Architecture and Pallet

#### Overview

The Decentralized Consensus Framework (DCF) is designed to facilitate a robust and efficient consensus mechanism for blockchain networks. The DCF architecture leverages validator scores based on their stake and inference results to determine their participation in the consensus process. This document provides an overview of the DCF architecture, its components, and how to interact with the associated pallet.

---

### Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Key Components](#key-components)
   - [ValidatorScore](#validatorscore)
   - [EpochConfig](#epochconfig)
   - [Pallet Configuration](#pallet-configuration)
3. [Storage Items](#storage-items)
4. [Events](#events)
5. [Errors](#errors)
6. [Functions](#functions)
   - [Public Functions](#public-functions)
   - [Private Functions](#private-functions)
7. [Usage](#usage)
8. [Testing](#testing)

---

### Architecture Overview

The DCF architecture is built around a set of validators that participate in the consensus process. Each validator is assigned a score based on their stake and inference results, which are weighted to determine their final score. The architecture supports epoch transitions, where validators can be added or removed based on their performance.

---

### Key Components

#### ValidatorScore

The `ValidatorScore` struct holds the score details for each validator, including:

- `stake_weight`: The weight of the validator's stake.
- `inference_weight`: The weight of the validator's inference score.
- `final_score`: The computed final score for the validator.
- `last_epoch_active`: The last epoch in which the validator was active.
- `participation_count`: The number of times the validator participated in the consensus.
- `missed_blocks`: The number of blocks missed by the validator.
- `authored_blocks`: The number of blocks authored by the validator.

#### EpochConfig

The `EpochConfig` struct defines the configuration for epoch transitions, including:

- `blocks_per_epoch`: The number of blocks in each epoch.
- `min_stake`: The minimum stake required to be a validator.
- `max_validators`: The maximum number of validators allowed per epoch.

#### Pallet Configuration

The DCF pallet requires several configurations, including:

- `MaxValidators`: The maximum number of validators that can be active at once.
- `DefaultPosWeight`: The default weight for the POS score in the final calculation.
- `DefaultPoiWeight`: The default weight for the POI score in the final calculation.
- `MinActiveValidators`: The minimum number of active validators required for a new epoch.
- `MinValidatorScore`: The minimum score required for a validator to remain active.

---

### Storage Items

The DCF pallet utilizes various storage items to maintain state:

- `ValidatorStakeScores`: Maps each validator to their stake score.
- `ValidatorInferenceScores`: Maps each validator to their inference score.
- `ValidatorFinalScores`: Stores the final score and related metrics for each validator.
- `PosWeight` and `PoiWeight`: Store the weights for POS and POI scores.
- `ValidatorSet`: Stores the current set of validators.
- `EpochConfigStorage`: Stores the current epoch configuration.
- `CurrentEpoch`: Tracks the current epoch number.
- `ActiveValidators`: Stores the active validators for the current epoch.
- `ValidatorScoreHistory`: Keeps a history of scores for each validator.
- `ValidatorParticipation`: Tracks the participation metrics for each validator.
- `ValidatorLastActive`: Records the last active epoch for each validator.

---

### Events

The DCF pallet emits several events to notify about state changes:

- `ValidatorScoreUpdated`: Emitted when a validator's score is updated.
- `ConsensusWeightsUpdated`: Emitted when consensus weights are updated.
- `EpochStarted`: Emitted when a new epoch starts.
- `ValidatorScoreDecayed`: Emitted when a validator's score is decayed.
- `ValidatorScoreBoosted`: Emitted when a validator's score is boosted.
- `ValidatorEjected`: Emitted when a validator is ejected from the active set.
- `ValidatorReEntered`: Emitted when a validator re-enters the active set.

---

### Errors

The DCF pallet defines several errors that can occur during operations:

- `ValidatorNotFound`: Raised when a validator is not found.
- `InvalidWeight`: Raised when an invalid weight value is provided.
- `InvalidEpochConfig`: Raised when the epoch configuration is invalid.
- `NotEnoughValidators`: Raised when there are not enough validators for an epoch.

---

### Functions

#### Public Functions

The DCF pallet provides several public functions that can be called by users:

- `update_validator_stake_score`: Updates the stake score for a validator.
- `update_validator_inference_score`: Updates the inference score for a validator.
- `update_consensus_weights`: Updates the weights for POS and POI scores.
- `get_validator_scores`: Retrieves the scores of all validators.
- `get_current_epoch`: Retrieves the current epoch number.
- `get_active_validators`: Retrieves the list of active validators.

#### Private Functions

The pallet also contains several private functions for internal logic:

- `update_final_score`: Updates the final score for a validator based on their stake and inference scores.
- `handle_epoch_transition`: Handles the transition to a new epoch.
- `apply_score_decay`: Applies score decay to a validator's score.
- `boost_score`: Boosts a validator's score based on specific criteria.
- `eject_validator`: Ejects a validator from the active set based on their score.

---

### Usage

To use the DCF pallet, integrate it into your Substrate runtime and configure the necessary parameters in your runtime's configuration. Validators can be added or removed based on their performance, and the consensus process will automatically adjust based on the scores.

---

### Testing

The DCF pallet includes a set of unit tests to ensure the correctness of its functionality. To run the tests, use the following command:

```bash
cargo test
```

Ensure that all tests pass before deploying the pallet to a live network.

---

### Conclusion

The DCF architecture provides a flexible and efficient framework for managing consensus in a decentralized network. By leveraging validator scores and epoch transitions, it ensures that only the most capable validators participate in the consensus process, enhancing the overall security and efficiency of the network.

---

This documentation can be expanded further based on specific use cases, examples, and additional details as needed. Make sure to place this document in the appropriate documentation folder of your project, and consider using Markdown formatting for better readability.