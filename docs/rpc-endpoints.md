# CBC API Documentation

This document provides comprehensive documentation for all APIs exposed by the CBC (Consensus-Based Chain) blockchain, including RPC endpoints and Runtime APIs.

## Table of Contents

1. [Overview](#overview)
2. [RPC Endpoints](#rpc-endpoints)
3. [Runtime APIs](#runtime-apis)
4. [Security Configuration](#security-configuration)
5. [Usage Examples](#usage-examples)
6. [Type Definitions](#type-definitions)

## Overview

The CBC blockchain exposes two main types of APIs:

### RPC Endpoints
External HTTP/WebSocket endpoints for querying blockchain state and submitting transactions:
- **PoS RPC**: Validator stake and performance data (4 methods)
- **PoI RPC**: Inference validation and challenge data (4 methods)  
- **DCF RPC**: Block authoring and consensus data (4 methods)
- **CBC Unified RPC**: Aggregated data from multiple pallets (9 methods)
- **Fork Detection RPC**: Network fork detection and monitoring (2 methods)
- **Chain RPC**: Basic chain information (1 method)

### Runtime APIs
Direct runtime state queries for internal node operations:
- **DCF Runtime API**: 45+ methods for consensus, validator management, and governance
- **PoS Runtime API**: 4 methods for stake and validator score queries
- **PoI Runtime API**: 3 methods for inference result and challenge queries

All CBC-specific endpoints require the `--enable-cbc-extensions` flag to be enabled on the node.

## RPC Endpoints

## Security Configuration

### CLI Flags

```bash
# Enable CBC RPC extensions (required for all CBC endpoints)
cbc-node --enable-cbc-extensions

# Enable unsafe methods (optional, for development only)
cbc-node --enable-cbc-extensions --rpc-unsafe-methods
```

### Security Checks

All CBC namespace endpoints (`cbc_*`, `dcf_*`, `poi_*`, `pos_*`, `fork_*`) perform security validation:

1. **Extension Check**: Verifies `--enable-cbc-extensions` flag is set
2. **Rate Limiting**: Enforces per-IP request limits
3. **Parameter Validation**: Validates all input parameters

## Rate Limiting

### Default Configuration

- **Window**: 60 seconds
- **Max Requests**: 100 per IP per window
- **Scope**: Per IP address

### Rate Limit Headers

Responses include rate limiting information:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

### Rate Limit Exceeded Response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32005,
    "message": "Rate limit exceeded. Try again later."
  },
  "id": 1
}
```

## Error Handling

### Standard Error Codes

| Code | Description | Example |
|------|-------------|---------|
| -32000 | Runtime API call failed | Invalid validator account |
| -32001 | Unauthorized access | CBC extensions disabled |
| -32002 | Invalid method | Method not found |
| -32005 | Rate limit exceeded | Too many requests |
| -32602 | Invalid params | Malformed parameters |
| -32603 | Internal error | Server error |

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Runtime API call failed: validator not found",
    "data": {
      "details": "Additional error context"
    }
  },
  "id": 1
}
```

## PoS RPC Endpoints

### pos_getValidatorScore

Get validator PoS performance score.

**Method**: `pos_getValidatorScore`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `u32` - Performance score (0-100)

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "pos_getValidatorScore",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"pos_getValidatorScore","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": 85,
  "id": 1
}
```

### pos_getValidatorStake

Get validator staked token amount.

**Method**: `pos_getValidatorStake`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `Balance` - Staked amount in smallest token unit

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "pos_getValidatorStake",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"pos_getValidatorStake","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "1000000000000000000",
  "id": 1
}
```

### pos_getSlashingCount

Get number of times validator has been slashed.

**Method**: `pos_getSlashingCount`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `u32` - Number of slashing events

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "pos_getSlashingCount",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"pos_getSlashingCount","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": 0,
  "id": 1
}
```

### pos_getValidatorStatus

Get validator status (Active/Inactive/Slashed).

**Method**: `pos_getValidatorStatus`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `ValidatorStatus` - Enum: "Active", "Inactive", or "Slashed"

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "pos_getValidatorStatus",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"pos_getValidatorStatus","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "Active",
  "id": 1
}
```

## PoI RPC Endpoints

### poi_getInferenceResult

Get validator inference result and confidence.

**Method**: `poi_getInferenceResult`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `Option<InferenceResult>` - Inference result with confidence score, or null

**InferenceResult Structure**:
```typescript
{
  result: number,      // Inference result value
  confidence: number   // Confidence score (0-100)
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "poi_getInferenceResult",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"poi_getInferenceResult","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "result": 42,
    "confidence": 95
  },
  "id": 1
}
```

### poi_getInferenceConfidence

Get inference confidence score only.

**Method**: `poi_getInferenceConfidence`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `Option<u32>` - Confidence score (0-100), or null

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "poi_getInferenceConfidence",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"poi_getInferenceConfidence","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": 95,
  "id": 1
}
```

### poi_getChallengeWindow

Get current challenge window parameters.

**Method**: `poi_getChallengeWindow`

**Parameters**: None

**Returns**: `ChallengeWindow` - Challenge window block range

**ChallengeWindow Structure**:
```typescript
{
  startBlock: number,  // Challenge window start block
  endBlock: number     // Challenge window end block
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "poi_getChallengeWindow",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"poi_getChallengeWindow","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "startBlock": 1000,
    "endBlock": 1050
  },
  "id": 1
}
```

### poi_getInferenceStatus

Get inference submission status.

**Method**: `poi_getInferenceStatus`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `InferenceStatus` - Enum: "Pending", "Submitted", "Challenged", or "Verified"

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "poi_getInferenceStatus",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"poi_getInferenceStatus","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "Submitted",
  "id": 1
}
```

