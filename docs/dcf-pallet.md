# DCF Pallet Documentation

The DCF (Dynamic Consensus Framework) pallet implements an advanced consensus mechanism that combines Proof of Stake (PoS) and Proof of Inference (PoI) scores with configurable weights and robust governance mechanisms.

---

## 1. Purpose

The DCF pallet is responsible for:
- Managing validator set with dynamic scoring based on PoS and PoI
- Handling epoch transitions and validator activity tracking
- Providing governance mechanisms for validator management
- Tracking block authorship and participation
- Integrating with PoS and PoI systems
- Maintaining validator state and history

---

## 2. Key Features

### **Validator Management**
- Dynamic validator set management
- Validator state tracking with history
- Epoch-based activity tracking
- Uptime and participation rate monitoring

### **Scoring System**
- Combined PoS and PoI scoring
- Configurable consensus weights
- Score decay mechanism
- Score boosting for good behavior
- Penalty for missed blocks

### **Governance**
- Proposal system for validator management
- Voting mechanism for proposals
- Sudo controls for emergency actions
- Governance mode toggle

### **Epoch Management**
- Automatic epoch transitions
- Score updates per epoch
- Validator set updates
- History tracking

### **Block Authorship**
- Expected author calculation
- Block authorship tracking
- Participation rate calculation
- Score adjustments for block production

---

## 3. Key Storage Items

### **ValidatorStates**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorState>`
- **Description**: Stores validator state including scores, history, and participation metrics

### **EpochHistory**
- **Type**: `StorageMap<_, Blake2_128Concat, u32, EpochHistory<T>>`
- **Description**: Tracks epoch statistics including validator scores and participation

### **GovernanceProposals**
- **Type**: `StorageMap<_, Blake2_128Concat, u32, GovernanceProposal<T>>`
- **Description**: Stores pending governance proposals and votes

### **PendingValidatorActions**
- **Type**: `StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorAction>`
- **Description**: Tracks pending validator join/leave requests

---

## 4. Key Events

### **ValidatorScoreUpdated**
- **Fields**: `{ validator: T::AccountId, score: u64 }`
- **Description**: Emitted when a validator's score is updated

### **EpochTransitioned**
- **Fields**: `{ epoch: u32 }`
- **Description**: Emitted when a new epoch begins

### **ProposalSubmitted**
- **Fields**: `{ proposal_id: u32, proposer: T::AccountId, action: ProposalAction<T> }`
- **Description**: Emitted when a governance proposal is submitted

### **ProposalVoted**
- **Fields**: `{ proposal_id: u32, voter: T::AccountId, approve: bool }`
- **Description**: Emitted when a proposal is voted on

### **ProposalExecuted**
- **Fields**: `{ proposal_id: u32, result: bool }`
- **Description**: Emitted when a proposal is executed

### **ValidatorJoined**
- **Fields**: `{ validator: T::AccountId, epoch: u32 }`
- **Description**: Emitted when a validator joins the set

### **ValidatorLeft**
- **Fields**: `{ validator: T::AccountId, epoch: u32 }`
- **Description**: Emitted when a validator leaves the set

---

## 5. Key Errors

### **ValidatorNotFound**
- **Description**: Validator not found in the system

### **InvalidWeight**
- **Description**: Invalid consensus weight configuration

### **InvalidEpochConfig**
- **Description**: Invalid epoch configuration

### **NotEnoughValidators**
- **Description**: Not enough active validators

### **ProposalNotFound**
- **Description**: Governance proposal not found

### **AlreadyVoted**
- **Description**: Account has already voted on the proposal

### **InvalidProposalAction**
- **Description**: Invalid proposal action

---

## 6. Runtime API

The pallet exposes the following runtime API:

```rust
trait DcfApi<AccountId> {
    fn get_validator_scores() -> Vec<(AccountId, u64)>;
    fn get_current_epoch() -> u32;
    fn get_validator_stake_score(validator: AccountId) -> u64;
    fn get_validator_inference_score(validator: AccountId) -> u64;
    fn get_consensus_weights() -> (u64, u64);
    fn is_validator_active(validator: AccountId) -> bool;
    fn get_expected_author(block_number: u32) -> Option<AccountId>;
    fn get_validator_score_history(validator: AccountId) -> Vec<u64>;
    fn get_validator_participation(validator: AccountId) -> (u32, u32);
    fn get_active_validators() -> Vec<AccountId>;
    fn get_validator_last_active(validator: AccountId) -> u32;
    fn validate_block_author(block_number: u32, author: AccountId);
    fn get_validator_profile(account_id: AccountId) -> Option<(u64, u32, u32, u32, u32)>;
    fn get_inference_result(account_id: AccountId) -> Option<u64>;
    fn get_epoch_history(epoch_number: u32) -> Option<RuntimeEpochHistory<AccountId>>;
    fn get_recent_epochs(n: u32) -> Vec<RuntimeEpochHistory<AccountId>>;
}
```

