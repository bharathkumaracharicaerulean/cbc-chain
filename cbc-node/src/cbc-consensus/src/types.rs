//! Common types used throughout the consensus engine
//! 
//! This module defines the core types used in the consensus implementation.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Block as BlockTrait;
use sp_runtime::generic::{Block, Header, UncheckedExtrinsic};
use sp_runtime::traits::BlakeTwo256;
use sp_core::sr25519::Public;
use std::default::Default;

/// Block type alias for the consensus engine
pub type BlockT = Block<Header<u32, BlakeTwo256>, UncheckedExtrinsic<Public, (), (), ()>>;

/// Configuration for epoch management
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct EpochConfig {
    /// Number of slots per epoch
    pub epoch_length: u32,
    /// Minimum stake required to be a validator
    pub min_validators: u32,
    /// Maximum number of validators per epoch
    pub max_validators: u32,
    /// Minimum stake required
    pub min_stake: u128,
}

/// Validator information including stake and performance metrics
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct ValidatorInfo {
    /// Validator's public key
    pub account_id: Public,
    /// Current stake amount
    pub stake: u128,
    /// Proof of Importance score
    pub performance_score: u32,
    /// Number of blocks produced
    pub blocks_produced: u32,
    /// Number of blocks missed
    pub blocks_missed: u32,
}

/// Mode for selecting block authors
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum AuthorSelectionMode {
    /// Round-robin selection
    RoundRobin,
    /// Stake-weighted selection
    StakeWeighted,
    /// Performance-based selection
    PerformanceBased,
    /// Pure Proof of Stake
    PoS,
    /// Hybrid PoS + Proof of Importance
    Hybrid,
}

/// Consensus parameters
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct ConsensusParams {
    /// Current author selection mode
    pub author_selection_mode: AuthorSelectionMode,
    /// Number of blocks required for finality
    pub finality_threshold: u32,
    /// Minimum time between blocks
    pub block_time: u64,
    /// Maximum block size
    pub max_block_size: u32,
    /// Maximum transactions per block
    pub max_transactions_per_block: u32,
}

/// Block import result
#[derive(Debug)]
pub enum ImportResult<B: BlockTrait> {
    /// Block imported successfully
    Imported(B::Hash),
    /// Block already exists
    AlreadyInChain(B::Hash),
    /// Block rejected
    Rejected(B::Hash, String),
}

/// Validator performance metrics
#[derive(Debug, Clone, Encode, Decode, TypeInfo, Default)]
pub struct ValidatorMetrics {
    /// Number of blocks produced
    pub blocks_produced: u32,
    /// Total blocks validated in current epoch
    pub blocks_validated: u32,
    /// Uptime percentage
    pub uptime: u32,
    /// Total blocks in epoch
    pub total_blocks: u32,
    /// Total missed blocks
    pub total_missed: u32,
}

impl ValidatorMetrics {
    /// Update validator score with new metrics
    pub fn update_validator_score(&mut self, _author: Public, _stake_weight: u32, _inference_weight: u32, final_score: u32) {
        self.blocks_produced = self.blocks_produced.saturating_add(1);
        self.blocks_validated = self.blocks_validated.saturating_add(1);
        self.uptime = final_score;
    }
}