# CBC Runtime (`cbc-runtime`)

[![Rust Compile & Build](https://github.com/bharathkumarachari/cbc-chain/actions/workflows/rust.yml/badge.svg)](https://github.com/bharathkumarachari/cbc-chain/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

<p align="center">
  <img src="../docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

The core state transition function (STF) of the **Caerulean ByteChains (CBC)** blockchain. It compiles to both a native binary and an optimized on-chain WebAssembly (WASM) blob, enabling forkless runtime upgrades and deterministic execution of the hybrid PoS/PoI consensus rules.

---

## Table of Contents
1. [Pallet Architecture](#pallet-architecture)
2. [Runtime APIs & Interfaces](#runtime-apis-&-interfaces)
3. [Configuration Constants](#configuration-constants)
4. [WASM Compilation & On-chain Upgrades](#wasm-compilation-&-on-chain-upgrades)
5. [Development, Testing & Benchmarking](#development-testing-&-benchmarking)

---

## Pallet Architecture

The runtime integrates core system pallets with custom business-logic pallets designed specifically for CBC's validator management and governance system.

```
+-----------------------------------------------------------+
|                     Runtime Executive                     |
+-----------------------------+-----------------------------+
                              |
     +------------------------+------------------------+
     | (System Infrastructure)                         | (Custom CBC Pallets)
     v                                                 v
+------------------+                             +------------------+
|   frame_system   |                             |  pallet_cbc_pos  |
| pallet_balances  |                             |  pallet_cbc_poi  |
| pallet_timestamp |                             |  pallet_cbc_dcf  |
|   pallet_sudo    |                             |  pallet_cbc_dvf  |
+------------------+                             |  cbc_governance  |
                                                 +------------------+
```

### 1. Core System Pallets
* **System (`frame_system`)**: Coordinates accounts, block header transitions, events, metadata, and execution invariants.
* **Balances (`pallet_balances`)**: Manages the native token ledger, free and reserved funds, and transfers.
* **Timestamp (`pallet_timestamp`)**: Enforces temporal consistency and exposes clock time to consensus.
* **Transaction Payment (`pallet_transaction_payment`)**: Computes transaction execution fees dynamically based on weight and length.
* **Sudo (`pallet_sudo`)**: Provides administrative authority for upgrades and testing bootstrapping.

### 2. Custom CBC Pallets
* **PoS Pallet (`pallet_cbc_pos`)**: Implements staking operations, locked deposit registries, validator application validation, performance rating updates, and reward boosts for top-performing nodes.
* **PoI Pallet (`pallet_cbc_poi`)**: Manages computational Proof of Inference workloads. Tracks ML result hash submissions, coordinates the 35-block challenge validation window, and scores nodes on inference validation speed and accuracy.
* **DCF Pallet (`pallet_cbc_dcf`)**: The Dynamic Consensus Framework core. Computes combined trust scores (65% PoS + 35% PoI), maintains active validator rosters, tracks block production metrics, and controls epoch slot boundaries.
* **DVF Pallet (`pallet_cbc_dvf`)**: The Dynamic Voting Framework core. It manages on-chain block finality transitions, aggregates validator signatures, validates voting round transitions, and stores the canonical finalized block header state.
* **Governance Pallet (`pallet_cbc_governance`)**: Implements decentralized proposal submission, voting mechanics, and execution callbacks for validator actions (e.g. slashing, ejection, additions, and removals). Decoupled from consensus internals via interfaces.

---

## Runtime APIs & Interfaces

The runtime exposes key system operations and metrics to external nodes and clients through runtime API macros.

### Standard APIs
- `Core`: Exposes standard block execution (`execute_block`) and runtime version identification functions.
- `Metadata`: Exposes compile-time metadata schema.
- `BlockBuilder`: Used by block authors to assemble inherents and apply extrinsics.
- `TaggedTransactionQueue`: Used by transaction pools to validate signatures and nonces.
- `SessionKeys`: Handles cryptographic key generation and decryption for validators.

### Custom CBC APIs
* **`PosApi`**: Fetches validator scores, stake locks, slash records, and statuses.
* **`PoiApi`**: Exposes latest inference results, confidence indexes, challenge durations, and operational states.
* **`DcfApi`**: Exposes active validator sets, current epoch numbers, author schedules, consensus weights, and epoch transition validators.
* **`DvfApi`**: Exposes DVF finalized block indices, accumulated vote weights, and round indexes.

---

## Configuration Constants

Primary parameters are defined in [configs/constants.rs](file:///home/bharat/projects/cbc-chain/cbc-runtime/src/configs/constants.rs) and [configs/mod.rs](file:///home/bharat/projects/cbc-chain/cbc-runtime/src/configs/mod.rs).

### 1. Staking & Monetary Metrics
* **Token Unit (`UNIT`)**: $1\text{ CBC} = 10^{12}\text{ Planck}$.
* **Existential Deposit**: $10^{9}\text{ Planck}$ (Minimum token quantity required to prevent state account cleanup).
* **Block Target Time**: 6 seconds (6000ms).

### 2. Consensus & Epoch Boundaries
* **Epoch Length**: 600 slots (approx. 1 hour).
* **Validator Count Bounds**: Minimum: 4 validators, Maximum: 100 validators.
* **Default Score Weights**: 65% Staking (PoS), 35% Computational (PoI).
* **Governance Quorum**: 50% validator vote requirement for proposal actions.

---

## WASM Compilation & On-chain Upgrades

Because the chain stores the WASM execution blob directly in state (`:code`), the blockchain can undergo forkless runtime upgrades.

### 1. Compiling the WASM Binary
Use standard release parameters to compile the optimized WASM image:
```bash
cargo build --release --target wasm32-unknown-unknown
```
The optimized compiled artifact will be generated at:
`target/release/wbuild/cbc-runtime/cbc_runtime.wasm`

### 2. Testing Upgrades with `try-runtime`
Simulate the on-chain upgrade process against live chain states to catch storage migration errors or logic failures before deploying:
```bash
# Build node with try-runtime capability enabled
cargo build --release --features try-runtime

# Run migration simulation
./target/release/cbc-node try-runtime \
  --runtime ./target/release/wbuild/cbc-runtime/cbc_runtime.wasm \
  on-runtime-upgrade live --uri ws://localhost:9944
```

---

## Development, Testing & Benchmarking

### 1. Unit & Integration Testing
Run the runtime test suite, including API validation and cross-pallet integration tests:
```bash
# Run unit and integration tests
cargo test -p cbc-runtime

# Run external runtime API integration tests
cargo test --test runtime_api_tests

# Run tests with stdout capturing disabled
cargo test -p cbc-runtime -- --nocapture
```

### 2. Benchmarking
Accurate extrinsic weights are computed using the runtime benchmarking features:
```bash
# Compile node with benchmarking flags
cargo build --release --features runtime-benchmarks

# Run benchmarks for pallet-cbc-dcf
./target/release/cbc-node benchmark pallet \
  --pallet pallet_cbc_dcf \
  --extrinsic "*" \
  --steps 50 \
  --repeat 20 \
  --output ./cbc-pallets/pallet-cbc-dcf/src/weights.rs
```
> [!IMPORTANT]
> Always verify that your local development setup matches the hardware specifications of the target production network to compile accurate execution weights.
