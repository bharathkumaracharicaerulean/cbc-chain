# CBC-Chain

CBC-Chain is a blockchain platform developed by Caerulean ByteChains Private Limited, implementing a unique consensus mechanism that combines Proof of Stake (PoS) and Proof of Inference (PoI) through the Dynamic Consensus Framework (DCF).

## Project Overview

CBC-Chain is a next-generation blockchain platform designed for high-performance and secure decentralized applications. The platform features:

1. **Hybrid Consensus Mechanism**
   - Combined PoS and PoI consensus
   - Dynamic Consensus Framework (DCF)
   - Validator score management
   - Epoch-based updates

2. **Key Components**
   - **cbc-node**: Blockchain node implementation
   - **cbc-runtime**: Blockchain runtime
   - **cbc-pallets**: FRAME pallets for PoS, PoI, and DCF
   - **tools**: Development and testing tools

## Project Structure

```
CBC-Chain/
├── cbc-node/              # Blockchain node implementation
├── cbc-runtime/           # Blockchain runtime
├── cbc-pallets/           # FRAME pallets
│   ├── pallet-cbc-poi/    # Proof of Inference pallet
│   ├── pallet-cbc-pos/    # Proof of Stake pallet
│   └── pallet-cbc-dcf/    # Dynamic Consensus Framework pallet
├── docs/                 # Project documentation
├── tools/                # Development tools
├── .github/              # GitHub workflows and configurations
└── target/              # Build artifacts
```

## Technical Specifications

### Consensus Parameters

- **Block Time**: 6 seconds (development) / 3 seconds (testnet)
- **Epoch Length**: 1000 blocks
- **Minimum Validators**: 2
- **Maximum Validators**: 100
- **Minimum Stake**: 1000 units
- **Finality Threshold**: 10 blocks

### Balance Parameters

- **Base Unit**: 1,000,000,000,000
- **Milli Unit**: 1,000,000,000
- **Micro Unit**: 1,000,000
- **Existential Deposit**: 1,000,000,000
- **DOLLARS**: 1,000,000,000,000

## Key Features

1. **Dynamic Consensus Framework (DCF)**
   - Combines PoS and PoI scores
   - Configurable weights for PoS and PoI
   - Validator score management
   - Epoch-based updates

2. **Proof of Stake (PoS)**
   - Validator stake management
   - Score submission and decay
   - Slashing mechanism
   - Stake bonding/unbonding

3. **Proof of Inference (PoI)**
   - Inference result management
   - Challenge mechanism
   - Score adjustments
   - Minimum confidence requirements

4. **Governance**
   - Proposal system
   - Voting mechanisms
   - Validator management
   - Configuration updates

## Building the Project

### Prerequisites

1. Rust toolchain
   ```bash
   rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu
   ```

2. Dependencies
   ```bash
   cargo install cargo-deb
   cargo install cargo-tarpaulin
   cargo install cargo-bloat
   ```

### Build Commands

```bash
# Build all components
cargo build

# Build for release
cargo build --release

# Build WASM runtime
cargo build --release --target wasm32-unknown-unknown
```

## Running the Node

```bash
# Start development node
target/release/cbc-node --dev

# Start local testnet
target/release/cbc-node --chain local
```

## Documentation

Detailed documentation is available in the `docs` directory:
- [Runtime Overview](docs/runtime-overview.md)
- [PoI Pallet](docs/cbc-PoI.md)
- [PoS Pallet](docs/cbc-PoS.md)
- [DCF Pallet](docs/dcf-pallet.md)

## Support

For support or questions about CBC-Chain, please contact:
- GitHub Issues: https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain/issues
- Documentation: https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain/tree/main/docs
- Website: https://cbytechains.com/



## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting pull requests.

## Security

If you discover a security vulnerability, please report it through our security policy.

## Acknowledgments

- Substrate framework
- FRAME pallets
- Rust development community
- Caerulean ByteChains team