## DCF RPC Endpoints

### dcf_getCurrentAuthor

Get current block author.

**Method**: `dcf_getCurrentAuthor`

**Parameters**: None

**Returns**: `Option<AccountId>` - Current block author account, or null

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "dcf_getCurrentAuthor",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"dcf_getCurrentAuthor","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "id": 1
}
```

### dcf_getExpectedAuthor

Get expected author for a specific block.

**Method**: `dcf_getExpectedAuthor`

**Parameters**:
- `blockNumber` (u32): Block number

**Returns**: `Option<AccountId>` - Expected block author account, or null

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "dcf_getExpectedAuthor",
  "params": [1000],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"dcf_getExpectedAuthor","params":[1000],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "id": 1
}
```

### dcf_getValidatorScores

Get trust scores for all active validators.

**Method**: `dcf_getValidatorScores`

**Parameters**: None

**Returns**: `Vec<(AccountId, u64)>` - Array of validator account and trust score pairs

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "dcf_getValidatorScores",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"dcf_getValidatorScores","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 8500],
    ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 7200],
    ["5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 6800]
  ],
  "id": 1
}
```

### dcf_getConsensusWeights

Get current PoS and PoI weight distribution.

**Method**: `dcf_getConsensusWeights`

**Parameters**: None

**Returns**: `ConsensusWeights` - Current consensus weight configuration

**ConsensusWeights Structure**:
```typescript
{
  posWeight: number,   // PoS weight in consensus calculation
  poiWeight: number    // PoI weight in consensus calculation
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "dcf_getConsensusWeights",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"dcf_getConsensusWeights","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "posWeight": 70,
    "poiWeight": 30
  },
  "id": 1
}
```

## CBC Unified RPC Endpoints

### cbc_getCurrentEpoch

Get the current epoch number.

**Method**: `cbc_getCurrentEpoch`

**Parameters**: None

**Returns**: `u32` - Current epoch number

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getCurrentEpoch",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getCurrentEpoch","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": 42,
  "id": 1
}
```

### cbc_getValidatorProfile

Get comprehensive validator profile including stake, scores, and status.

**Method**: `cbc_getValidatorProfile`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `ValidatorProfile` - Complete validator information

**ValidatorProfile Structure**:
```typescript
{
  account: string,           // Validator account ID
  stake: string,            // Staked amount (as string for large numbers)
  posScore: number,         // PoS performance score
  poiScore: number,         // PoI inference score
  trustScore: number,       // Combined trust score
  status: string,           // "Active", "Inactive", or "Slashed"
  authoredBlocks: number,   // Number of blocks authored
  missedBlocks: number      // Number of blocks missed
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getValidatorProfile",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getValidatorProfile","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "account": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "stake": "1000000000000000000",
    "posScore": 85,
    "poiScore": 95,
    "trustScore": 8800,
    "status": "Active",
    "authoredBlocks": 150,
    "missedBlocks": 5
  },
  "id": 1
}
```

