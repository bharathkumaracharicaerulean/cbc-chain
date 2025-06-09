#![allow(unused_imports)]
//! Core consensus types and structures
//!
//! This module defines the fundamental types used throughout the consensus system,
//! including validator information, epoch configuration, and author selection modes.

use sp_runtime::traits::Block as BlockT;
use sp_core::ed25519::Public;
use std::collections::HashMap;

/// Validator information
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Validator's public key
    pub public_key: Public,
    /// Validator's stake
    pub stake: u128,
    /// Validator's metrics
    pub metrics: ValidatorMetrics,
}

/// Validator metrics
#[derive(Debug, Clone, Default)]
pub struct ValidatorMetrics {
    /// Number of blocks produced
    pub blocks_produced: u32,
    /// Number of blocks missed
    pub blocks_missed: u32,
    /// Average block time
    pub avg_block_time: f64,
    /// Total transactions processed
    pub total_transactions: u64,
}

/// Block statistics
#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    /// Total number of blocks
    pub total_blocks: u64,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Average block time
    pub avg_block_time: f64,
    /// Average transactions per block
    pub avg_transactions_per_block: f64,
    /// Number of failed imports
    pub failed_imports: u32,
}

/// Epoch information
#[derive(Debug, Clone)]
pub struct EpochInfo {
    /// Epoch number
    pub number: u32,
    /// Validators in this epoch
    pub validators: HashMap<Public, ValidatorInfo>,
    /// Start block number
    pub start_block: u32,
    /// End block number
    pub end_block: u32,
}

/// Epoch configuration
#[derive(Clone, Debug)]
pub struct EpochConfig {
    /// Number of blocks per epoch
    pub blocks_per_epoch: u32,
    /// Minimum stake required for validators
    pub min_stake: u128,
    /// Maximum number of validators per epoch
    pub max_validators: u32,
}

impl Default for EpochConfig {
    fn default() -> Self {
        Self {
            blocks_per_epoch: 100,
            min_stake: 1000,
            max_validators: 100,
        }
    }
}

/// Author selection criteria
#[derive(Debug, Clone)]
pub enum AuthorSelectionCriteria {
    /// Proof of Stake
    ProofOfStake,
    /// Proof of Inference
    ProofOfInference,
    /// Hybrid (PoS + PoI)
    Hybrid,
}

/// Author selection configuration
#[derive(Debug, Clone)]
pub struct AuthorSelectionConfig {
    /// Selection criteria
    pub criteria: AuthorSelectionCriteria,
    /// Minimum stake required
    pub min_stake: u128,
    /// Cooldown period
    pub cooldown_period: u32,
}

/// Proposer configuration
#[derive(Debug, Clone)]
pub struct ProposerConfig {
    /// Maximum block size
    pub max_block_size: u32,
    /// Maximum block weight
    pub max_block_weight: u32,
    /// Maximum transactions per block
    pub max_transactions: u32,
    /// Block time in milliseconds
    pub block_time: u32,
}

impl Default for ProposerConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1024 * 1024, // 1MB
            max_block_weight: 1_000_000,
            max_transactions: 1000,
            block_time: 6000, // 6 seconds
        }
    }
}

/// Import configuration
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Maximum block size
    pub max_block_size: u32,
    /// Maximum block weight
    pub max_block_weight: u32,
    /// Maximum transactions per block
    pub max_transactions: u32,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1024 * 1024, // 1MB
            max_block_weight: 1_000_000,
            max_transactions: 1000,
        }
    }
}

/// Finality configuration
#[derive(Debug, Clone)]
pub struct FinalityConfig {
    /// Number of blocks required for finality
    pub finality_blocks: u32,
    /// Maximum time allowed for finality
    pub max_finality_time: u64,
}

impl Default for FinalityConfig {
    fn default() -> Self {
        Self {
            finality_blocks: 10,
            max_finality_time: 60000, // 1 minute
        }
    }
}

/// Validator set configuration
#[derive(Debug, Clone)]
pub struct ValidatorSetConfig {
    /// Maximum number of validators
    pub max_validators: u32,
    /// Minimum stake required
    pub min_stake: u64,
    /// Cooldown period in blocks
    pub cooldown_period: u32,
    /// Number of blocks per epoch
    pub blocks_per_epoch: u32,
}

impl Default for ValidatorSetConfig {
    fn default() -> Self {
        Self {
            max_validators: 100,
            min_stake: 1000,
            cooldown_period: 100,
            blocks_per_epoch: 1000,
        }
    }
}

/// Consensus metrics
#[derive(Clone, Debug, Default)]
pub struct ConsensusMetrics {
    /// Block production stats
    pub block_stats: BlockStats,
    /// Validator metrics
    pub validator_metrics: HashMap<sp_core::ed25519::Public, ValidatorMetrics>,
    /// Current epoch number
    pub current_epoch: u32,
    /// Total epochs completed
    pub total_epochs: u32,
}

/// Generic block type for consensus
pub type Block = sp_runtime::generic::Block<
    sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
    sp_runtime::generic::UncheckedExtrinsic<(), (), (), ()>,
>; 