---

## 7. Configuration

The pallet requires the following configuration parameters:

### **Runtime Configuration**
- `MaxValidators`: Maximum number of validators
- `MinStake`: Minimum stake requirement
- `MaxEpochHistory`: Maximum number of epochs to track
- `MaxValidatorHistory`: Maximum validator history entries
- `MinActiveValidators`: Minimum active validators
- `MinValidatorScore`: Minimum score requirement
- `BlocksPerEpoch`: Number of blocks per epoch
- `PosWeight`: PoS score weight (0-100)
- `PoiWeight`: PoI score weight (0-100)

### **Genesis Configuration**
- Initial validator set
- Initial scores
- Initial epoch
- Initial epoch configuration
- Initial governance state

---

## 8. Key Functions

### **Validator Management**
- `join_validator_set`: Request to join validator set
- `leave_validator_set`: Request to leave validator set
- `eject_validator`: Eject a validator for misbehavior

### **Score Management**
- `update_validator_stake_score`: Update PoS score
- `update_validator_inference_score`: Update PoI score
- `update_consensus_weights`: Update PoS/PoI weights
- `boost_score`: Boost validator score
- `apply_score_decay`: Apply score decay

### **Governance**
- `submit_proposal`: Submit governance proposal
- `vote_proposal`: Vote on proposal
- `execute_proposal`: Execute approved proposal
- `set_governance_mode`: Toggle governance mode

### **Epoch Management**
- `handle_epoch_transition`: Handle epoch transitions
- `should_transition_epoch`: Check if epoch should transition
- `apply_pending_validator_actions`: Apply validator actions

### **Block Authorship**
- `record_block_authorship`: Track block authorship
- `record_missed_block`: Track missed blocks
- `get_expected_author`: Get expected block author

---

## 9. Integration Points

The DCF pallet integrates with:
- PoS system for stake management
- PoI system for inference scoring
- Runtime hooks for block and epoch events
- Governance system for validator management
- Runtime APIs for external access

---

## 10. Best Practices

1. **Score Management**
   - Regularly update PoS and PoI scores
   - Monitor score decay and boost mechanisms
   - Maintain optimal consensus weights

2. **Governance**
   - Use governance mode for sensitive operations
   - Monitor proposal voting and execution
   - Maintain quorum for proposals

3. **Epoch Management**
   - Configure appropriate epoch length
   - Monitor validator activity
   - Track score history

4. **Validator Management**
   - Maintain minimum validator count
   - Monitor validator participation
   - Handle inactive validators promptly

---

## 11. Security Considerations

1. **Score Manipulation**
   - Implement proper score validation
   - Prevent score gaming
   - Monitor score changes

2. **Governance Security**
   - Secure proposal submission
   - Prevent double voting
   - Validate proposal actions

3. **Validator Security**
   - Monitor validator activity
   - Handle inactive validators
   - Maintain minimum stake requirements

4. **Epoch Security**
   - Prevent epoch manipulation
   - Validate epoch transitions
   - Track validator history

---

## 12. Future Enhancements

1. **Advanced Scoring**
   - More granular score components
   - Dynamic weight adjustments
   - Advanced score decay mechanisms

2. **Enhanced Governance**
   - More proposal types
   - Advanced voting mechanisms
   - Proposal categorization

3. **Improved Validator Management**
   - Better validator selection
   - Advanced participation tracking
   - Enhanced ejection criteria

4. **Better Integration**
   - More runtime APIs
   - Better PoS/PoI integration
   - Enhanced telemetry

---

## 13. Conclusion

The DCF pallet provides a robust and flexible consensus framework that combines PoS and PoI scoring with advanced governance mechanisms. It offers comprehensive validator management, epoch handling, and score tracking capabilities while maintaining security and efficiency.