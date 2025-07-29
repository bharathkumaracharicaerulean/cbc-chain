# Implementation Plan

- [x] 1. Replace simulated block production with real Substrate block authoring
  - Replace the TODO/simulation code in `produce_block_with_validation` method in `dcf.rs`
  - Integrate with existing `ProposerFactory` in `proposer_factory.rs` to create real blocks
  - Use the existing `transaction_pool` from `service.rs` to include real transactions
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Enhance existing ProposerFactory for real block creation
  - Update `ProposerFactory::create_block` method to create full blocks with transactions
  - Add transaction pool integration to the existing proposer factory
  - Implement proper block body creation with extrinsics from transaction pool
  - _Requirements: 1.1, 1.2, 4.2_

- [x] 3. Implement real block signing and import in DCF consensus
  - Add block signing functionality to `produce_block_with_validation` method
  - Integrate with existing `DcfImportQueue` for block import
  - Use keystore from service configuration for block signing
  - _Requirements: 1.3, 1.4, 4.3_

- [x] 4. Update consensus state management after real block production
  - Fix the commented-out state updates in `produce_block_with_validation`
  - Update `last_block_time` and `current_slot` after successful block production
  - Remove the error return that indicates simulation mode
  - _Requirements: 6.1, 6.4_

- [ ] 5. Integrate real block production with existing finality system
  - Connect the enhanced block production with existing `DcfFinality` in `finality.rs`
  - Submit produced blocks to finality engine for consensus
  - Update finality processing to handle real blocks instead of proposals
  - _Requirements: 3.1, 3.2_

- [ ] 6. Enhance existing import queue for real block validation
  - Update `DcfImportQueue` in `import_queue.rs` to validate real block authors
  - Add proper block header author extraction (replace placeholder implementation)
  - Integrate PoS+PoI validation in the import queue
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 7. Update service configuration to use real block production
  - Modify the consensus monitoring in `service.rs` to trigger real block production
  - Remove simulation logging and add real block production coordination
  - Ensure proper integration between all existing services
  - _Requirements: 1.5, 4.4, 4.5_

- [ ] 8. Add proper error handling for real block production failures
  - Replace generic error messages with specific block production errors
  - Add fallback mechanisms when block production fails
  - Implement proper recovery when validators miss their slots
  - _Requirements: 1.5, 2.5, 5.4, 5.5_

- [ ] 9. Enhance existing metrics and monitoring
  - Update existing `ValidatorMetrics` to track real block production
  - Add block production success/failure rates to monitoring
  - Integrate with existing consensus health monitoring
  - _Requirements: 6.2, 6.3, 6.5_

- [ ] 10. Test real block production with existing test framework
  - Update existing tests in `dcf.rs` to test real block production
  - Add integration tests for the complete block production pipeline
  - Test interaction between all existing components
  - _Requirements: 1.1, 2.1, 3.1_