### cbc_getTrustScore

Get validator trust score with PoS and PoI components.

**Method**: `cbc_getTrustScore`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `TrustScore` - Trust score breakdown

**TrustScore Structure**:
```typescript
{
  total: number,          // Total weighted trust score
  posComponent: number,   // PoS component contribution
  poiComponent: number    // PoI component contribution
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getTrustScore",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getTrustScore","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "total": 8800,
    "posComponent": 5950,
    "poiComponent": 2850
  },
  "id": 1
}
```

### cbc_listValidators

Get list of all active validators.

**Method**: `cbc_listValidators`

**Parameters**: None

**Returns**: `Vec<AccountId>` - Array of active validator account IDs

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_listValidators",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_listValidators","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
  ],
  "id": 1
}
```

### cbc_getStatus

Get system-wide status including epoch, validators, and health.

**Method**: `cbc_getStatus`

**Parameters**: None

**Returns**: `SystemStatus` - Comprehensive system status

**SystemStatus Structure**:
```typescript
{
  currentEpoch: number,        // Current epoch number
  activeValidators: number,    // Number of active validators
  totalValidators: number,     // Total number of validators
  lastFinalizedBlock: number,  // Last finalized block number
  consensusHealth: string      // "Healthy", "Degraded", or "Critical"
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getStatus",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getStatus","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "currentEpoch": 42,
    "activeValidators": 10,
    "totalValidators": 10,
    "lastFinalizedBlock": 4200,
    "consensusHealth": "Healthy"
  },
  "id": 1
}
```

### cbc_describe

List all available CBC RPC methods.

**Method**: `cbc_describe`

**Parameters**: None

**Returns**: `Vec<RpcMethodDescription>` - Array of method descriptions

**RpcMethodDescription Structure**:
```typescript
{
  name: string,           // Method name
  description: string,    // Method description
  params: string[],       // Parameter types
  returns: string         // Return type
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_describe",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_describe","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "name": "cbc_getCurrentEpoch",
      "description": "Get the current epoch number",
      "params": [],
      "returns": "u32"
    },
    {
      "name": "cbc_getValidatorProfile",
      "description": "Get comprehensive validator profile including stake, scores, and status",
      "params": ["AccountId"],
      "returns": "ValidatorProfile"
    }
  ],
  "id": 1
}
```

### cbc_health

Get health check status for monitoring.

**Method**: `cbc_health`

**Parameters**: None

**Returns**: `HealthStatus` - System health information

**HealthStatus Structure**:
```typescript
{
  isHealthy: boolean,     // Overall health status
  issues: string[]        // Array of health issues (empty if healthy)
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_health",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_health","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "isHealthy": true,
    "issues": []
  },
  "id": 1
}
```

### cbc_getBlockAuthoringStats

Get block authoring statistics for a specific validator.

**Method**: `cbc_getBlockAuthoringStats`

**Parameters**:
- `validator` (AccountId): Validator account ID

**Returns**: `BlockAuthoringStats` - Detailed authoring statistics

**BlockAuthoringStats Structure**:
```typescript
{
  authoredBlocks: number,      // Number of blocks authored
  missedBlocks: number,        // Number of blocks missed
  expectedBlocks: number,      // Total expected blocks
  participationRate: number,   // Participation rate percentage
  consecutiveMisses: number,   // Current consecutive misses
  lastAuthoredBlock: number?   // Last authored block number (null if none)
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getBlockAuthoringStats",
  "params": ["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getBlockAuthoringStats","params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "authoredBlocks": 150,
    "missedBlocks": 5,
    "expectedBlocks": 155,
    "participationRate": 96.77,
    "consecutiveMisses": 0,
    "lastAuthoredBlock": 4195
  },
  "id": 1
}
```

### cbc_getAllBlockAuthoringStats

Get block authoring statistics for all active validators.

**Method**: `cbc_getAllBlockAuthoringStats`

**Parameters**: None

**Returns**: `Vec<(AccountId, BlockAuthoringStats)>` - Array of validator and stats pairs

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getAllBlockAuthoringStats",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"cbc_getAllBlockAuthoringStats","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    [
      "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      {
        "authoredBlocks": 150,
        "missedBlocks": 5,
        "expectedBlocks": 155,
        "participationRate": 96.77,
        "consecutiveMisses": 0,
        "lastAuthoredBlock": 4195
      }
    ]
  ],
  "id": 1
}
```

