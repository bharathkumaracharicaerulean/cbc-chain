# CBC Runtime

This directory contains the runtime implementation for the CBC Chain, which is built on Substrate.

## Runtime APIs

Runtime APIs are the interface between the runtime (on-chain logic) and the client (off-chain logic). They allow the client to query runtime state and submit transactions.

### Location

All runtime APIs are defined and implemented in:
- **`src/apis.rs`** - Main API implementations and custom CBC APIs

### API Categories

#### Core APIs
- **Core API** (`sp_api::Core`) - Basic runtime operations like version, block execution, and initialization
- **Metadata API** (`sp_api::Metadata`) - Provides runtime metadata for client-side type information
- **View Function API** (`frame_support::view_functions::runtime_api::RuntimeViewFunction`) - Executes read-only operations

#### Block Building & Transaction APIs
- **BlockBuilder API** (`sp_block_builder::BlockBuilder`) - Handles extrinsic application, block finalization, and inherents
- **TaggedTransactionQueue API** (`sp_transaction_pool::runtime_api::TaggedTransactionQueue`) - Validates transactions before inclusion

#### Consensus APIs
- **Aura API** (`sp_consensus_aura::AuraApi`) - Authority Round consensus mechanism for block production
- **Grandpa API** (`sp_consensus_grandpa::GrandpaApi`) - Finality gadget for block finalization
- **Session Keys API** (`sp_session::SessionKeys`) - Manages session key generation and decoding

#### Account & Payment APIs
- **AccountNonce API** (`frame_system_rpc_runtime_api::AccountNonceApi`) - Retrieves account nonces for transaction ordering
- **TransactionPayment API** (`pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi`) - Calculates transaction fees
- **TransactionPaymentCall API** (`pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi`) - Fee calculation for specific calls

#### Development & Testing APIs
- **Benchmark API** (`frame_benchmarking::Benchmark`) - Runtime benchmarking for performance testing (feature-gated)
- **TryRuntime API** (`frame_try_runtime::TryRuntime`) - Testing and migration utilities (feature-gated)
- **GenesisBuilder API** (`sp_genesis_builder::GenesisBuilder`) - Genesis state construction and preset management

#### Custom CBC APIs
- **CbcCustomApi** - Custom APIs specific to CBC Chain functionality:
  - Validator profile information (scores, slashing history)
  - Proof of Inference results
  - Epoch management
  - Block author prediction

## Extending Runtime APIs

### Adding a New Standard API

To add a new standard Substrate API:

1. **Import the trait** at the top of `src/apis.rs`:
   ```rust
   use some_crate::SomeApi;
   ```

2. **Add the implementation** inside the `impl_runtime_apis!` macro:
   ```rust
   impl some_crate::SomeApi<Block> for Runtime {
       fn some_function() -> SomeReturnType {
           // Implementation
       }
   }
   ```

3. **Add necessary imports** for the types used in your implementation.

### Adding a Custom API

To add a custom CBC-specific API:

1. **Declare the API trait** using `sp_api::decl_runtime_apis!`:
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
           // Implementation using your pallets
           MyPallet::some_storage_value(param)
       }
   }
   ```

3. **Add any necessary data structures** outside the macro:
   ```rust
   #[derive(codec::Encode, codec::Decode, scale_info::TypeInfo, Clone, PartialEq, Eq, Debug)]
   pub struct MyReturnType {
       pub field1: u32,
       pub field2: Vec<u8>,
   }
   ```

### Best Practices

1. **Documentation**: Always add comprehensive documentation for your APIs:
   ```rust
   /// Brief description of what this API does
   /// 
   /// More detailed explanation if needed.
   fn my_function(param: ParamType) -> ReturnType {
       // Implementation
   }
   ```

2. **Error Handling**: Use appropriate error types and handle edge cases:
   ```rust
   fn my_function(param: ParamType) -> Result<ReturnType, ErrorType> {
       if param.is_invalid() {
           return Err(ErrorType::InvalidParameter);
       }
       Ok(ReturnType::new(param))
   }
   ```

3. **Type Safety**: Use strong types and avoid raw bytes when possible:
   ```rust
   // Good
   fn get_account_info(account: AccountId) -> AccountInfo;
   
   // Avoid
   fn get_account_info(account: Vec<u8>) -> Vec<u8>;
   ```

4. **Performance**: Consider the performance impact of your API calls:
   - Avoid expensive computations in frequently called APIs
   - Use appropriate storage access patterns
   - Consider caching for expensive operations

### Testing APIs

To test your runtime APIs:

1. **Unit Tests**: Test individual API functions:
   ```rust
   #[test]
   fn test_my_custom_api() {
       // Setup test environment
       let mut t = frame_system::GenesisConfig::default()
           .build_storage::<Runtime>()
           .unwrap();
       
       // Test your API
       let result = MyCustomApi::my_custom_function(&mut t, param);
       assert_eq!(result, expected_value);
   }
   ```

2. **Integration Tests**: Test APIs in the context of the full runtime:
   ```rust
   #[test]
   fn test_api_integration() {
       new_test_ext().execute_with(|| {
           // Test your API with the full runtime
           let result = Runtime::my_custom_function(param);
           assert_eq!(result, expected_value);
       });
   }
   ```

### Versioning

When modifying existing APIs:

1. **Backward Compatibility**: Maintain backward compatibility when possible
2. **Version Bumps**: Increment the runtime version when breaking changes are made
3. **Migration**: Provide migration logic for breaking changes

## Related Files

- `src/lib.rs` - Main runtime configuration and pallet setup
- `src/apis.rs` - Runtime API implementations
- `src/configs.rs` - Runtime configuration constants
- `src/genesis_config_presets.rs` - Genesis configuration presets

## Resources

- [Substrate Runtime APIs Documentation](https://docs.substrate.io/reference/runtime-apis/)
- [Frame Support Documentation](https://docs.rs/frame-support/latest/frame_support/)
- [Substrate API Documentation](https://docs.rs/sp-api/latest/sp_api/) 