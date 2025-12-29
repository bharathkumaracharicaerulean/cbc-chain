# Developer Onboarding Guide

Welcome to CBC Chain development! This guide will help you set up your development environment and get started contributing to the project.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Dependency Installation](#dependency-installation)
4. [Building the Project](#building-the-project)
5. [Running Tests](#running-tests)
6. [Common Build Errors](#common-build-errors)
7. [Development Workflow](#development-workflow)
8. [Cargo Tree Snapshots](#cargo-tree-snapshots)

## Prerequisites

Before starting development, ensure you have the following installed:

### Required Software

- **Operating System**: Linux (Ubuntu 20.04+), macOS (10.15+), or Windows 10+ with WSL2
- **Rust**: Version 1.70.0 or later (stable toolchain)
- **Git**: Version 2.20 or later
- **Node.js**: Version 16.0 or later (for frontend tools)
- **Python**: Version 3.8 or later (for some build tools)

### Hardware Requirements

- **RAM**: Minimum 8GB, recommended 16GB+
- **Storage**: At least 20GB free space for dependencies and build artifacts
- **CPU**: Multi-core processor recommended for faster compilation

## Environment Setup

### 1. Install Rust

Install Rust using rustup (recommended method):

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Configure Rust Toolchain

Set up the required Rust toolchain and targets:

```bash
# Install stable toolchain (if not already default)
rustup toolchain install stable

# Add WebAssembly target (required for Substrate)
rustup target add wasm32-unknown-unknown

# Install nightly toolchain (for some advanced features)
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# Verify targets are installed
rustup target list --installed
```

### 3. Install Additional Tools

Install essential development tools:

```bash
# Install cargo-expand for macro debugging
cargo install cargo-expand

# Install cargo-audit for security auditing
cargo install cargo-audit

# Install cargo-outdated for dependency management
cargo install cargo-outdated

# Install substrate-contracts-node (if working with smart contracts)
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git

# Install subkey for key generation
cargo install --force subkey --git https://github.com/paritytech/polkadot-sdk
```

### 4. System Dependencies

Install system-level dependencies based on your operating system:

#### Ubuntu/Debian

```bash
# Update package list
sudo apt update

# Install build essentials
sudo apt install -y build-essential

# Install required libraries
sudo apt install -y \
    git \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    make \
    protobuf-compiler \
    pkg-config

# Install additional tools
sudo apt install -y \
    cmake \
    git \
    libclang-dev \
    libssl-dev \
    pkg-config
```

#### macOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install required packages
brew install openssl cmake llvm protobuf

# Install Xcode command line tools
xcode-select --install
```

#### Windows (WSL2)

```bash
# Update WSL2 Ubuntu
sudo apt update && sudo apt upgrade -y

# Follow Ubuntu instructions above
# Additionally, ensure WSL2 has sufficient memory allocated (8GB+)
```

## Dependency Installation

### 1. Clone the Repository

```bash
# Clone the CBC Chain repository
git clone https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain.git
cd CBC-Chain

# Verify repository structure
ls -la
```

### 2. Pinned Dependency Versions

The project uses workspace dependencies with pinned versions for reproducible builds. Key versions include:

```toml
# Core Substrate dependencies (from Cargo.toml)
sp-runtime = "41.1.0"
sp-core = "36.1.0"
sp-api = "36.0.1"
frame-support = "40.1.0"
frame-system = "40.1.0"

# Consensus and networking
sc-service = "0.50.0"
sc-client-api = "39.0.0"
sc-consensus = "0.48.0"

# RPC and JSON handling
jsonrpsee = "0.24.9"
serde_json = "1.0.140"

# Async runtime
futures = "0.3.31"
tokio = "1.0"

# CLI and utilities
clap = "4.5.13"
```

### 3. Environment Variables

Set up required environment variables:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export RUST_LOG=info
export RUST_BACKTRACE=1

# For development builds (faster compilation)
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C target-cpu=native"

# For WebAssembly builds
export WASM_BUILD_TYPE=release

# Reload your shell or source the profile
source ~/.bashrc  # or ~/.zshrc
```

## Building the Project

### 1. Initial Build

Perform the first build to download and compile all dependencies:

```bash
# Clean build (recommended for first time)
cargo clean

# Build all workspace members
cargo build

# Build in release mode (optimized, slower compilation)
cargo build --release
```

**Note**: The initial build may take 30-60 minutes depending on your hardware.

### 2. Build Individual Components

Build specific components for faster iteration:

```bash
# Build only the node
cargo build -p cbc-node

# Build only the runtime
cargo build -p cbc-runtime

# Build specific pallet
cargo build -p pallet-cbc-dcf
```

### 3. WebAssembly Runtime Build

The runtime is compiled to WebAssembly for on-chain execution:

```bash
# Build runtime with WASM
cd cbc-runtime
cargo build --release

# Verify WASM artifact is created
ls -la target/release/wbuild/cbc-runtime/
```

## Running Tests

### 1. Unit Tests

Run unit tests for all components:

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p pallet-cbc-dcf

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_validator_registration
```

### 2. Integration Tests

Run integration tests:

```bash
# Run integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test runtime_api_tests
```

### 3. Property-Based Tests

Run property-based tests (if implemented):

```bash
# Run property tests with more iterations
PROPTEST_CASES=1000 cargo test

# Run specific property test
cargo test property_test_rpc_data_consistency
```

### 4. Benchmarks

Run runtime benchmarks:

```bash
# Run benchmarks for all pallets
cargo test --features runtime-benchmarks

# Run benchmarks for specific pallet
cargo test --features runtime-benchmarks -p pallet-cbc-dcf
```

## Common Build Errors

### 1. WebAssembly Target Missing

**Error**: `error: the 'wasm32-unknown-unknown' target may not be installed`

**Solution**:
```bash
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### 2. Protobuf Compiler Missing

**Error**: `protoc: program not found`

**Solution**:
```bash
# Ubuntu/Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf

# Verify installation
protoc --version
```

### 3. OpenSSL Development Libraries Missing

**Error**: `failed to run custom build command for 'openssl-sys'`

**Solution**:
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### 4. Clang/LLVM Missing

**Error**: `Unable to find libclang`

**Solution**:
```bash
# Ubuntu/Debian
sudo apt install clang libclang-dev llvm

# macOS
brew install llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
```

### 5. Memory Issues During Compilation

**Error**: `signal: 9, SIGKILL: kill` or out of memory errors

**Solution**:
```bash
# Reduce parallel compilation jobs
export CARGO_BUILD_JOBS=2

# Or build with limited parallelism
cargo build -j 2

# For WSL2, increase memory allocation in .wslconfig
```

### 6. Cargo Lock Conflicts

**Error**: `Cargo.lock` conflicts or dependency resolution issues

**Solution**:
```bash
# Remove lock file and rebuild
rm Cargo.lock
cargo build

# Or update dependencies
cargo update
```

### 7. Substrate Version Conflicts

**Error**: Version conflicts between Substrate crates

**Solution**:
```bash
# Ensure all Substrate dependencies use the same version
cargo tree | grep -E "(sp-|sc-|frame-|pallet-)"

# Update workspace dependencies if needed
cargo update -p sp-runtime
```

## Development Workflow

### 1. Code Organization

The project follows Substrate's standard structure:

```
CBC-Chain/
├── cbc-node/                 # Node implementation
│   ├── src/
│   │   ├── main.rs          # Node entry point
│   │   ├── service.rs       # Node service configuration
│   │   ├── rpc.rs           # RPC endpoint implementations
│   │   └── cbc-consensus/   # Custom consensus engine
├── cbc-runtime/             # Runtime implementation
│   ├── src/
│   │   ├── lib.rs           # Runtime configuration
│   │   └── configs/         # Pallet configurations
├── cbc-pallets/             # Custom pallets
│   ├── pallet-cbc-dcf/      # DCF consensus pallet
│   ├── pallet-cbc-pos/      # Proof of Stake pallet
│   └── pallet-cbc-poi/      # Proof of Inference pallet
└── tools/                   # Utility tools
```

### 2. Development Commands

Common commands for development:

```bash
# Start development node
cargo run --release -- --dev

# Start node with custom chain spec
cargo run --release -- --chain=local

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Clean build artifacts
cargo clean
```

### 3. Testing Strategy

Follow this testing approach:

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **Property Tests**: Test invariants across random inputs
4. **End-to-End Tests**: Test complete workflows

### 4. Code Quality

Maintain code quality with these tools:

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features

# Check for security vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Cargo Tree Snapshots

### Current Dependency Tree (Top Level)

```
cbc-node v0.1.0
├── cbc-runtime v0.1.0 (workspace)
├── cbc-consensus v0.1.0 (workspace)
├── pallet-cbc-dcf v0.1.0 (workspace)
├── pallet-cbc-pos v0.1.0 (workspace)
├── clap v4.5.13
├── futures v0.3.31
├── jsonrpsee v0.24.9
├── sc-service v0.50.0
├── sc-client-api v39.0.0
├── sp-runtime v41.1.0
└── tokio v1.0
```

### Runtime Dependencies

```
cbc-runtime v0.1.0
├── frame-executive v40.0.1
├── frame-support v40.1.0
├── frame-system v40.1.0
├── pallet-balances v41.1.0
├── pallet-cbc-dcf v0.1.0 (workspace)
├── pallet-cbc-pos v0.1.0 (workspace)
├── pallet-cbc-poi v0.1.0 (workspace)
├── sp-api v36.0.1
├── sp-core v36.1.0
└── sp-runtime v41.1.0
```

### Pallet Dependencies

```
pallet-cbc-dcf v0.1.0
├── frame-support v40.1.0
├── frame-system v40.1.0
├── pallet-cbc-pos v0.1.0 (workspace)
├── pallet-cbc-poi v0.1.0 (workspace)
├── sp-runtime v41.1.0
└── sp-std v14.0.0
```

### Key Version Constraints

- **Substrate Framework**: All `sp-*`, `sc-*`, and `frame-*` crates use compatible versions
- **Async Runtime**: `tokio` v1.0+ with `futures` v0.3.31
- **Serialization**: `parity-scale-codec` v3.7.5 with `scale-info` v2.11.6
- **JSON-RPC**: `jsonrpsee` v0.24.9 for all RPC functionality
- **CLI**: `clap` v4.5.13 for command-line interface

### Dependency Update Strategy

1. **Substrate Updates**: Update all Substrate crates together to maintain compatibility
2. **Security Updates**: Regularly run `cargo audit` and update vulnerable dependencies
3. **Version Pinning**: Pin exact versions in workspace for reproducible builds
4. **Testing**: Run full test suite after any dependency updates

## Next Steps

After completing the setup:

1. **Read the Architecture Guide**: Understand the system design in `docs/node-architecture.md`
2. **Explore the Code**: Start with `cbc-node/src/main.rs` and `cbc-runtime/src/lib.rs`
3. **Run the Node**: Start a development node and explore the RPC endpoints
4. **Write Tests**: Add tests for any new functionality you develop
5. **Join the Community**: Connect with other developers on Discord or GitHub

## Getting Help

If you encounter issues:

1. **Check Common Errors**: Review the common build errors section above
2. **Search Issues**: Look for similar issues in the GitHub repository
3. **Ask for Help**: Create a new issue with detailed error information
4. **Community Support**: Join the developer Discord for real-time help

## Contributing

Ready to contribute? Check out:

- `CONTRIBUTING.md` for contribution guidelines
- Open issues labeled "good first issue"
- The project roadmap for upcoming features
- Code review process and standards

Welcome to the CBC Chain development community!