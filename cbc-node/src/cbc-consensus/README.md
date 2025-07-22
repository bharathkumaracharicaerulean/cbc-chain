# CBC Consensus

A high-performance, modular consensus framework for the CBC blockchain, combining Proof of Stake (PoS) with Proof of Inference (PoI) mechanisms.

## Overview

The CBC Consensus implements a hybrid consensus mechanism that leverages both staking and computational work (inference) to achieve secure and efficient blockchain validation. It's built on Substrate's consensus framework and provides a flexible architecture for different consensus modes.

## Features

- **Dynamic Consensus Framework (DCF)**: Core consensus engine that coordinates between validators
- **Validator Management**: On-chain validator set management and rotation
- **Epoch-based Operation**: Time-based epochs for validator set rotation and reward distribution
- **Block Production**: Efficient block production with configurable author selection
- **Finality Gadget**: Ensures block finality after sufficient confirmations
- **Metrics & Monitoring**: Built-in metrics collection for monitoring consensus health

## Architecture

### Core Components

1. **DCF Engine** (`dcf.rs`)
   - Handles core consensus logic
   - Manages block production and validation
   - Implements author selection algorithm

2. **Validator Management** (`validator_set.rs`, `author_selection.rs`)
   - Tracks active validators
   - Handles validator set changes
   - Implements selection algorithms for block production

3. **Epoch Management** (`epoch_manager.rs`)
   - Manages epoch transitions
   - Handles validator set rotation
   - Coordinates reward distribution

4. **Block Production** (`proposer_factory.rs`)
   - Creates new block proposals
   - Handles transaction inclusion
   - Manages block production timing

5. **Finality** (`finality.rs`)
   - Implements finality gadget
   - Tracks finalized blocks
   - Handles finalization proofs

## Usage

### Basic Setup

```rust
use cbc_consensus::{
    DcfConsensus,
    ConsensusParams,
    ValidatorSet,
    EpochManager,
};

// Initialize consensus with default parameters
let consensus = DcfConsensus::new(
    client.clone(),
    ConsensusParams::default()
);

// Start the consensus engine
consensus.run();
```

### Configuration

Key configuration parameters include:

```rust
pub struct ConsensusParams {
    /// Slot duration in milliseconds
    pub slot_duration: u64,
    /// Epoch length in slots
    pub epoch_length: u64,
    /// Minimum stake required to become a validator
    pub min_validator_stake: u128,
    /// Maximum number of validators
    pub max_validators: u32,
    /// Block production timeout
    pub block_production_timeout: Duration,
}
```

## Integration

The consensus engine integrates with:

- **Runtime API**: For validator set management and state queries
- **Networking**: For block and transaction propagation
- **Finality Gadget**: For block finalization
- **Metrics**: For monitoring and observability

## Metrics

Key metrics exposed:

- `consensus_block_production_time`: Time taken to produce blocks
- `consensus_block_import_time`: Time to import blocks
- `consensus_epoch_changes`: Epoch transition events
- `consensus_validator_set_size`: Current number of active validators
- `consensus_finalized_block_height`: Latest finalized block number

## Development

### Building

```bash
cargo build --release -p cbc-consensus
```

### Testing

Run the test suite with:

```bash
cargo test --all-features
```

### Benchmarks

Run benchmarks with:

```bash
cargo bench -p cbc-consensus
```

