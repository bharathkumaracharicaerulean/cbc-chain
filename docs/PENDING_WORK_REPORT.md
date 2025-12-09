# CBC Project - Pending Work Report

---

## 1. RPC Endpoints & API

### 1.1 PoS RPC Endpoints

Status: Runtime API exists, but RPC handler layer is missing

Implemented Components:
- Storage layer in `pallet-cbc-pos/src/lib.rs` (ValidatorScores, Stake, SlashingCount)
- Runtime API declaration in `pallet-cbc-pos/src/lib.rs` (PosApi trait with methods)
- Runtime API implementation in `cbc-runtime/src/apis.rs` (reads from PoS pallet storage)

Missing Components:
- RPC handler in `cbc-node/src/rpc.rs` to expose endpoints externally
- RPC trait definition (e.g., `#[rpc(server)] pub trait PosRpcApi`)
- RPC implementation struct (e.g., `PosRpcApiImpl`)
- Registration in `create_full()` function to merge PoS RPC module

Required Endpoints:
- `pos_getValidatorScore` - Get validator's performance score
- `pos_getValidatorStake` - Get validator's staked amount
- `pos_getSlashingCount` - Get number of times validator was slashed
- `pos_getValidatorStatus` - Get validator's active/inactive status

Note: DCF pallet accesses PoS data directly via pallet-to-pallet calls. These RPC endpoints are only needed for external clients such as explorers and CLI tools.

### 1.2 PoI RPC Endpoints

Status: Runtime API exists, but RPC handler layer is missing

Implemented Components:
- Storage layer in `pallet-cbc-poi/src/lib.rs` (inference results and challenges)
- Runtime API declaration in `pallet-cbc-poi/src/lib.rs` (PoiApi trait)
- Runtime API implementation in `cbc-runtime/src/apis.rs`

Missing Components:
- RPC handler in `cbc-node/src/rpc.rs` to expose endpoints externally
- RPC trait definition (e.g., `#[rpc(server)] pub trait PoiRpcApi`)
- RPC implementation struct (e.g., `PoiRpcApiImpl`)
- Registration in `create_full()` function to merge PoI RPC module

Required Endpoints:
- `poi_getInferenceResult` - Get validator's inference result and confidence
- `poi_getInferenceConfidence` - Get confidence score for inference
- `poi_getChallengeWindow` - Get current challenge window
- `poi_getInferenceStatus` - Get inference submission status

### 1.3 DCF RPC Endpoints

Status: Extensive Runtime API exists (50+ methods), but RPC handler layer is missing

Implemented Components:
- Comprehensive DCF pallet in `pallet-cbc-dcf/src/lib.rs` with all functionality
- Runtime API declaration in `pallet-cbc-dcf/src/lib.rs` (DcfApi trait with 50+ methods)
- Full Runtime API implementation in `cbc-runtime/src/apis.rs`

Missing Components:
- RPC handler in `cbc-node/src/rpc.rs` to expose DCF endpoints externally
- RPC trait definition (e.g., `#[rpc(server)] pub trait DcfRpcApi`)
- RPC implementation struct (e.g., `DcfRpcApiImpl`)
- Registration in `create_full()` function to merge DCF RPC module
- Connection to `--enable-cbc-extensions` CLI flag (flag exists but not used for DCF RPCs)

Key Endpoints Required:
- `dcf_getCurrentAuthor()` - Get current block author
- `dcf_getExpectedAuthor(blockNumber)` - Get expected author for a block
- `dcf_getValidatorScores()` - Get all validator scores
- `dcf_getConsensusWeights()` - Get PoS/PoI weight distribution
- Additional 40+ DCF API methods already implemented in runtime

### 1.4 CBC Custom RPC Endpoints

Status: Could be implemented as convenience wrappers around existing Runtime APIs

