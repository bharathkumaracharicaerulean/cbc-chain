# CBC Node Architecture

This document provides an overview of the CBC node architecture and its key components.

## Node Structure

The CBC node is built using Substrate framework and consists of several key components:

### Runtime Integration
- The runtime is located in `cbc-runtime/` directory
- Runtime is compiled to WebAssembly (Wasm) and linked into the node binary
- Runtime upgrades can be performed on-chain through governance

### Genesis Configuration
- Genesis state is defined in `chain-spec-plain.json`
- Raw chain spec is generated in `chain-spec-raw.json`
- Custom genesis configurations can be added in the runtime's `GenesisConfig`

### RPC System
- Custom RPCs are exposed through the `rpc` module in the node
- RPC methods are defined in the runtime's `rpc.rs`
- JSON-RPC API is available on the default port 9933
- WebSocket RPC is available on port 9944

## Adding New Pallets

To add a new pallet to the node:

1. Add the pallet dependency to `cbc-runtime/Cargo.toml`
2. Configure the pallet in `cbc-runtime/src/lib.rs`:
   - Add the pallet to the `construct_runtime!` macro
   - Configure the pallet's parameters in the `Runtime` struct
3. Update the runtime's `GenesisConfig` if the pallet requires initial state
4. Rebuild the runtime and node

## Development Workflow

### Local Development
- Use `--dev` flag for local development
- Use `--tmp` flag to start with a fresh database
- Use `scripts/reset-node.sh` (Linux) or `scripts/reset-node.ps1` (Windows) to reset the node

### Testing
- Unit tests are in the `tests` directory
- Integration tests can be added in `cbc-node/tests`
- Use `cargo test` to run all tests

## Performance Considerations

- The node uses RocksDB for storage
- Performance metrics are available through Prometheus on port 9615
- Logging can be configured through RUST_LOG environment variable

## Common Issues

1. Database corruption:
   - Use the reset scripts to clear the database
   - Check disk space and permissions

2. Build issues:
   - Ensure all dependencies are installed
   - Check Rust toolchain version
   - Clear target directory if needed

3. Runtime upgrade failures:
   - Verify runtime version compatibility
   - Check governance parameters
   - Ensure sufficient funds for upgrade 