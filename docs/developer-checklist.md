# Developer Checklist

This checklist will help you get started with CBC node development.

## Prerequisites

- [ ] Install Rust (latest stable)
- [ ] Install build dependencies:
  - Linux: `build-essential`, `clang`, `libclang-dev`
  - Windows: Visual Studio Build Tools
- [ ] Install Git
- [ ] Set up your development environment

## Initial Setup

- [ ] Clone the repository
- [ ] Install dependencies:
  ```bash
  cargo build
  ```
- [ ] Verify the build:
  ```bash
  cargo test
  ```

## Running the Node

- [ ] Start in development mode:
  ```bash
  ./target/release/cbc-node --dev
  ```
- [ ] Reset node database (if needed):
  - Linux: `./scripts/reset-node.sh`
  - Windows: `.\scripts\reset-node.ps1`

## Development Workflow

- [ ] Set up your IDE (recommended: VSCode with Rust extensions)
- [ ] Configure logging:
  ```bash
  export RUST_LOG=debug
  ```
- [ ] Learn the codebase structure:
  - `cbc-node/`: Node implementation
  - `cbc-runtime/`: Runtime logic
  - `cbc-pallets/`: Custom pallets

## Testing

- [ ] Run unit tests:
  ```bash
  cargo test
  ```
- [ ] Run specific test:
  ```bash
  cargo test test_name
  ```
- [ ] Run with logging:
  ```bash
  RUST_LOG=debug cargo test
  ```

## Common Tasks

### Adding a New Pallet

- [ ] Add dependency to `cbc-runtime/Cargo.toml`
- [ ] Configure in `cbc-runtime/src/lib.rs`
- [ ] Update genesis config if needed
- [ ] Rebuild and test

### Modifying Runtime

- [ ] Make changes in `cbc-runtime/`
- [ ] Update version in `runtime/src/lib.rs`
- [ ] Rebuild and test
- [ ] Consider runtime upgrade process

### Debugging

- [ ] Check logs with `RUST_LOG=debug`
- [ ] Use `--dev` flag for local testing
- [ ] Monitor metrics on port 9615
- [ ] Check RPC calls on port 9933

## Performance Testing

- [ ] Run with `--dev` flag
- [ ] Monitor system resources
- [ ] Check Prometheus metrics
- [ ] Test RPC performance

## Troubleshooting

### Build Issues

- [ ] Clear target directory:
  ```bash
  cargo clean
  ```
- [ ] Update Rust:
  ```bash
  rustup update
  ```
- [ ] Check dependency versions

### Runtime Issues

- [ ] Check logs
- [ ] Verify genesis configuration
- [ ] Test with fresh database
- [ ] Check system resources

### Network Issues

- [ ] Verify port availability
- [ ] Check firewall settings
- [ ] Test peer connections
- [ ] Monitor network metrics 