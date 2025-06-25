# CBC Runtime

The CBC Runtime is the core blockchain implementation for the CBC (Cerulean Blockchain Chain) network. It implements a unique consensus mechanism combining Proof of Stake (PoS) and Proof of Inference (PoI) through the Dynamic Consensus Framework (DCF).

## Project Structure

```
cbc-runtime/
├── Cargo.toml          # Project dependencies and configuration
├── build.rs           # Build script for WASM binary generation
├── src/
│   ├── apis.rs        # Runtime APIs exposed to the outside world
│   ├── benchmarks.rs  # Runtime benchmarking logic
│   ├── configs/       # Runtime configuration files
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

To build the runtime:

```bash
# Build native runtime
cargo build

# Build WASM runtime
cargo build --release --target wasm32-unknown-unknown
```

## Runtime Version

Current runtime version:
- Spec Version: 100
- Impl Version: 1
- Transaction Version: 1
- System Version: 1

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

5. **Sudo (`pallet_sudo`)**
   - Administrative operations
   - Runtime upgrades

6. **Pallet-cbc-PoI**
   - Proof of Inference implementation
   - Inference result management
   - Challenge handling

7. **Pallet-cbc-PoS**
   - Proof of Stake implementation
   - Validator stake management
   - Score tracking

8. **Pallet-cbc-Dcf**
   - Dynamic Consensus Framework
   - Combined PoS/PoI scoring
   - Governance mechanisms

## Security Considerations

1. **Block Time**: Fixed at 6 seconds
2. **Balance Parameters**: Secure existential deposit
3. **Transaction Processing**: Multiple validation checks
4. **Consensus Security**: Combined PoS/PoI mechanism
5. **Key Management**: Secure ed25519 keys

## Future Enhancements

1. **Runtime Upgrades**: Support for runtime versioning
2. **Consensus Improvements**: Additional mechanisms
3. **Transaction Processing**: Enhanced validation
4. **API Expansion**: Additional runtime APIs
5. **Configuration**: More flexible options

## Support

For support or questions about the CBC Runtime, please contact:
- GitHub Issues: https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain/issues
- Documentation: https://github.com/CAERULEAN-BYTECHAINS-PRIVATE-LIMITED/CBC-Chain/tree/main/docs

