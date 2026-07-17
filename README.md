# CBC Chain - Caerulean ByteChains

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

<p align="center">
  <img src="docs/assets/logo.png" alt="CBC Logo" width="200" />
</p>

A production-ready blockchain implementing a sophisticated **Dynamic Consensus Framework (DCF)** that combines **Proof of Stake (PoS)** and **Proof of Inference (PoI)** for advanced validator selection and network security, coupled with the **Dynamic Voting Framework (DVF)** for BFT block finalization.

---

## Table of Contents
1. [Overview](#overview)
2. [Core Features](#core-features)
3. [Architecture & Repository Organization](#architecture-&-repository-organization)
4. [Consensus & Scoring Formulation](#consensus-&-scoring-formulation)
5. [The 35 Specialized RPC API Catalog](#the-35-specialized-rpc-api-catalog)
6. [Quick Start & Installation](#quick-start-&-installation)
7. [Testing & Verification Suite](#testing-&-verification-suite)
8. [Telemetry & Prometheus Monitoring](#telemetry-&-prometheus-monitoring)
9. [Development Roadmap](#development-roadmap)
10. [Troubleshooting & FAQ](#troubleshooting-&-faq)

---

## Overview

CBC Chain represents a next-generation blockchain platform that extends traditional PoS consensus by incorporating **Proof of Inference (PoI)**. Validators are selected and rewarded based on both their economic stake (PoS) and their computational contributions to AI/ML inference tasks (PoI). 

The system features real-time performance tracking, BFT block finalization through signature justification aggregation (DVF), decoupled governance architecture, structured logging with deduplication, and production-ready Grafana dashboards.

---

## Core Features

- **Dynamic Consensus Engine (DCF)**: Adaptive validator selection featuring real-time rating updates, score decay rules, and slot scheduling.
- **Proof of Inference (PoI) Workload**: On-chain verification of AI model execution confidence scores with a 35-block score dispute/challenge window.
- **BFT Finality Engine (DVF)**: High-performance signature voting gossip protocol that triggers block finalization when $\ge 67\%$ weight quorum is reached.
- **Decoupled Governance**: Separate governance module managing proposal workflows, validator additions, slashings, and ejections via strict interfaces.
- **High-Density API Layer**: Exposes 35 specialized endpoints across 7 custom RPC namespaces.
- **Security & Safety Rails**: Rate limiting, access control for unsafe methods, key allowlists, and validator cooling-off bounds.
- **Comprehensive Monitoring**: Built-in Prometheus metrics export with Grafana dashboard provisioning templates.

---

## Architecture & Repository Organization

### System Architecture Layering
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CBC Chain Production Architecture                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  Application Layer                                                          │
│  ├── RPC APIs (35 endpoints)     ├── CLI Tools & Utilities                  │
│  ├── Monitoring (Grafana)        ├── Fork Detection System                  │
│  └── External Integrations       └── Development Tools                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Runtime Layer (WASM + Native)                                              │
│  ├── pallet-cbc-dcf    (Dynamic Consensus Framework)                        │
│  ├── pallet-cbc-pos    (Proof of Stake Staking Registry)                    │
│  ├── pallet-cbc-poi    (Proof of Inference & Challenge Window)              │
│  ├── pallet-cbc-dvf    (Dynamic Voting finality tracker)                    │
│  ├── cbc-governance    (Decoupled Governance mechanics)                     │
│  └── Runtime APIs      (Specialized query interfaces)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  Consensus Layer                                                            │
│  ├── DCF Consensus Engine        ├── Advanced Block Import Pipeline         │
│  ├── Epoch Rotation              ├── Inherent Data Providers                │
│  ├── DVF Gossip Voting           ├── Justification Finality Engine          │
│  ├── Block Tracker               └── Comprehensive Metrics System           │
├─────────────────────────────────────────────────────────────────────────────┤
│  Node Layer                                                                 │
│  ├── CBC Node (Multi-mode)       ├── Advanced Logging System                │
│  ├── Service Configuration       ├── Security & Rate Limiting               │
│  ├── Network Protocol            ├── Chain Specifications (4 modes)         │
│  └── Client Services             └── Benchmarking Infrastructure            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Infrastructure Layer                                                       │
│  ├── Monitoring (Prometheus)     ├── Testing Infrastructure                 │
│  ├── Docker Compose Setup        ├── Build & Deployment Scripts             │
│  └── Documentation System        └── Development Tools                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Folder Layout
* **`cbc-node/`**: Core blockchain client, networking threads, 35 RPC server endpoints, CLI command controllers, and custom logger.
  * **`cbc-node/src/cbc-consensus/`**: The consensus engine including the DCF execution loop, BVF gossip, vote pool aggregation, and lifecycle tracer.
  * **`cbc-node/tests/`**: Integration and CLI verification suite.
* **`cbc-runtime/`**: Blockchain State Transition Function (STF) executing on-chain transactions and WASM exports.
* **`cbc-pallets/`**: Custom FRAME business logic modules.
  * **`pallet-cbc-dcf`**: Staking and inference score aggregation logic.
  * **`pallet-cbc-pos`**: Staking registration, token bonding, and slashing metrics.
  * **`pallet-cbc-poi`**: Inference result registration and dispute resolution.
  * **`pallet-cbc-dvf`**: BFT-style vote aggregation and finalized head caching.
  * **`pallet-cbc-governance`**: Decoupled governance proposal lifecycle.
* **`monitoring/`**: Docker configs for Prometheus metrics scraping and Grafana dashboards.
* **`docs/`**: Setup guides and tech specifications.

---

## Consensus & Scoring Formulation

### Validator Trust Scoring
At epoch boundaries, validators are evaluated and sorted based on their combined score:
$$\text{Trust Score} = \frac{(\text{PoS Score} \times \text{PoS Weight}) + (\text{PoI Score} \times \text{PoI Weight})}{\text{Precision Divisor}}$$

By default, PoS is weighted at **65%** and PoI at **35%**, dynamically configurable on-chain.

```rust
// Trust score calculation logic
fn calculate_trust_score(validator: &ValidatorProfile) -> TrustScore {
    let pos_score = validator.pos_performance_score;
    let poi_score = validator.poi_performance_score;
    let pos_weight = 65; // 65% weight for PoS
    let poi_weight = 35; // 35% weight for PoI
    
    let combined_score = (pos_score * pos_weight + poi_score * poi_weight) / 100;
    
    TrustScore {
        total: combined_score,
        pos_component: pos_score,
        poi_component: poi_score,
        pos_weight,
        poi_weight,
    }
}
```

---

## The 35 Specialized RPC API Catalog

CBC Chain implements 35 custom RPC endpoints to expose consensus stats, validator profiles, and node diagnostics:

### 1. `cbc_` Namespace (9 endpoints)
System telemetry and validator statistics:
* `cbc_getCurrentEpoch`: Get current epoch number.
* `cbc_getValidatorProfile`: Get comprehensive validator profile.
* `cbc_getTrustScore`: Get trust score breakdown.
* `cbc_listValidators`: List active validator account addresses.
* `cbc_getStatus`: Get general network status.
* `cbc_describe`: List descriptions of all active RPC endpoints.
* `cbc_health`: Get diagnostic health codes.
* `cbc_getBlockAuthoringStats`: Get blocks authored/missed stats for a validator.
* `cbc_getAllBlockAuthoringStats`: Get authoring stats for all validators.

### 2. `dcf_` Namespace (8 endpoints)
DCF consensus and parameters:
* `dcf_getCurrentAuthor`: Get active slot block author.
* `dcf_getExpectedAuthor`: Get expected author for a block number.
* `dcf_getValidatorScores`: Returns all active validator scores.
* `dcf_getConsensusWeights`: Returns PoS vs. PoI weights.
* `dcf_validateEpochReplay`: Triggers an epoch transition replay check.
* `dcf_queryEvmEvents`: Queries EVM-compatible consensus events.
* `dcf_validateCurrentInvariants`: Asserts that all chain invariants hold.
* `dcf_generateValidatorProposals`: Predicts validator rewards/ejections.

### 3. `dvf_` Namespace (7 endpoints)
DVF finality tracking:
* `dvf_getFinalizedHead`: Highest finalized block number.
* `dvf_getFinalizedHash`: Block hash of finalized head.
* `dvf_isBlockFinalized`: Checks finality status of a block.
* `dvf_getCurrentRound`: Active finalization voting round.
* `dvf_getAccumulatedWeight`: Voting weight accumulated by a block.
* `dvf_getValidatorSetId`: Active validator set identifier.
* `dvf_getValidatorWeights`: Voting weight allocation for validators.

### 4. `pos_` Namespace (4 endpoints)
* Staking metadata queries: `pos_getValidatorScore`, `pos_getValidatorStake`, `pos_getSlashingCount`, `pos_getValidatorStatus`.

### 5. `poi_` Namespace (4 endpoints)
* Inference tracking: `poi_getInferenceResult`, `poi_getInferenceConfidence`, `poi_getChallengeWindow`, `poi_getInferenceStatus`.

### 6. `fork_` Namespace (2 endpoints)
* Fork detection tools: `fork_checkPeers`, `fork_getStatus`.

### 7. `chain_` Namespace (1 endpoint)
* `chain_getChainName`: Returns the network name (`CBC-Chain`).

---

## Quick Start & Installation

### Prerequisites
* **Rust**: Stable Rust compiler toolchain with `wasm32-unknown-unknown` target.
* **C++ Compiler**: Clang (v11+) and GCC.
* **System Libraries**: OpenSSL (`libssl-dev`), CMake, `pkg-config`, and `make`.

### Building the Node
```bash
# Clone the repository
git clone https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain.git
cd CBC-Chain

# Compile the project
cargo build --release
```

### Running Node Configurations
```bash
# Development mode (single validator - Alice)
./target/release/cbc-node --dev

# Local validator testnet
./target/release/cbc-node --chain local

# Production configurations with rate limiting & CBC extensions
./target/release/cbc-node \
  --chain local \
  --cbc-mode production \
  --enable-cbc-extensions \
  --rpc-rate-limit-window 60 \
  --rpc-rate-limit-requests 120
```

---

## Testing & Verification Suite

The repository contains unit tests, integration scenarios, and multi-node voting simulations:

```bash
# Run all workspace unit and integration tests
cargo test --all-features

# Run custom consensus tests
cargo test -p cbc-consensus

# Run RPC API tests
cargo test --package cbc-node --test rpc_api_tests
```

---

## Telemetry & Prometheus Monitoring

Nodes expose consensus performance and lifecycle metrics (e.g., `cbc_lifecycle_step_total`, `cbc_consensus_active_validators`) on target port `127.0.0.1:9615/metrics`.

### Starting the Dashboard Stack
1. Start Grafana & Prometheus containers:
   ```bash
   cd monitoring
   docker-compose up -d
   ```
2. Navigate to `http://localhost:3000` (User: `admin` / Pass: `admin`) to view validator health, sync rates, block times, and fork warnings.

---

## Development Roadmap

## Troubleshooting & FAQ

#### Q: The node logs periodically show `Idle (0 peers)`. Has a chain fork occurred?
**No.** The `Idle (X peers)` log indicates the **block replication sync substream** has completed synchronization to the tip of the canonical chain and closed redundant sync streams. Consensus voting, block propagation, and BFT gossip continue uninterrupted on separate dedicated libp2p channels.

#### Q: How is validator uptime calculated?
Uptime tracks block proposal slots completed versus expected. Uptime is logged as `N/A` (returning `None`) if data is temporarily unavailable. New validators are automatically granted a grace period with 100% starting uptime to prevent false penalization.

---

**CBC Chain** - Next-generation blockchain with hybrid PoS/PoI consensus  
*Built by Caerulean ByteChains Private Limited*
