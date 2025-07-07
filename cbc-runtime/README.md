# CBC Runtime

The CBC Runtime is the core blockchain implementation for the CBC (Caerulean ByteChains). It implements a unique consensus mechanism combining Proof of Stake (PoS) and Proof of Inference (PoI) through the Dynamic Consensus Framework (DCF).

## Project Structure

```
cbc-runtime/
├── Cargo.toml          # Project dependencies and configuration
├── build.rs           # Build script for WASM binary generation
├── src/
│   ├── apis.rs        # Runtime APIs exposed to the outside world
│   ├── benchmarks.rs  # Runtime benchmarking logic
│   ├── configs/       # Runtime configuration files
│   │   ├── constants.rs  # Runtime constants and parameters
│   │   ├── mod.rs     # Configuration module definitions
│   │   └── weights.rs # Weight calculations for extrinsics
│   ├── genesis_config_presets.rs # Genesis block configuration presets
│   └── lib.rs         # Main runtime implementation
└── README.md          # This file
```

## Key Files

### `Cargo.toml`
- Project configuration file
- Defines dependencies for the runtime
- Specifies build features and target specifications
- Includes configuration for WASM compilation

### `build.rs`
- Build script for the runtime
- Generates WASM binary for the runtime
- Handles compilation flags and feature toggles
- Ensures proper WASM optimization

### `src/lib.rs`
- Main runtime implementation
- Defines runtime version and constants
- Configures FRAME pallets
- Sets up transaction processing
- Implements runtime executive
- Defines opaque types for block and extrinsics

### `src/apis.rs`
- Runtime APIs exposed to external systems
- Provides access to runtime state and functionality
- Includes PoS, PoI, and DCF APIs
- Used by tools like Polkadot-JS Apps

### `src/benchmarks.rs`
- Benchmarking logic for runtime performance
- Measures execution time of runtime operations
- Used for optimizing runtime performance
- Provides weight calculations for transactions

### `src/configs/`
- Contains runtime configuration files
- Defines constants and parameters
- Includes genesis configuration
- Handles runtime parameters

### `src/genesis_config_presets.rs`
- Genesis block configuration presets
- Defines initial validator set
- Sets initial balances
- Configures PoS/PoI parameters
- Sets up initial DCF configuration

## Runtime Features

### Consensus Mechanism
- Combined PoS and PoI consensus
- Dynamic Consensus Framework (DCF)
- Validator score management
- Epoch-based updates

### Transaction Processing
- Weight-based fee calculation
- Multiple transaction checks
- Secure transaction signing
- Transaction validation

### Runtime APIs
- Validator management
- Score tracking
- Epoch handling
- Block authorship
- Governance operations

### Security Features
- Secure key management
- Block time enforcement
- Balance protection
- Transaction validation
- Consensus security

## Building the Runtime

### Basic Build
```bash
# Build native runtime
cargo build

# Build WASM runtime
cargo build --release --target wasm32-unknown-unknown
```

### Feature Flags

- `runtime-benchmarks`: Enables runtime benchmarking
  ```bash
  cargo build --features runtime-benchmarks
  ```

- `try-runtime`: Enables try-runtime for testing upgrades
  ```bash
  cargo build --features try-runtime
  ```

- `on-chain-release-build`: Prepares runtime for on-chain release
  ```bash
  cargo build --features on-chain-release-build
  ```

### WASM Build with All Features
```bash
cargo build --release --features runtime-benchmarks,try-runtime --target wasm32-unknown-unknown
```

## Runtime Version

The runtime version is defined in `src/lib.rs`. To check the current version:

```bash
grep -A 5 "pub const VERSION" src/lib.rs
```

Or build the node and check the version:

```bash
cargo run -- --version
```

## Block Parameters

- Block Time: 6 seconds
- Slot Duration: 6 seconds
- Block Hash Count: 2400 blocks

## Balance Parameters

- Base Unit: 1,000,000,000,000
- Milli Unit: 1,000,000,000
- Micro Unit: 1,000,000
- Existential Deposit: 1,000,000,000
- DOLLARS: 1,000,000,000,000

## Pallets

The runtime includes the following pallets:

### Core Pallets
1. **System (`frame_system`)**
   - Basic blockchain system management
   - Accounts, transactions, block execution

2. **Timestamp (`pallet_timestamp`)**
   - Block timestamp management
   - Time-dependent logic

3. **Balances (`pallet_balances`)**
   - Account balance management
   - Token transfers
   - Existential deposits

4. **Transaction Payment (`pallet_transaction_payment`)**
   - Transaction fee handling
   - Weight-based fee calculation
   - RPC API for fee queries

5. **Sudo (`pallet_sudo`)**
   - Administrative operations
   - Runtime upgrades
   - Emergency interventions

### Custom Pallets
6. **Pallet-cbc-PoI**
   - Proof of Inference implementation
   - Inference result management
   - Challenge handling
   - On-chain verification

7. **Pallet-cbc-PoS**
   - Proof of Stake implementation
   - Validator stake management
   - Score tracking
   - Epoch-based rewards

8. **Pallet-cbc-Dcf**
   - Dynamic Consensus Framework
   - Combined PoS/PoI scoring
   - Governance mechanisms
   - Consensus parameter adjustments

## Security Considerations

1. **Block Time**: Fixed at 6 seconds
2. **Balance Parameters**: Secure existential deposit
3. **Transaction Processing**: Multiple validation checks
4. **Consensus Security**: Combined PoS/PoI mechanism
5. **Key Management**: Secure ed25519 keys

## Development

### Testing

Run tests with:
```bash
cargo test
```

### Benchmarking

To run benchmarks:
```bash
cargo test --features runtime-benchmarks --bench benchmarking
```

### Try Runtime

For testing runtime upgrades:
```bash
cargo test --features try-runtime
```



