# CBC Node (`cbc-node`)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

<p align="center">
  <img src="../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

A production-grade blockchain node implementing the **Caerulean ByteChains (CBC)** protocol. It integrates a hybrid Dynamic Consensus Framework (DCF) with Proof of Stake (PoS) and Proof of Inference (PoI) capabilities, backed by the Dynamic Voting Framework (DVF) for BFT finality.

---

## Table of Contents
1. [Core Features](#core-features)
2. [Quick Start & Installation](#quick-start-&-installation)
3. [CLI Commands & Options Reference](#cli-commands-&-options-reference)
4. [Architecture & Namespace Directory](#architecture-&-namespace-directory)
5. [Specialized RPC API Catalog (35 Endpoints)](#specialized-rpc-api-catalog-35-endpoints)
6. [Testing & Benchmarking Suite](#testing-&-benchmarking-suite)
7. [Operational Monitoring & Telemetry](#operational-monitoring-&-telemetry)

---

## Core Features

- **Hybrid Consensus Model**: Integrates PoS economic safety with PoI runtime performance metrics.
- **Dynamic Consensus Engine (DCF)**: Epoch-based round-robin expected author selection, dynamic scoring, and score decay operations.
- **Advanced Block Finality (DVF)**: Gossip-based block vote creation, fast vote aggregation, and signature justification proof finalization.
- **Specialized RPC Layer**: High-density RPC API exposing 35 specialized endpoints across 7 unique namespaces.
- **Node Operations Security**: Configurable RPC rate limiting, unsafe method controls, and private validator allowlist validation.
- **Integrated Tooling**: Embedded fork detection, author query utilities, developer faucets, and telemetry.

---

## Quick Start & Installation

### Prerequisites
To compile the CBC Node, your development environment must have:
* **Rust Toolchain**: Stable Rust (via `rustup`) with the `wasm32-unknown-unknown` target installed.
* **C/C++ Compiler**: Clang (v11 or higher) and GCC/LLVM.
* **Libraries**: SSL/OpenSSL (`libssl-dev`), CMake, `pkg-config`, and `make`.

### Building the Node
```bash
# Clone the repository
git clone https://github.com/bharathkumarachari/cbc-chain.git
cd cbc-chain

# Build the node in release mode (optimizations enabled)
cargo build --release
```

### Launching the Node
```bash
# Start a single-node development network (Alice validator)
./target/release/cbc-node --dev

# Start a local testnet node
./target/release/cbc-node --chain local
```

---

## CLI Commands & Options Reference

The node binary includes core consensus features as well as custom CLI hooks for network operations.

### Custom CLI Commands
- `info`: Prints core node system parameters, identity details, and local genesis constants.
- `health`: Performs validation checks against running processes, reporting diagnostic health codes.
- `fork-check`: Run-time tool querying peers to validate block finality and check for forks.
- `query-authors`: Query validator schedules and predicted expected block authors for an epoch.
- `faucet`: Development/testnet tool to request token funding.
- `runtime-upgrade`: Performs an on-chain WASM runtime upgrade (requires Sudo/Root permissions).

### Key Custom CLI Options
| Flag | Description | Default |
|---|---|---|
| `--cbc-mode <MODE>` | Custom operational configuration mode (`production`, `testing`, `development`). | `production` |
| `--enable-cbc-extensions` | Exposes additional specialized developer RPC endpoints. | `false` |
| `--log-file <PATH>` | Diverts node logger output from stdout to a designated log file. | `None` |
| `--cbc-log-only` | Filters out general node logs, showing only `[cerulea]` messages. | `false` |
| `--unsafe-rpc-expose` | Bypasses local filters, exposing administrative endpoints to public networks. | `false` |
| `--rpc-rate-limit-window <SECS>` | Duration window in seconds for RPC rate limiting. | `60` |
| `--rpc-rate-limit-requests <NUM>` | Maximum RPC requests allowed per account per window. | `100` |

---

## Architecture & Namespace Directory

```
cbc-node/
├── src/
│   ├── cbc-consensus/      # Consensus sub-crate (DCF/DVF workflows)
│   ├── main.rs             # Node bootstrapper & entrypoint
│   ├── cli.rs              # CLI arguments configuration
│   ├── command.rs          # CLI command execution handlers
│   ├── chain_spec.rs       # Genesis parameters & authority configurations
│   ├── rpc.rs              # JSON-RPC Server & 35 endpoint implementations
│   ├── service.rs          # CBC network service constructor & runtime threads
│   ├── logging.rs          # Custom structured logging and file output management
│   ├── block_tracker.rs    # Validator blocks authored vs. missed tracker
│   ├── fork_detection.rs   # Fork detection engine implementation
│   ├── benchmarking.rs     # Overhead and benchmarking infrastructure hooks
│   └── lib.rs              # Library definitions
├── tests/                  # Integration tests (RPC, multi-node setups, log management)
└── Cargo.toml              # Node manifest dependencies
```

---

## Specialized RPC API Catalog (35 Endpoints)

The CBC node implements 35 specialized JSON-RPC endpoints mapped across 7 namespaces:

### 1. `cbc_` Namespace (9 endpoints)
Comprehensive state profiles and health checks:
* `cbc_getCurrentEpoch`: Gets the current epoch number.
* `cbc_getValidatorProfile`: Gets a validator's combined PoS/PoI profiles.
* `cbc_getTrustScore`: Gets a validator's trust score breakdown.
* `cbc_listValidators`: Lists active validators.
* `cbc_getStatus`: Gets system-wide stats and telemetry.
* `cbc_describe`: Lists description metadata for all active RPC endpoints.
* `cbc_health`: Gets validator diagnostic codes.
* `cbc_getBlockAuthoringStats`: Gets authored/missed stats for a validator.
* `cbc_getAllBlockAuthoringStats`: Gets stats for all validators.

### 2. `dcf_` Namespace (8 endpoints)
DCF consensus state and parameter verification:
* `dcf_getCurrentAuthor`: Gets the current block author.
* `dcf_getExpectedAuthor`: Gets the expected author for a specific block height.
* `dcf_getValidatorScores`: Returns all active validator scores.
* `dcf_getConsensusWeights`: Returns PoS vs. PoI weights.
* `dcf_validateEpochReplay`: Triggers a deterministic epoch transition replay.
* `dcf_queryEvmEvents`: Queries EVM-compatible consensus events.
* `dcf_validateCurrentInvariants`: Asserts that all chain invariants hold.
* `dcf_generateValidatorProposals`: Algorithmically predicts validator rewards/ejections.

### 3. `dvf_` Namespace (7 endpoints)
DVF finality tracking and vote aggregation:
* `dvf_getFinalizedHead`: Gets the highest finalized block number.
* `dvf_getFinalizedHash`: Gets the block hash of the finalized head.
* `dvf_isBlockFinalized`: Checks if a specific block height is finalized.
* `dvf_getCurrentRound`: Gets the current DVF voting round.
* `dvf_getAccumulatedWeight`: Gets the accumulated vote weight for a candidate hash.
* `dvf_getValidatorSetId`: Gets the current DVF validator set identifier.
* `dvf_getValidatorWeights`: Gets voting weight allocations for active validators.

### 4. `pos_` Namespace (4 endpoints)
Staking scores and records:
* `pos_getValidatorScore`: Gets a validator's PoS score.
* `pos_getValidatorStake`: Gets a validator's reserved stake amount.
* `pos_getSlashingCount`: Gets a validator's slashing frequency.
* `pos_getValidatorStatus`: Gets validator status (Active/Inactive/Slashed).

### 5. `poi_` Namespace (4 endpoints)
Proof of Inference performance metrics:
* `poi_getInferenceResult`: Returns the latest ML inference result hash.
* `poi_getInferenceConfidence`: Returns the confidence metric for an inference output.
* `poi_getChallengeWindow`: Returns the active block duration window for score challenges.
* `poi_getInferenceStatus`: Gets a validator's inference status.

### 6. `fork_` Namespace (2 endpoints)
Fork detection and audit tools:
* `fork_checkPeers`: Compares block histories with external peers.
* `fork_getStatus`: Returns status and warning logs of the fork detector.

### 7. `chain_` Namespace (1 endpoint)
* `chain_getChainName`: Returns the network identifier (`CBC-Chain`).

---

## Testing & Benchmarking Suite

### Running Tests
Execute unit and integration tests covering RPC validation, fork checking, and logging configurations:
```bash
# Run all tests
cargo test --all-features

# Run specific integration tests
cargo test --package cbc-node --test rpc_api_tests
cargo test --package cbc-node --test fork_detection_test
```

### Running Benchmarks
Run benchmarking tools to measure compute overhead:
```bash
# Overhead benchmarks
./target/release/cbc-node benchmark overhead --chain dev

# Hardware compatibility benchmarks
./target/release/cbc-node benchmark machine --chain dev
```

---

## Operational Monitoring & Telemetry

### Prometheus Integration
The node publishes monitoring statistics on the Prometheus port (default: `127.0.0.1:9615/metrics`).

### Health Ingest Endpoint
To fetch validator node diagnostics:
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"cbc_health","params":[],"id":1}' \
  http://localhost:9944
```
