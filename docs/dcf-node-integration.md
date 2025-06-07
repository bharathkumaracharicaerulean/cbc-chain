# DCF-Node Architecture Documentation

## Overview
This document outlines the integration of the Dynamic Consensus Framework (DCF) into the node architecture, replacing traditional Aura/Grandpa consensus mechanisms with an AI-enhanced Proof-of-Stake (PoS) and Proof-of-Inference (PoI) system.

## DCF Replacement of Aura/Grandpa

### Traditional vs DCF Consensus
- **Traditional (Aura/Grandpa)**: Uses fixed validator sets with stake-based selection
- **DCF**: Implements dynamic validator selection with AI-informed scoring

### Key Differences
1. **Validator Selection**
   - Aura/Grandpa: Static stake-based selection
   - DCF: Dynamic selection based on composite scores (stake + AI metrics)

2. **Finality Mechanism**
   - Aura/Grandpa: Fixed finality rounds
   - DCF: Adaptive finality based on validator performance and inference accuracy

## CBC-PoS Integration in Node-Side Validator Logic

### Score Components
1. **Stake Weight**
   - Traditional PoS component
   - Minimum stake requirements maintained

2. **Behavior Metrics**
   - Uptime tracking
   - Finality participation
   - Block production efficiency

3. **Inference Accuracy**
   - PoI feedback integration
   - Model output validation scores
   - Challenge resolution history

### Score Calculation
```rust
CompositeScore = α * StakeWeight + β * BehaviorScore + γ * InferenceScore
```
Where α, β, and γ are configurable weights that can be adjusted based on network requirements.

## Runtime Block Author Validation

### Validation Process
1. **Pre-block Validation**
   - Check validator eligibility
   - Verify minimum score threshold
   - Validate stake requirements

2. **Block Production**
   - Author selection based on composite score
   - Dynamic weight adjustment
   - Epoch-based rotation

3. **Post-block Validation**
   - Score update triggers
   - Slashing condition checks
   - Reward distribution

## RPC Integration

### DCF-specific RPC Endpoints
1. **Validator Information**
   ```json
   {
     "method": "dcf_getValidatorInfo",
     "params": ["validator_address"],
     "returns": {
       "score": "number",
       "stake": "number",
       "inference_accuracy": "number",
       "status": "string"
     }
   }
   ```

2. **Epoch Information**
   ```json
   {
     "method": "dcf_getEpochInfo",
     "params": [],
     "returns": {
       "current_epoch": "number",
       "validators": "array",
       "scores": "object"
     }
   }
   ```

3. **Inference Results**
   ```json
   {
     "method": "dcf_getInferenceResults",
     "params": ["round_number"],
     "returns": {
       "results": "array",
       "challenges": "array",
       "consensus": "object"
     }
   }
   ```

## Adaptive Proof-of-Stake (PoS) Implementation

### Core Components
1. **Validator Set Management**
   - Dynamic set maintenance
   - Periodic rotation
   - Score-based inclusion/exclusion

2. **Epoch System**
   - Configurable epoch length
   - Score re-evaluation
   - Validator set updates

3. **Score Management**
   - On-chain score submission
   - Runtime event hooks
   - External oracle integration

### Slashing and Rewards
1. **Slashing Conditions**
   - Score threshold violations
   - Malicious behavior detection
   - Challenge validation failures

2. **Reward Distribution**
   - Non-linear reward scaling
   - AI metric integration
   - Performance-based multipliers

## Proof-of-Inference (PoI) Integration

### Inference Validation
1. **Submission Process**
   - Model output submission
   - Result verification
   - Challenge mechanism

2. **Verification System**
   - Off-chain verification
   - Oracle integration
   - Challenge resolution

### PoI to PoS Feedback
1. **Score Impact**
   - Accuracy metrics
   - Challenge history
   - Behavior patterns

2. **Inter-pallet Communication**
   - Score updates
   - Slashing triggers
   - Reward adjustments

## Events and Monitoring

### Key Events
1. **Validator Events**
   - `ValidatorRegistered`
   - `ScoreSubmitted`
   - `ValidatorSlashed`
   - `EpochEnded`
   - `ValidatorEjected`

2. **Inference Events**
   - `InferenceSubmitted`
   - `InferenceChallenged`
   - `ChallengeAccepted`
   - `ChallengeRejected`
   - `InferenceScoreUpdated`

## Future Considerations

### V2 Enhancements
1. **Oracle Integration**
   - Model execution verification
   - Tamper detection
   - Delay monitoring

2. **Advanced Scoring**
   - Machine learning-based score adjustment
   - Cross-validator correlation analysis
   - Historical performance weighting

### Security Considerations
1. **Attack Vectors**
   - Sybil resistance
   - Stake manipulation
   - Inference manipulation

2. **Mitigation Strategies**
   - Dynamic threshold adjustment
   - Multi-factor validation
   - Challenge escalation 