Implemented Components:
- All underlying data accessible via DCF/PoS/PoI Runtime APIs
- Basic RPC structure with `RpcSecurityConfig` in `cbc-node/src/rpc.rs`
- `--enable-cbc-extensions` CLI flag in command.rs

Missing Components:
- Unified `cbc_*` namespace RPC handler
- Convenience methods that aggregate data from multiple pallets
- Help/discovery endpoint (`cbc_describe`)
- Health check endpoint

Required Endpoints:
- `cbc_getCurrentEpoch` - Wrapper for DCF's get_current_epoch
- `cbc_getExpectedAuthor` - Wrapper for DCF's get_expected_author
- `cbc_getValidatorProfile` - Wrapper for DCF's get_validator_profile
- `cbc_getScoreBreakdown` - Wrapper for DCF's get_validator_score_breakdown
- `cbc_getUptime` - Wrapper for DCF's get_validator_uptime
- `cbc_getTrustScore` - Aggregate PoS + PoI scores
- `cbc_getStatus` - System-wide status summary
- `cbc_getSlashingHistory` - Wrapper for DCF's get_slashing_history
- `cbc_describe` - List all available CBC RPC methods with descriptions
- `cbc_health` - Health check endpoint for monitoring

### 1.5 Additional RPC Infrastructure

RPC Infrastructure Requirements:
- RPC registration in `create_full()` function in `cbc-node/src/rpc.rs`
- Handler structs for custom RPCs (PosRpcApiImpl, PoiRpcApiImpl, DcfRpcApiImpl)
- CBC RPC namespace handler checks (validate `--enable-cbc-extensions` flag)
- Error handling and rate limiting for CBC RPCs

Specific Methods Required:
- `get_current_author()` - Available via DCF Runtime API, needs RPC wrapper
- `get_validator_score(account)` - Available via PoS Runtime API, needs RPC wrapper
- `get_epoch_participation(account)` - Available via DCF's get_validator_participation, needs RPC wrapper
- RPC method to fetch runtime version - Standard Substrate API, may need CBC-specific version info
- RPC for network discovery - Standard Substrate networking, may need CBC peer filtering

Implementation Pattern:
```rust
// In cbc-node/src/rpc.rs
#[rpc(server)]
pub trait PosRpcApi {
    #[method(name = "pos_getValidatorScore")]
    fn get_validator_score(&self, validator: AccountId) -> RpcResult<u32>;
}

pub struct PosRpcApiImpl<C> {
    client: Arc<C>,
}

impl<C> PosRpcApiServer for PosRpcApiImpl<C>
where
    C: ProvideRuntimeApi<Block> + 'static,
    C::Api: pallet_cbc_pos::PosApi<Block, AccountId, Balance>,
{
    fn get_validator_score(&self, validator: AccountId) -> RpcResult<u32> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;
        api.get_validator_score(at, validator)
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }
}
```

### 1.6 RPC Documentation

Status: Runtime APIs are implemented but not documented for external use

Existing Documentation:
- Runtime API implementations in `cbc-runtime/src/apis.rs`
- Some inline documentation in pallet files
- `docs/rpc-endpoints.md` exists in `rd` branch (needs to be copied)

Missing Documentation:
- Comprehensive RPC documentation for PoS/PoI/DCF endpoints
- Example curl/wscat commands for testing each endpoint
- Polkadot.js type definitions for custom RPCs
- RPC method signatures and return types documentation
- Error codes and handling documentation
- Rate limiting and security considerations
- Manual RPC test confirmation and results

Polkadot.js Integration Issues:
- Not accessible via Polkadot.js Apps (no RPC bridge implemented)
- Custom types need to be defined for CBC-specific structures
- RPC methods need to be registered in Polkadot.js metadata

Required Documentation:
- Complete RPC endpoint reference with examples
- Integration guide for Polkadot.js
- Testing procedures and expected responses
- Security and rate limiting configuration

---

## 2. Chain Specification & Configuration

