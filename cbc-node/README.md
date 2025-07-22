# CBC Node

A Substrate-based blockchain node implementing the CBC (Caerulean ByteChains) protocol, featuring a hybrid consensus mechanism combining Proof of Stake (PoS) and Proof of Inference (PoI).

## Features

- **Hybrid Consensus**: Combines PoS and PoI for secure and efficient block validation
- **Modular Architecture**: Built on Substrate for maximum flexibility and upgradability
- **High Performance**: Optimized for high transaction throughput and low latency
- **RPC Support**: JSON-RPC interface for interacting with the blockchain
- **Telemetry**: Built-in monitoring and metrics collection
- **Benchmarking**: Tools for performance testing and optimization

## Prerequisites

- Rust (latest stable version)
- LLVM and Clang
- OpenSSL
- Git

## Installation

1. Clone the repository:
```bash
git clone https://github.com/your-org/cbc-chain.git
cd cbc-chain/cbc-node
```

2. Build the node:
```bash
cargo build --release
```

## Usage

### Start a Development Node

```bash
./target/release/cbc-node --dev
```

### Connect to a Testnet

```bash
./target/release/cbc-node --chain testnet
```

### Available CLI Options

```
USAGE:
    cbc-node [OPTIONS]

OPTIONS:
    --dev               Run in development mode
    --chain <CHAIN>     Specify the chain specification (dev, local, or custom)
    --base-path <PATH>  Specify custom base path
    --port <PORT>       P2P port (default: 30333)
    --rpc-port <PORT>   HTTP-RPC port (default: 9944)
    --ws-port <PORT>    WebSockets port (default: 9944)
    --telemetry-url <URL>  Send telemetry to URL
    --log <LOG>         Set custom logging filter
    --help              Print help information
```

## Architecture

### Core Components

- **Consensus Layer**: Implements the hybrid PoS/PoI consensus mechanism
- **Runtime**: Contains the blockchain's business logic and state transition function
- **Networking**: Handles peer-to-peer communication and block propagation
- **RPC Server**: Provides JSON-RPC interface for client applications
- **CLI**: Command-line interface for node management

### Directory Structure

```
cbc-node/
├── src/
│   ├── cbc-consensus/          # Consensus implementation
│   │   ├── src/
│   │   │   ├── author_selection.rs  # Validator selection logic
│   │   │   ├── dcf.rs               # Dynamic Consensus Framework core
│   │   │   ├── epoch_manager.rs      # Epoch transition handling
│   │   │   ├── error.rs              # Consensus error types
│   │   │   ├── finality.rs           # Block finalization logic
│   │   │   ├── import_queue.rs       # Block import queue implementation
│   │   │   ├── lib.rs                # Module exports and core types
│   │   │   ├── metrics.rs            # Metrics collection
│   │   │   ├── proposer_factory.rs   # Block proposer creation
│   │   │   ├── types.rs              # Common types and traits
│   │   │   └── validator_set.rs      # Validator set management
│   │   └── Cargo.toml
│   │
│   ├── chain_spec.rs   # Chain specification
│   ├── cli.rs          # Command-line interface
│   ├── command.rs      # CLI command handlers
│   ├── rpc.rs          # RPC server implementation
│   ├── service.rs      # Node service setup
│   └── main.rs         # Entry point
└── Cargo.toml          # Project manifest
```

## Development

### Building for Production

```bash
cargo build --release
```

### Running Tests

```bash
cargo test --all-features
```

### Generating Documentation

```bash
cargo doc --open
```

## Configuration

### Chain Specification

Chain specifications define the genesis state and network parameters. They can be found in `src/chain_spec.rs`.

### Runtime Configuration

The node's runtime configuration is defined in the `cbc-runtime` crate.

## Monitoring

The node exposes Prometheus metrics on the configured metrics endpoint (default: `127.0.0.1:9615`).

