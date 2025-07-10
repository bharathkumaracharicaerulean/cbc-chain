# CBC Chain Runtime APIs

This document describes the developer-facing runtime APIs exposed by the CBC Chain. These APIs are the interface between the on-chain runtime and off-chain clients (such as the node, CLI, or frontend applications). They allow querying runtime state, submitting transactions, and interacting with consensus and custom logic.

---

## Table of Contents
- [API Categories](#api-categories)
- [Core APIs](#core-apis)
- [Block Building & Transaction APIs](#block-building--transaction-apis)
- [Consensus APIs](#consensus-apis)
- [Account & Payment APIs](#account--payment-apis)
- [Development & Testing APIs](#development--testing-apis)
- [Custom CBC APIs](#custom-cbc-apis)
- [Extending Runtime APIs](#extending-runtime-apis)
- [Example Usage](#example-usage)
- [References](#references)

---

## API Categories

CBC Chain exposes the following categories of runtime APIs:

- **Core APIs**: Basic runtime operations and metadata
- **Block Building & Transaction APIs**: Block construction, extrinsic processing, transaction validation
- **Consensus APIs**: Consensus engine and session key management
- **Account & Payment APIs**: Account nonces, transaction fees
- **Development & Testing APIs**: Benchmarking, try-runtime, genesis builder
- **Custom CBC APIs**: Chain-specific APIs for validator and inference logic

All runtime APIs are defined and implemented in [`cbc-runtime/src/apis.rs`](../cbc-runtime/src/apis.rs).

---

## Core APIs

### Core API (`sp_api::Core`)
- **version() -> RuntimeVersion**: Returns the runtime version.
- **execute_block(block: Block)**: Executes a block by applying all extrinsics.
- **initialize_block(header: &Header) -> ExtrinsicInclusionMode**: Initializes a new block.

### Metadata API (`sp_api::Metadata`)
- **metadata() -> OpaqueMetadata**: Returns the runtime metadata.
- **metadata_at_version(version: u32) -> Option<OpaqueMetadata>**: Metadata for a specific version.
- **metadata_versions() -> Vec<u32>**: All available metadata versions.

### View Function API (`frame_support::view_functions::runtime_api::RuntimeViewFunction`)
- **execute_view_function(id: ViewFunctionId, input: Vec<u8>) -> Result<Vec<u8>, ViewFunctionDispatchError>**: Executes a read-only operation.

---

## Block Building & Transaction APIs

### BlockBuilder API (`sp_block_builder::BlockBuilder`)
- **apply_extrinsic(extrinsic: Extrinsic) -> ApplyExtrinsicResult**: Applies an extrinsic.
- **finalize_block() -> Header**: Finalizes the current block.
- **inherent_extrinsics(data: InherentData) -> Vec<Extrinsic>**: Creates inherent extrinsics.
- **check_inherents(block: Block, data: InherentData) -> CheckInherentsResult**: Checks inherents.

### TaggedTransactionQueue API (`sp_transaction_pool::runtime_api::TaggedTransactionQueue`)
- **validate_transaction(source: TransactionSource, tx: Extrinsic, block_hash: Hash) -> TransactionValidity**: Validates a transaction.

---

## Consensus APIs

### Aura API (`sp_consensus_aura::AuraApi`)
- **slot_duration() -> SlotDuration**: Returns the slot duration in ms.
- **authorities() -> Vec<AuraId>**: List of current authorities.

### Grandpa API (`sp_consensus_grandpa::GrandpaApi`)
- **grandpa_authorities() -> AuthorityList**: Current Grandpa authorities.
- **current_set_id() -> SetId**: Current set ID.
- **submit_report_equivocation_unsigned_extrinsic(...) -> Option<()>**: (Disabled)
- **generate_key_ownership_proof(...) -> Option<OpaqueKeyOwnershipProof>**: (Disabled)

### Session Keys API (`sp_session::SessionKeys`)
- **generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8>**: Generates session keys.
- **decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>>**: Decodes session keys.

---

## Account & Payment APIs

### AccountNonce API (`frame_system_rpc_runtime_api::AccountNonceApi`)
- **account_nonce(account: AccountId) -> Nonce**: Returns the current nonce for an account.

### TransactionPayment API (`pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi`)
- **query_info(uxt: Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>**: Dispatch info for a transaction.
- **query_fee_details(uxt: Extrinsic, len: u32) -> FeeDetails<Balance>**: Fee details for a transaction.
- **query_weight_to_fee(weight: Weight) -> Balance**: Converts weight to fee.
- **query_length_to_fee(length: u32) -> Balance**: Converts length to fee.

### TransactionPaymentCall API (`pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi`)
- **query_call_info(call: RuntimeCall, len: u32) -> RuntimeDispatchInfo<Balance>**: Dispatch info for a call.
- **query_call_fee_details(call: RuntimeCall, len: u32) -> FeeDetails<Balance>**: Fee details for a call.
- **query_weight_to_fee(weight: Weight) -> Balance**: Converts weight to fee.
- **query_length_to_fee(length: u32) -> Balance**: Converts length to fee.

---

## Development & Testing APIs

### Benchmark API (`frame_benchmarking::Benchmark`, feature-gated)
- **benchmark_metadata(extra: bool) -> (Vec<BenchmarkList>, Vec<StorageInfo>)**: Benchmark metadata.
- **dispatch_benchmark(config: BenchmarkConfig) -> Result<Vec<BenchmarkBatch>, String>**: Dispatches benchmark execution.

### TryRuntime API (`frame_try_runtime::TryRuntime`, feature-gated)
- **on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight)**: Performs runtime upgrade checks.
- **execute_block(block: Block, state_root_check: bool, signature_check: bool, select: TryStateSelect) -> Weight**: Executes a block for testing.

### GenesisBuilder API (`sp_genesis_builder::GenesisBuilder`)
- **build_state(config: Vec<u8>) -> Result**: Builds genesis state from config.
- **get_preset(id: &Option<PresetId>) -> Option<Vec<u8>>**: Retrieves a genesis preset.
- **preset_names() -> Vec<PresetId>**: All available preset names.

---

## Custom CBC APIs

### CbcCustomApi
Custom APIs for validator and inference logic. Defined in `cbc-runtime/src/apis.rs`:

#### get_validator_profile(account: AccountId) -> ValidatorProfile<AccountId>
Returns profile information for a validator, including:
- `account_id: AccountId`
- `score: Option<u32>`
- `uptime: Option<u32>`
- `display_name: Option<Vec<u8>>`
- `inference_count: Option<u32>`
- `status: ValidatorStatus` (enum: Active, Inactive, Slashed, Unknown)

#### get_inference_result(account: AccountId) -> Option<InferenceResult>
Returns inference result for a given account:
- `submitted: bool`
- `accuracy: Option<u32>`
- `status: InferenceStatus` (enum: Pending, Verified, Rejected, Unknown)
- `last_submission_block: Option<u32>`

#### get_current_epoch() -> u32
Returns the current epoch from the PoS pallet.

#### get_expected_block_author() -> Option<AccountId>
Returns the expected block author (currently not implemented).

**Type Definitions:**
See [`cbc-runtime/src/types.rs`](../cbc-runtime/src/types.rs) for full type details.

---

## Example Usage

### From the CLI (Polkadot.js or Substrate API CLI)

- **Query account nonce:**
  ```bash
  subxt rpc call system.accountNextIndex --account <ACCOUNT_ID>
  ```
- **Query validator profile (custom API):**
  ```bash
  # Using Polkadot.js Apps (Developer > RPC calls):
  # Method: cbc_getValidatorProfile, Params: ["<ACCOUNT_ID>"]
  ```
- **Query transaction fee info:**
  ```bash
  subxt rpc call payment.queryInfo --extrinsic <HEX_EXTRINSIC> --len <LENGTH>
  ```

### From the Frontend (Polkadot.js API)

```js
// Example: Query validator profile
const result = await api.rpc.cbc.getValidatorProfile(accountId);
console.log(result);

// Example: Query account nonce
const nonce = await api.rpc.system.accountNextIndex(accountId);
```

---

## Extending Runtime APIs

To add or extend runtime APIs:

1. **Declare the API trait** using `sp_api::decl_runtime_apis!` in `cbc-runtime/src/apis.rs`:
   ```rust
   sp_api::decl_runtime_apis! {
       pub trait MyCustomApi {
           fn my_custom_function(param: SomeType) -> SomeReturnType;
       }
   }
   ```
2. **Implement the trait** inside the `impl_runtime_apis!` macro:
   ```rust
   impl crate::apis::MyCustomApi<Block> for Runtime {
       fn my_custom_function(param: SomeType) -> SomeReturnType {
           // Implementation
       }
   }
   ```
3. **Register the API**: The implementation in `impl_runtime_apis!` automatically registers the API for the runtime.
4. **Add types**: Define any new input/output types in `cbc-runtime/src/types.rs` or an appropriate module.

**Best Practices:**
- Document all APIs and types with Rust doc comments.
- Use strong types for input/output.
- Handle errors and edge cases.
- Consider performance and storage access patterns.

---

## References
- [Substrate Runtime APIs Documentation](https://docs.substrate.io/reference/runtime-apis/)
- [Frame Support Documentation](https://docs.rs/frame-support/latest/frame_support/)
- [Substrate API Documentation](https://docs.rs/sp-api/latest/sp_api/)
- [Polkadot.js API Docs](https://polkadot.js.org/docs/) 