### 2.1 Chain Properties (Missing)
- `tokenSymbol` not configured
- `tokenDecimals` not configured
- `protocolId` not set
- Logo URI not configured
- No `TOKEN_SYMBOL` constant
- No `TOKEN_DECIMALS` constant

### 2.2 Metadata Exposure (Missing)
- `cbcEpochLength` not exposed in metadata
- `cbcValidatorCount` not exposed in metadata

### 2.3 Chain Spec Generation
- Generate and test JSON chain specs
- No automated chain spec generation
- No Polkadot.js validation documented

### 2.4 Network Configuration (Missing)
- Bootnode configuration examples
- Telemetry endpoint configuration
- Token properties and telemetry endpoints not explicitly configured

---

## 3. Testing & Validation

### 3.1 Node Testing (Incomplete)
- Test node with `--dev` flag (FAILING, needs implementation)
- Test custom RPC endpoint
- Not tested on clean machine

### 3.2 Multi-Node Testing (Not Started)
- Multi-Node Test Setup (needs to start from scratch)
- No documented multi-node test setup
- No bootnode configuration for testing
- No 2+ node test with DCF consensus active
- No verification of block author rotation
- No epoch transition verification logs
- No PoI update verification

### 3.3 Runtime API Testing (Missing)
- No dedicated `cbc-runtime/tests/` directory
- No comprehensive API test suite covering all 50+ APIs
- Tests for `get_current_epoch()`
- Tests for `get_expected_block_author()`
- Tests for `get_inference_result()`

### 3.4 Deterministic Authoring Testing (Missing)
- Deterministic authoring smoke test
- CLI/RPC utility to query `expected_author` for upcoming N blocks
- Verification that produced authors match expectations

---

## 4. Explorer & UI Integration

### 4.1 BlockScout Integration (Not Done)
- BlockScout not tested
- BlockScout not integrated
- No BlockScout testing documented

### 4.2 Substrate Frontend Template (Not Done)
- No Substrate Frontend Template setup
- No Substrate Frontend Template testing

### 4.3 Explorer Integration Documentation (Missing)
- `docs/explorer-integration.md` (missing)
- No explorer connection automation
- No UI showing current block author
- No score history visualization
- No epoch transition display

### 4.4 Explorer Utilities (Missing)
- No `explorer-utils/` directory
- No `preview-validators.js` script
- No frontend integration
- No validator display tooling

### 4.5 Polkadot.js Integration
- Confirm Polkadot.js Apps can read CBC-specific pallets and metadata
- Document minimum working versions

---

## 5. Logging & Monitoring

### 5.1 Log Filtering & Formatting (Missing)
- No startup log filtering
- No `[CBC-LOG]` prefixes for CBC modules
- No `[CBC-VAL]` log tags for validator tracing
- No color-coding
- No `RUST_LOG` configuration in `main.rs`
- No consistent log tag format (e.g., `[CBC-VAL]`, `[CBC-EPOCH]`)
- No log level configuration per module
- CLI filter option for CBC logs
- `--cbc-log-only` flag to filter for `cbc-*` modules

### 5.2 Log File Management (Missing)
- Log file flag not actually used for redirection
- No audit of spam logs vs summaries

### 5.3 Startup Information (Missing)
- No runtime version printed at startup
- No peer ID display
- No latency information

### 5.4 Metrics & Monitoring (Missing)
- `author_mismatch_total` Prometheus metric not registered
- No Grafana dashboard JSON
- No performance profiling results documented
- No bottleneck analysis
- No optimization suggestions

---

## 6. CLI & Tooling

### 6.1 CLI Subcommands (Missing)
- No CLI subcommand for runtime version info
- No CLI subcommand for peer info
- `cbc-runtime-upgrade --wasm` CLI subcommand
- No CLI for one-line health summary

### 6.2 Output Modes (Missing)
- No JSON/plain-text output modes

