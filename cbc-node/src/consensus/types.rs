use sp_runtime::traits::Block as BlockT;
use std::collections::HashMap;

/// Configuration for epoch management
#[derive(Debug, Clone)]
pub struct EpochConfig {
    /// Number of slots per epoch
    pub slots_per_epoch: u32,
    /// Minimum stake required to be a validator
    pub min_stake: u128,
    /// Maximum number of validators per epoch
    pub max_validators: u32,
}

/// Validator information including stake and performance metrics
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Validator's public key
    pub public_key: Vec<u8>,
    /// Current stake amount
    pub stake: u128,
    /// Proof of Importance score
    pub poi_score: u32,
    /// Number of blocks produced
    pub blocks_produced: u32,
    /// Number of blocks missed
    pub blocks_missed: u32,
}

/// Mode for selecting block authors
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorSelectionMode {
    /// Pure Proof of Stake
    PoS,
    /// Hybrid PoS + Proof of Importance
    Hybrid,
    /// Round-robin selection
    RoundRobin,
}

/// Consensus parameters
#[derive(Debug, Clone)]
pub struct ConsensusParams {
    /// Current author selection mode
    pub author_selection_mode: AuthorSelectionMode,
    /// Number of blocks required for finality
    pub finality_threshold: u32,
    /// Minimum time between blocks
    pub min_block_time: u64,
}

/// Block import result
#[derive(Debug)]
pub enum ImportResult<B: BlockT> {
    /// Block imported successfully
    Imported(B::Hash),
    /// Block already exists
    AlreadyInChain(B::Hash),
    /// Block rejected
    Rejected(B::Hash, String),
}

/// Validator performance metrics
#[derive(Debug, Default)]
pub struct ValidatorMetrics {
    /// Map of validator public key to their performance stats
    pub validators: HashMap<Vec<u8>, ValidatorInfo>,
    /// Total blocks produced in current epoch
    pub total_blocks: u32,
    /// Total slots missed in current epoch
    pub total_missed: u32,
}