## Fork Detection RPC Endpoints

### fork_checkPeers

Check for forks by comparing local and peer states.

**Method**: `fork_checkPeers`

**Parameters**:
- `peerEndpoints` (Vec<String>): Array of peer RPC endpoints
- `threshold` (u32): Divergence threshold for fork detection

**Returns**: `Vec<ForkReport>` - Array of fork detection reports

**ForkReport Structure**:
```typescript
{
  peerId: string,        // Peer identifier
  localBest: number,     // Local best block number
  peerBest: number,      // Peer best block number
  divergence: number     // Block number divergence
}
```

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "fork_checkPeers",
  "params": [
    ["http://peer1:9944", "http://peer2:9944"],
    10
  ],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"fork_checkPeers","params":[["http://peer1:9944","http://peer2:9944"],10],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "peerId": "peer-0",
      "localBest": 4200,
      "peerBest": 4195,
      "divergence": 5
    },
    {
      "peerId": "peer-1",
      "localBest": 4200,
      "peerBest": 4180,
      "divergence": 20
    }
  ],
  "id": 1
}
```

### fork_getStatus

Get fork detection service status.

**Method**: `fork_getStatus`

**Parameters**: None

**Returns**: `HashMap<String, Value>` - Service status information

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "fork_getStatus",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"fork_getStatus","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "service": "fork-detection",
    "version": "0.1.0",
    "localBestBlock": 4200,
    "localFinalizedBlock": 4150,
    "localBestHash": "0x1234567890abcdef..."
  },
  "id": 1
}
```

## Chain RPC Endpoints

### chain_getChainName

Get the chain name.

**Method**: `chain_getChainName`

**Parameters**: None

**Returns**: `String` - Chain name

**Example curl**:
```bash
curl -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "chain_getChainName",
  "params": [],
  "id": 1
}' http://localhost:9944
```

**Example wscat**:
```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"chain_getChainName","params":[],"id":1}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": "CBC-Chain",
  "id": 1
}
```

## Example Usage

### Monitoring Script

```bash
#!/bin/bash

# Check system health
curl -s -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_health",
  "params": [],
  "id": 1
}' http://localhost:9944 | jq '.result.isHealthy'

# Get current epoch
curl -s -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_getCurrentEpoch",
  "params": [],
  "id": 1
}' http://localhost:9944 | jq '.result'

# List all validators
curl -s -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0",
  "method": "cbc_listValidators",
  "params": [],
  "id": 1
}' http://localhost:9944 | jq '.result[]'
```

### JavaScript Client

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:9944');

ws.on('open', function open() {
  // Get validator profile
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'cbc_getValidatorProfile',
    params: ['5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'],
    id: 1
  }));
});

ws.on('message', function message(data) {
  const response = JSON.parse(data);
  console.log('Validator Profile:', response.result);
});
```

### Python Client

```python
import requests
import json

def get_validator_profile(validator_id):
    payload = {
        "jsonrpc": "2.0",
        "method": "cbc_getValidatorProfile",
        "params": [validator_id],
        "id": 1
    }
    
    response = requests.post(
        "http://localhost:9944",
        headers={"Content-Type": "application/json"},
        data=json.dumps(payload)
    )
    
    return response.json()["result"]

# Usage
profile = get_validator_profile("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
print(f"Trust Score: {profile['trustScore']}")
print(f"Status: {profile['status']}")
```

## Integration with Substrate-based Apps

### Type Definitions

For Substrate-based applications and blockchain explorers, use the CBC type definitions in `docs/cbc-types.json`:

```javascript
import { ApiPromise, WsProvider } from '@polkadot/api';
import cbcTypes from './docs/cbc-types.json';

const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ 
  provider, 
  types: cbcTypes.types,
  rpc: cbcTypes.rpc
});