### 6.3 Fork Detection Tool (Missing)
- Fork detection CLI/RPC tool
- Method to compare local best/finalized blocks vs peer's state
- Warn on divergence above threshold with peer ID and block number difference

### 6.4 Runtime Upgrade Tooling (Incomplete)
- Runtime Upgrade Tooling (complete task needs implementation)
- Subcommand to submit new runtime WASM
- Print current and target spec/impl versions before submission
- Poll until enactment is confirmed
- `sudoUncheckedWeight` upgrade logic
- Version comparison flag
- WASM upload test script

---

## 7. Scripts & Automation

### 7.1 Maintenance Scripts (Missing)
- No `scripts/` directory
- No `reset-node.sh` (Linux/Mac)
- No `reset-node.ps1` (Windows)
- No database reset scripts
- No maintenance documentation
- No maintenance scripts

### 7.2 Bootstrap & Setup Scripts (Missing)
- Developer Setup Package (no bootstrap automation implemented)
- No shell/Python setup script
- No one-command node startup
- No cross-platform support (Linux/Windows WSL)
- Bootstrap scripts for 3-node localnet
- Bootnode configuration scripts
- Node bootstrap scripts (Linux + PowerShell for Windows)
- One-command scripts to clone, build, start localnet, and query CBC RPC

---

## 8. Documentation

### 8.1 Missing Documentation Files
- `docs/node-architecture.md` - mentioned but missing
- `docs/explorer-integration.md` - **missing**
- `docs/runtime-apis.md` - doesn't exist

### 8.2 Developer Documentation (Incomplete)
- No developer onboarding checklist
- No common errors section
- No README developer checklist
- No dependency install guide
- No documented pinned versions
- No `cargo tree` snapshots

### 8.3 Testing Documentation (Missing)
- No documented test procedures for multi-node setup

---

## 9. Runtime & Pallets

### 9.1 Validator Status (Missing)
- Explicit Validator Status Enum
- `ValidatorUptimeUpdated` Event at Epoch Transitions

### 9.2 Block Events (Missing)
- Log filtering for block events

---

## 10. CI/CD & Build

### 10.1 Build Artifacts (Missing)
- No Windows builds
- No artifact uploads on tags

### 10.2 Release Packaging (Missing)
- Release bundles (Linux + Windows binaries)
- ChainSpec JSONs in release
- Sample configs in release
- Bootstrap scripts in release
- README "localnet in 60s"
- Checksums for releases
- Ensure `--version` shows correct CBC version

---

## 12. Files to Copy from `rd` Branch

The following files exist in the `rd` branch and need to be merged into the current branch to address some of the pending work items:

### Tools
- `tools/fork-checker.rs`

### Scripts
- `scripts/network-info.sh`
- `scripts/network-info.ps1`
- `scripts/reset-node.sh`
- `scripts/reset-node.ps1`

### Environment Setup
- `env-setup/README.md`
- `env-setup/flake.nix`
- `env-setup/flake.lock`
- `env-setup/rust-toolchain.toml`

### Documentation
- `docs/developer-checklist.md`
- `docs/dcf-node-integration.md`
- `docs/rpc-endpoints.md`
- `docs/runtime-apis.md`
- `docs/network-tools.md`
- `docs/node-architecture.md`

### Diagrams
- `docs/diagrams/dcf_sequence.png`

### Impact on Pending Work

Copying these files will address the following pending items. However, since the current branch is not building, compatibility with the stable version should be verified. Files should be copied and tested individually rather than merged in bulk, or reimplemented from scratch if necessary.

Items Addressed by rd Branch Files:
- Fork detection CLI tool (`tools/fork-checker.rs`)
- Network info scripts (`scripts/network-info.*`)
- Node reset scripts (`scripts/reset-node.*`)
- Developer checklist documentation
- Node architecture documentation
- RPC endpoints documentation
- Runtime APIs documentation
- Network tools documentation
- Environment setup automation (Nix flake)
- DCF sequence diagram