// Use RPC methods
const epoch = await api.rpc.cbc.getCurrentEpoch();
const validators = await api.rpc.cbc.listValidators();
```

### Custom RPC Methods

```javascript
// CBC type definitions are already included in cbc-types.json
import cbcTypes from './docs/cbc-types.json';

const api = await ApiPromise.create({ 
  provider, 
  types: cbcTypes.types,
  rpc: cbcTypes.rpc
});
```

## Security Considerations

### Production Deployment

1. **Enable Rate Limiting**: Configure appropriate rate limits for your use case
2. **Use HTTPS**: Always use HTTPS in production environments
3. **Firewall Configuration**: Restrict RPC access to authorized IPs only
4. **Monitor Usage**: Track RPC usage patterns for anomaly detection
5. **Regular Updates**: Keep the node software updated

### Access Control

```bash
# Recommended production configuration
cbc-node \
  --enable-cbc-extensions \
  --rpc-port 9944 \
  --rpc-bind-ip 127.0.0.1 \
  --rpc-rate-limit-window 60 \
  --rpc-rate-limit-requests 50
```

### Monitoring

Set up monitoring for:
- RPC request rates and patterns
- Error rates by endpoint
- Response times
- Authentication failures
- Rate limit violations

### Best Practices

1. **Validate Inputs**: Always validate AccountId format before making requests
2. **Handle Errors**: Implement proper error handling for all RPC calls
3. **Cache Results**: Cache frequently accessed data to reduce load
4. **Use Batch Requests**: Combine multiple requests when possible
5. **Implement Timeouts**: Set appropriate timeouts for RPC calls

---

For additional support or questions about CBC APIs, please refer to the [CBC Documentation](../README.md) or open an issue in the project repository.

## Runtime APIs

Runtime APIs provide direct access to blockchain state for internal node operations. These are used by the node itself and by tools that need low-level access to runtime state.

### DCF Runtime API

The DCF (Dynamic Consensus Framework) API provides comprehensive access to consensus state, validator information, and governance parameters with 45+ methods including:

**Key Methods:**
- `get_validator_scores() -> Vec<(AccountId, u64)>`: Returns trust scores for all validators
- `get_validator_profile(validator: AccountId) -> Option<ValidatorProfile>`: Complete validator information
- `get_current_epoch() -> u32`: Current consensus epoch number
- `get_expected_author(block_number: u32) -> Option<AccountId>`: Expected block author
- `get_consensus_weights() -> (u64, u64)`: Current PoS and PoI weights
- `get_active_validators() -> Vec<AccountId>`: List of active validators
- `get_validator_uptime(validator: AccountId) -> Option<UptimeStats>`: Validator uptime statistics
- `get_slashing_history(validator: AccountId) -> Vec<SlashingRecord>`: Complete slashing history
- `get_epoch_config() -> EpochConfig`: Current epoch configuration
- `get_system_metrics() -> Vec<u8>`: Comprehensive system metrics

### PoS Runtime API

The PoS (Proof of Stake) API provides access to validator stake information:

- `get_validator_stake(validator: AccountId) -> Balance`: Current stake amount
- `get_validator_score(validator: AccountId) -> u32`: PoS performance score
- `get_active_validators() -> Vec<AccountId>`: Active validators list
- `get_slashing_count(validator: AccountId) -> u32`: Number of slashing events

### PoI Runtime API

The PoI (Proof of Inference) API provides access to inference results and challenges:

- `get_inference_result(validator: AccountId) -> Option<(u32, u32)>`: Inference result and epoch
- `get_challenge(validator: AccountId) -> Option<(AccountId, u32, u32)>`: Challenge information
- `get_current_epoch() -> u32`: Current inference epoch number

### Standard Substrate APIs

The CBC runtime also implements standard Substrate Runtime APIs:
- `sp_api::Core`: Basic runtime functionality
- `sp_api::Metadata`: Runtime metadata access
- `sp_block_builder::BlockBuilder`: Block construction
- `sp_transaction_pool::runtime_api::TaggedTransactionQueue`: Transaction validation
- `sp_session::SessionKeys`: Session key management
- `frame_system_rpc_runtime_api::AccountNonceApi`: Account nonce queries
- `pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi`: Fee calculation