//! # pallet-cbc-dcf
//!
//! This pallet implements the Dynamic Consensus Framework (DCF) for the CBC-Chain, providing advanced validator management, scoring, and on-chain governance. It supports dynamic validator sets, configurable consensus weights, and robust governance mechanisms for slashing, rewards, and ejection.
//!
//! ## Main Features
//! - **Validator Scoring:** Combines Proof-of-Stake (PoS) and Proof-of-Inference (PoI) scores with configurable weights, supporting dynamic adjustment and decay.
//! - **Epoch Management:** Handles epoch transitions, validator activity tracking, score decay, and validator set updates.
//! - **Governance:** Enables on-chain proposals for slashing, rewarding, and ejecting validators, with voting and execution logic.
//! - **Block Authorship Tracking:** Monitors block authorship and missed blocks, applying score boosts or penalties accordingly.
//! - **Runtime Hooks:** Integrates with runtime hooks for per-block and per-epoch logic, including automatic epoch transitions and score updates.
//! - **Genesis Configuration:** Allows initialization of validator set, scores, and consensus parameters at genesis.
//! - **APIs:** Exposes runtime APIs for querying validator scores, participation, epoch state, and expected block authors.
//! - **Sudo Controls:** Supports governance mode toggling and sudo-only operations for manual intervention and testing.
//!
//! ## Dispatchable Calls (pallet index)
//! - **0. `update_validator_stake_score`**: Update a validator's PoS stake score.  
//!   - Root required if governance mode is enabled, otherwise signed.
//! - **1. `update_validator_inference_score`**: Update a validator's PoI inference score.  
//!   - Root required if governance mode is enabled, otherwise signed.
//! - **2. `update_consensus_weights`**: Update PoS and PoI weights (must sum to 100).  
//!   - Root required.
//! - **3. `set_governance_mode`**: Toggle governance mode (sudo-like).  
//!   - Root required.
//! - **4. `sudo_advance_epoch`**: Manually advance epoch (governance mode only).  
//!   - Root required.
//! - **5. `submit_proposal`**: Submit a governance proposal (slash, reward, eject).  
//!   - Signed.
//! - **6. `vote_proposal`**: Vote on a governance proposal.  
//!   - Signed.
//! - **7. `execute_proposal`**: Execute an approved governance proposal.  
//!   - Root required.
//! - **8. `propose_slash_validator`**: Sudo propose to slash a validator.  
//!   - Root required.
//! - **9. `propose_reward_validator`**: Sudo propose to reward a validator.  
//!   - Root required.
//! - **10. `propose_eject_validator`**: Sudo propose to eject a validator.  
//!   - Root required.
//! - **18. `join_validators`**: Join the validator set by meeting minimum stake requirements.  
//!   - Signed required.
//! - **19. `leave_validators`**: Request to leave validator set with cooldown period.  
//!   - Signed required. Funds remain reserved until cooldown expires.
//! - **25. `cancel_leave_request`**: Cancel a pending leave request before cooldown expires.  
//!   - Signed required.
//! - **26. `propose_default_reward_validator`**: Propose to reward a validator with default amount.  
//!   - Root required.
//! - **27. `propose_reward_multiple_validators`**: Propose to reward multiple validators.  
//!   - Root required.
//! - **28. `propose_default_reward_multiple_validators`**: Propose to reward multiple validators with default amount.  
//!   - Root required.
//! - **29. `propose_reward_all_active_validators`**: Propose to reward all active validators.  
//!   - Root required.
//! - **30. `slash_validator`**: Directly slash a validator's reserved stake.  
//!   - Root required. Slashes from reserved balance, burns funds, updates scores.
//! - **31. `slash_validator_percentage`**: Slash a percentage of validator's reserved stake.  
//!   - Root required. Slashes percentage (0-100%) of reserved balance.
//! - **32. `slash_multiple_validators`**: Slash multiple validators with same amount.  
//!   - Root required. Batch slashing operation.
//! - **21. `report_validator_misbehavior`**: Report validator misbehavior with evidence.  
//!   - Signed required, must be in ValidatorSet, automatic slashing at threshold.
//! - **22. `simulate_inference`**: Simulate an inference event for development.  
//!   - Signed required, increments inference_count for validator.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#[warn(unused_comparisons)]

// Benchmarking module
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

// --- Imports --- //
use frame_support::{
    pallet_prelude::*,
    traits::{Get, Currency, ReservableCurrency},
    BoundedVec,
    Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{SaturatedConversion, AtLeast32BitUnsigned, Saturating},
    DigestItem,
    codec, 
    offchain::{
        storage::StorageValueRef,
        storage_lock::{StorageLock, BlockAndTime},
        Duration,
    },

};
use sp_io;
use sp_std::prelude::*;
use sp_std::fmt; 
use pallet_cbc_pos as pos;
use pallet_cbc_poi as poi;
use serde::{Serialize, Deserialize};

use scale_info::prelude::format;
// --- Runtime API Declarations --- //
// These APIs are exposed to the runtime for querying validator and consensus state.
sp_api::decl_runtime_apis! {
    pub trait DcfApi<AccountId>
    where
        AccountId: codec::Codec + Clone + Eq + sp_std::fmt::Debug,
    {
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
        fn get_validator_profile(account_id: AccountId) -> Option<(u64, u64, u64, u32, u32, u32, u32)>;
        fn get_inference_result(account_id: AccountId) -> Option<u64>;
        fn get_epoch_history(epoch_number: u32) -> Option<RuntimeEpochHistory<AccountId>>;
        fn get_recent_epochs(n: u32) -> Vec<RuntimeEpochHistory<AccountId>>;
        fn get_governance_mode() -> bool;
        fn get_validator_consensus_contribution(validator: AccountId) -> Option<(u64, u64, u64)>;
        fn get_epoch_config() -> EpochConfig;
        fn get_validator_epoch_stats(validator: AccountId, epoch: u32) -> Option<EpochStats>;
        fn get_total_validators_count() -> u32;
        fn get_validator_set_info() -> (u32, u32, u32);
        fn get_validators_by_score() -> Vec<(AccountId, u64)>;
        fn get_last_finalized_block() -> u32;
        fn is_block_finalized(block_number: u32) -> bool;
        fn get_misbehavior_report_count(validator: AccountId) -> u32;
        fn get_misbehavior_reporters(validator: AccountId) -> Vec<AccountId>;
        fn get_misbehavior_evidence(validator: AccountId, reporter: AccountId) -> Option<Vec<u8>>;
        fn is_validator_at_risk(validator: AccountId) -> bool;
        fn get_finality_info() -> (u32, u32);
        fn blocks_since_finalization(current_block: u32) -> u32;
        fn get_validator_leave_request(validator: AccountId) -> Option<u32>;
        fn validate_expected_author(block_number: u32, actual_author: AccountId) -> bool;
        fn get_validator_stake(validator: AccountId) -> u128;
        fn get_leave_request_status(validator: AccountId) -> Option<(u32, u32, bool)>; // (request_block, expires_at, can_execute)
        fn get_epoch_manager_config() -> (u64, u64, u32, u32, u32, u32, u64, u32, u32, u32, u32); // (min_perf_score, high_perf_score, min_participation, high_participation, max_missed_blocks, max_missed_blocks_high, healthy_validator_score, healthy_participation_rate, healthy_missed_blocks_max, leave_cooldown, top_validators_display_count)
        fn get_epoch_length() -> u32; // Get configurable epoch length from T::EpochLength
        fn validate_block_author_strict(block_number: u32, actual_author: AccountId) -> Result<(), u8>;
    }
}

// --- Weights Module --- //
pub mod weights;
pub use weights::*;

// --- Pallet Declaration --- //
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    // --- Data Structures --- //

    /// Per-epoch statistics for a validator.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct EpochStats {
        pub epoch: u32,
        pub stake_score: u64,
        pub inference_score: u64,
        pub final_score: u64,
        pub authored_blocks: u32,
        pub missed_blocks: u32,
    }

    /// State for a validator, including current stats and history.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorState<T: Config> {
        pub last_active_epoch: u32,
        pub current: EpochStats,
        pub history: BoundedVec<EpochStats, T::MaxValidatorHistorySize>, // Use Config constant
        pub uptime: u32, // Number of epochs active
        pub inference_success_count: u32,
        pub participation_rate: u32, // Percentage (0 to T::PercentagePrecision)
        pub inference_count: u64, // Total number of inferences performed
        pub last_active_block: u32, // Last block number where validator was active
        pub name: Option<BoundedVec<u8, T::MaxValidatorNameSize>>, // Use Config constant
        pub trust_score: u64, // Computed trust score based on uptime, inference success, and slashing history
    }

    /// Configuration for epochs (block count, min stake, max validators).
    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EpochConfig {
        pub blocks_per_epoch: u32,
        pub min_stake: u128,
        pub max_validators: u32,
    }

    /// Actions that can be proposed via governance.
    #[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
    pub enum ProposalAction<T: Config + TypeInfo + fmt::Debug> { // <-- Change here
        Slash { validator: T::AccountId, amount: <T as pallet::Config>::Balance },
        Reward { validator: T::AccountId, amount: <T as pallet::Config>::Balance },
        Eject { validator: T::AccountId, reason: EjectionReason },
        AddValidator { validator: T::AccountId },
        RemoveValidator { validator: T::AccountId },
        RewardMultiple { validators: BoundedVec<T::AccountId, T::MaxProposalActionBoundedVecSize>, amount: <T as pallet::Config>::Balance },
    }

    impl<T: Config + TypeInfo + fmt::Debug> Clone for ProposalAction<T> {
        fn clone(&self) -> Self {
            match self {
                Self::Slash { validator, amount } => Self::Slash { validator: validator.clone(), amount: *amount },
                Self::Reward { validator, amount } => Self::Reward { validator: validator.clone(), amount: *amount },
                Self::Eject { validator, reason } => Self::Eject { validator: validator.clone(), reason: reason.clone() },
                Self::AddValidator { validator } => Self::AddValidator { validator: validator.clone() },
                Self::RemoveValidator { validator } => Self::RemoveValidator { validator: validator.clone() },
                Self::RewardMultiple { validators, amount } => Self::RewardMultiple { validators: validators.clone(), amount: *amount },
            }
        }
    }

    impl<T: Config + TypeInfo + fmt::Debug> PartialEq for ProposalAction<T> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Slash { validator: v1, amount: a1 }, Self::Slash { validator: v2, amount: a2 }) => v1 == v2 && a1 == a2,
                (Self::Reward { validator: v1, amount: a1 }, Self::Reward { validator: v2, amount: a2 }) => v1 == v2 && a1 == a2,
                (Self::Eject { validator: v1, reason: r1 }, Self::Eject { validator: v2, reason: r2 }) => v1 == v2 && r1 == r2,
                (Self::AddValidator { validator: v1 }, Self::AddValidator { validator: v2 }) => v1 == v2,
                (Self::RemoveValidator { validator: v1 }, Self::RemoveValidator { validator: v2 }) => v1 == v2,
                (Self::RewardMultiple { validators: v1, amount: a1 }, Self::RewardMultiple { validators: v2, amount: a2 }) => v1 == v2 && a1 == a2,
                _ => false,
            }
        }
    }

    impl<T: Config + TypeInfo + fmt::Debug> Eq for ProposalAction<T> {}

    /// Governance proposal structure.
    #[derive(Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct GovernanceProposal<T: Config + TypeInfo + fmt::Debug> { // <-- Change here
        pub proposer: T::AccountId,
        pub action: ProposalAction<T>,
        pub status: ProposalStatus,
        pub votes_for: u32,
        pub votes_against: u32,
    }

    impl<T: Config + TypeInfo + fmt::Debug> Clone for GovernanceProposal<T> {
        fn clone(&self) -> Self {
            Self {
                proposer: self.proposer.clone(),
                action: self.action.clone(),
                status: self.status.clone(),
                votes_for: self.votes_for,
                votes_against: self.votes_against,
            }
        }
    }

    impl<T: Config + TypeInfo + fmt::Debug> PartialEq for GovernanceProposal<T> {
        fn eq(&self, other: &Self) -> bool {
            self.proposer == other.proposer &&
            self.action == other.action &&
            self.status == other.status &&
            self.votes_for == other.votes_for &&
            self.votes_against == other.votes_against
        }
    }

    impl<T: Config + TypeInfo + fmt::Debug> Eq for GovernanceProposal<T> {}

    /// Status of a governance proposal.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
    pub enum ProposalStatus {
        Pending,
        Approved,
        Rejected,
        Executed,
    }

    /// History of recent epochs for analytics and tracking.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct EpochHistory<T: Config> {
        pub epoch_number: u32,
        pub active_validators: BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxValidators>,
        pub score_snapshot: BoundedVec
            <(<T as frame_system::Config>::AccountId, u64), <T as Config>::MaxValidators>,
        pub inference_summary: BoundedVec
            <(<T as frame_system::Config>::AccountId, Option<u64>), <T as Config>::MaxValidators>,
    }

    /// Non-generic struct for runtime API (AccountId = T::AccountId, all BoundedVecs use configurable size).
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct RuntimeEpochHistory<AccountId> {
        pub epoch_number: u32,
        pub active_validators: BoundedVec<AccountId, ConstU32<100>>, // Keep as reasonable default for runtime API compatibility
        pub score_snapshot: BoundedVec<(AccountId, u64), ConstU32<100>>,
        pub inference_summary: BoundedVec<(AccountId, Option<u64>), ConstU32<100>>,
    }

    /// Comprehensive validator metadata information
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorMetadataInfo {
        pub name: BoundedVec<u8, ConstU32<32>>,
        pub website: Option<BoundedVec<u8, ConstU32<64>>>,
        pub contact: Option<BoundedVec<u8, ConstU32<64>>>,
        pub description: Option<BoundedVec<u8, ConstU32<128>>>,
        pub location: Option<BoundedVec<u8, ConstU32<32>>>,
        pub commission_rate: Option<u32>, // Percentage (0 to T::MaxCommissionRate)
        pub min_stake_required: Option<u128>,
        pub created_at: u64, // Timestamp in milliseconds
        pub updated_at: u64, // Timestamp in milliseconds
    }

    /// Performance record for tracking validator performance over time
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct PerformanceRecord {
        pub epoch: u32,
        pub blocks_authored: u32,
        pub blocks_missed: u32,
        pub uptime_percentage: u32, // 0 to T::PercentagePrecision
        pub inference_score: u64,
        pub stake_score: u64,
        pub final_score: u64,
        pub participation_rate: u32, // 0 to T::PercentagePrecision
        pub timestamp: u64, // Timestamp in milliseconds
    }

    impl<T: Config> From<EpochHistory<T>> for RuntimeEpochHistory<<T as frame_system::Config>::AccountId> {
        fn from(e: EpochHistory<T>) -> Self {
            RuntimeEpochHistory {
                epoch_number: e.epoch_number,
                active_validators: BoundedVec::truncate_from(e.active_validators.into_inner()),
                score_snapshot: BoundedVec::truncate_from(e.score_snapshot.into_inner()),
                inference_summary: BoundedVec::truncate_from(e.inference_summary.into_inner()),
            }
        }
    }

    
    #[pallet::config]
    pub trait Config: frame_system::Config + pos::Config + poi::Config + TypeInfo + fmt::Debug {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        // Validator set configuration
        /// Maximum number of validators that can be registered in the network.
        /// 
        /// This limit prevents unbounded growth of the validator set which could
        /// impact network performance and storage requirements. When this limit
        /// is reached, new validators cannot join until existing ones leave.
        /// 
        /// Typical values: 100-1000 depending on network requirements.
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        
        /// Minimum number of active validators required for network operation.
        /// 
        /// The network will not function properly if the active validator count
        /// falls below this threshold. This ensures sufficient decentralization
        /// and consensus security at all times.
        /// 
        /// Should be set to a value that maintains network security while
        /// allowing for validator churn and temporary outages.
        #[pallet::constant]
        type MinActiveValidators: Get<u32>;
        
        /// Maximum number of epoch history records to maintain in storage.
        /// 
        /// Controls the size of the circular buffer storing recent epoch data.
        /// Higher values provide more historical data for analysis but consume
        /// more storage space. When the limit is reached, oldest records are
        /// automatically removed.
        /// 
        /// Typical values: 24-168 (representing hours to weeks of history).
        #[pallet::constant]
        type MaxEpochHistory: Get<u32>;
        
        // Scoring weights and thresholds
        /// Default weight for Proof-of-Stake (PoS) component in final score calculation.
        /// 
        /// Determines the initial influence of validator stake on their consensus score.
        /// Higher values give more importance to economic stake, while lower values
        /// reduce the impact of wealth on consensus participation.
        /// 
        /// Used in: final_score = (stake_score * pos_weight + inference_score * poi_weight) / precision
        /// Typical values: 3000-7000 (30%-70% when using 10000 as 100%).
        #[pallet::constant]
        type DefaultPosWeight: Get<u64>;
        
        /// Default weight for Proof-of-Inference (PoI) component in final score calculation.
        /// 
        /// Determines the initial influence of AI/ML inference performance on consensus score.
        /// Higher values emphasize technical AI capabilities, while lower values reduce
        /// the impact of inference accuracy on consensus participation.
        /// 
        /// Used in: final_score = (stake_score * pos_weight + inference_score * poi_weight) / precision
        /// Typical values: 3000-7000 (30%-70% when using 10000 as 100%).
        #[pallet::constant]
        type DefaultPoiWeight: Get<u64>;
        
        /// Minimum score threshold for validator participation in consensus.
        /// 
        /// Validators with scores below this threshold are automatically removed
        /// from the active set and cannot participate in block production until
        /// their performance improves above this level.
        /// 
        /// Prevents low-performing validators from degrading network quality.
        /// Typical values: 10-50 depending on scoring scale.
        #[pallet::constant]
        type MinValidatorScore: Get<u32>;
        
        /// Maximum possible score a validator can achieve.
        /// 
        /// Provides an upper bound for validator scores to prevent overflow
        /// and ensure consistent score calculations across the network.
        /// Also used for percentage calculations and score normalization.
        /// 
        /// Typical values: 1000-10000 depending on desired precision.
        #[pallet::constant]
        type MaxValidatorScore: Get<u64>;
        
        // Score decay and activity parameters
        /// Percentage of score lost per epoch for inactive validators.
        /// 
        /// Inactive validators gradually lose score to encourage active participation
        /// and prevent dormant validators from maintaining high rankings indefinitely.
        /// Applied as: new_score = old_score * (100 - decay_percent) / 100
        /// 
        /// Typical values: 5-20 (5%-20% decay per epoch of inactivity).
        #[pallet::constant]
        type ValidatorScoreDecay: Get<u32>;
        
        /// Maximum number of consecutive epochs a validator can be inactive.
        /// 
        /// After this many epochs of inactivity, validators may be automatically
        /// ejected from the validator set to maintain network health and prevent
        /// accumulation of dead weight in the validator registry.
        /// 
        /// Typical values: 3-10 epochs depending on epoch length.
        #[pallet::constant]
        type MaxInactiveEpochs: Get<u32>;
        
        /// Block interval for applying score decay to inactive validators.
        /// 
        /// Determines how frequently the pallet checks for and applies score
        /// decay to validators who haven't been active recently. More frequent
        /// checks provide better responsiveness but consume more computational resources.
        /// 
        /// Typical values: 10-100 blocks (1-10 minutes at 6s block time).
        #[pallet::constant]
        type ScoreDecayInterval: Get<u32>; // blocks
        
        /// Block interval for updating validator participation rate metrics.
        /// 
        /// Controls how often the pallet recalculates participation rates,
        /// block production statistics, and other activity-based metrics.
        /// Affects the responsiveness of performance tracking.
        /// 
        /// Typical values: 50-200 blocks (5-20 minutes at 6s block time).
        #[pallet::constant]
        type ParticipationUpdateInterval: Get<u32>; // blocks
        
        /// Block interval for checking and handling underperforming validators.
        /// 
        /// Determines how frequently the pallet evaluates validator performance
        /// and takes action against consistently poor performers. This includes
        /// score penalties, warnings, and potential ejection decisions.
        /// 
        /// Typical values: 50-500 blocks (5-50 minutes at 6s block time).
        #[pallet::constant]
        type UnderperformanceCheckInterval: Get<u32>; // blocks
        
        /// Block interval for generating automatic validator management proposals.
        /// 
        /// Controls how often the pallet creates governance proposals for
        /// validator rewards, penalties, or set changes based on performance
        /// analysis and network health metrics.
        /// 
        /// Typical values: 100-1000 blocks (10-100 minutes at 6s block time).
        #[pallet::constant]
        type ValidatorProposalInterval: Get<u32>; // blocks
        
        /// Block interval for updating comprehensive health and performance metrics.
        /// 
        /// Determines how frequently the pallet performs detailed health checks,
        /// updates performance histories, and generates network statistics.
        /// Less frequent updates reduce computational overhead.
        /// 
        /// Typical values: 500-2000 blocks (50-200 minutes at 6s block time).
        #[pallet::constant]
        type HealthMetricsInterval: Get<u32>; // blocks
        
        /// Block interval for off-chain worker execution and data collection.
        /// 
        /// Controls how often off-chain workers run to collect external data,
        /// perform inference operations, and submit results back to the chain.
        /// Affects the freshness of off-chain data integration.
        /// 
        /// Typical values: 5-50 blocks (30 seconds to 5 minutes at 6s block time).
        #[pallet::constant]
        type OffchainWorkerInterval: Get<u32>; // blocks
        
        /// Cooldown period in blocks before validators can leave after requesting.
        /// 
        /// Prevents rapid validator set changes that could destabilize consensus
        /// by requiring validators to continue participating for a minimum period
        /// after requesting to leave. Provides time for orderly transitions.
        /// 
        /// Typical values: 1000-10000 blocks (2.8-28 hours at 6s block time).
        #[pallet::constant]
        type LeaveCooldown: Get<u32>; // blocks
        
        /// Number of blocks per epoch for validator set management and scoring.
        /// 
        /// Defines the fundamental time unit for validator operations including
        /// score updates, set rotations, reward distributions, and governance
        /// actions. Longer epochs provide stability, shorter epochs enable
        /// faster adaptation to changing conditions.
        /// 
        /// Typical values: 1200-7200 blocks (2-12 hours at 6s block time).
        #[pallet::constant]
        type EpochLength: Get<u32>; // blocks per epoch
        
        // Block authorship rewards and penalties
        /// Score boost awarded to validators for successfully authoring blocks.
        /// 
        /// Incentivizes consistent block production by rewarding validators
        /// who successfully create and submit valid blocks when selected.
        /// The boost is added to the validator's performance score.
        /// 
        /// Higher values encourage active participation in block production,
        /// while lower values reduce the impact of block authorship on scoring.
        /// 
        /// Typical values: 5-50 score points per successful block.
        #[pallet::constant]
        type BlockAuthorshipBoost: Get<u64>;
        
        /// Score penalty applied to validators for missing assigned block slots.
        /// 
        /// Discourages validator downtime and unreliability by penalizing
        /// validators who fail to produce blocks when selected as the author.
        /// The penalty is subtracted from the validator's performance score.
        /// 
        /// Should be balanced to discourage poor performance without being
        /// so harsh as to cause cascading failures during network issues.
        /// 
        /// Typical values: 2-20 score points per missed block.
        #[pallet::constant]
        type MissedBlockPenalty: Get<u64>;
        
        // Inference scoring parameters
        /// Score boost for low-quality inference results that meet minimum standards.
        /// 
        /// Applied to validators who submit inference results with confidence
        /// scores above the minimum threshold but below the medium threshold.
        /// Encourages participation while maintaining quality standards.
        /// 
        /// Typical values: 1-5 score points per low-quality inference.
        #[pallet::constant]
        type InferenceBoostLow: Get<u64>;
        
        /// Score boost for medium-quality inference results.
        /// 
        /// Applied to validators who submit inference results with confidence
        /// scores between the low and high thresholds. Represents the standard
        /// reward for acceptable inference performance.
        /// 
        /// Typical values: 3-10 score points per medium-quality inference.
        #[pallet::constant]
        type InferenceBoostMedium: Get<u64>;
        
        /// Score boost for high-quality inference results exceeding excellence threshold.
        /// 
        /// Applied to validators who submit inference results with confidence
        /// scores above the high threshold. Rewards exceptional AI/ML performance
        /// and encourages validators to optimize their inference capabilities.
        /// 
        /// Typical values: 5-20 score points per high-quality inference.
        #[pallet::constant]
        type InferenceBoostHigh: Get<u64>;
        
        /// Score penalty for poor-quality inference results below minimum standards.
        /// 
        /// Applied to validators who submit inference results with very low
        /// confidence scores or incorrect results. Discourages spam submissions
        /// and maintains network inference quality.
        /// 
        /// Typical values: 1-3 score points penalty per poor inference.
        #[pallet::constant]
        type InferencePenaltyLow: Get<u64>;
        
        /// Score penalty for consistently poor inference performance.
        /// 
        /// Applied to validators who repeatedly submit low-quality inference
        /// results or demonstrate unreliable AI/ML capabilities. Stronger
        /// penalty than low penalty to address persistent poor performance.
        /// 
        /// Typical values: 2-8 score points penalty per medium-level failure.
        #[pallet::constant]
        type InferencePenaltyMedium: Get<u64>;
        
        /// Score penalty for severely poor inference performance or malicious behavior.
        /// 
        /// Applied to validators who submit obviously incorrect results,
        /// attempt to manipulate inference outcomes, or demonstrate gross
        /// negligence in AI/ML operations. Strongest penalty level.
        /// 
        /// Typical values: 5-25 score points penalty per severe failure.
        #[pallet::constant]
        type InferencePenaltyHigh: Get<u64>;
        
        /// Confidence score threshold separating low-quality from medium-quality inference.
        /// 
        /// Inference results with confidence scores below this threshold receive
        /// low-quality treatment (minimal rewards or penalties). Results above
        /// this threshold are considered acceptable quality.
        /// 
        /// Typical values: 60-80 (representing 60%-80% confidence).
        #[pallet::constant]
        type InferenceConfidenceThresholdLow: Get<u32>;
        
        /// Confidence score threshold separating medium-quality from high-quality inference.
        /// 
        /// Inference results with confidence scores above this threshold receive
        /// high-quality rewards and recognition. Sets the bar for exceptional
        /// AI/ML performance in the network.
        /// 
        /// Typical values: 85-95 (representing 85%-95% confidence).
        #[pallet::constant]
        type InferenceConfidenceThresholdHigh: Get<u32>;
        
        // Governance and slashing parameters
        /// Maximum score penalty that can be applied through slashing actions.
        /// 
        /// Caps the score reduction from slashing to prevent excessive punishment
        /// that could permanently damage a validator's standing. Provides a
        /// balance between accountability and recovery opportunity.
        /// 
        /// Typical values: 50-200 score points maximum penalty.
        #[pallet::constant]
        type MaxSlashPenalty: Get<u64>;
        
        /// Maximum score boost that can be applied through reward actions.
        /// 
        /// Caps the score increase from rewards to prevent excessive inflation
        /// of validator scores and maintain competitive balance. Ensures rewards
        /// are meaningful but not game-breaking.
        /// 
        /// Typical values: 20-100 score points maximum boost.
        #[pallet::constant]
        type MaxRewardBoost: Get<u64>;
        
        /// Divisor for calculating score penalties from slashed stake amounts.
        /// 
        /// Used in: score_penalty = slashed_amount / divisor
        /// Higher divisors result in smaller score penalties for the same
        /// slashed amount, while lower divisors increase the score impact.
        /// 
        /// Typical values: 1000-10000 (meaning 1 score point per 1000-10000 units slashed).
        #[pallet::constant]
        type SlashPenaltyDivisor: Get<u64>;
        
        /// Divisor for calculating score boosts from reward amounts.
        /// 
        /// Used in: score_boost = reward_amount / divisor
        /// Higher divisors result in smaller score boosts for the same
        /// reward amount, while lower divisors increase the score impact.
        /// 
        /// Typical values: 1000-10000 (meaning 1 score point per 1000-10000 units rewarded).
        #[pallet::constant]
        type RewardBoostDivisor: Get<u64>;
        
        /// Percentage of validator stake to slash for serious misbehavior.
        /// 
        /// Applied when validators are found guilty of malicious behavior,
        /// consensus violations, or other serious infractions. Expressed
        /// as a percentage (0-100) of the validator's total staked amount.
        /// 
        /// Typical values: 5-30 (5%-30% of stake slashed).
        #[pallet::constant]
        type SlashPercent: Get<u32>; // Percentage of stake to slash (0-100)
        
        /// Default reward amount for validators in governance proposals.
        /// 
        /// Used when reward proposals don't specify a custom amount,
        /// providing a standard reward value for validator incentives.
        /// Should be meaningful enough to encourage good behavior.
        /// 
        /// Typical values: 1000-100000 units depending on token economics.
        #[pallet::constant]
        type ValidatorReward: Get<<Self as Config>::Balance>; // Default reward amount for validators
        
        // Validator metadata limits
        /// Maximum length in bytes for validator display names.
        /// 
        /// Prevents abuse of the naming system while allowing reasonable
        /// length names for validator identification. Names are UTF-8 encoded
        /// so actual character count may be less than byte count.
        /// 
        /// Typical values: 32-128 bytes.
        #[pallet::constant]
        type MaxValidatorNameLength: Get<u32>;
        
        /// Maximum length in bytes for validator website URLs.
        /// 
        /// Allows validators to provide website links for additional information
        /// while preventing storage abuse. Should accommodate typical URL lengths
        /// including domain names and paths.
        /// 
        /// Typical values: 64-256 bytes.
        #[pallet::constant]
        type MaxValidatorWebsiteLength: Get<u32>;
        
        /// Maximum length in bytes for validator contact information.
        /// 
        /// Enables validators to provide contact details (email, social media)
        /// for community engagement while limiting storage usage. Should
        /// accommodate email addresses and social media handles.
        /// 
        /// Typical values: 64-128 bytes.
        #[pallet::constant]
        type MaxValidatorContactLength: Get<u32>;
        
        /// Maximum length in bytes for validator description text.
        /// 
        /// Allows validators to provide detailed descriptions of their services,
        /// capabilities, and value propositions while preventing storage abuse.
        /// Should accommodate meaningful descriptions without excessive length.
        /// 
        /// Typical values: 128-512 bytes.
        #[pallet::constant]
        type MaxValidatorDescriptionLength: Get<u32>;
        
        /// Maximum length in bytes for validator location information.
        /// 
        /// Enables validators to specify their geographic location for
        /// transparency and network distribution analysis. Should accommodate
        /// city, country, or region names.
        /// 
        /// Typical values: 32-64 bytes.
        #[pallet::constant]
        type MaxValidatorLocationLength: Get<u32>;
        
        /// Maximum number of performance records to maintain per validator.
        /// 
        /// Controls the size of the performance history circular buffer.
        /// Higher values provide more detailed historical analysis but
        /// consume more storage space per validator.
        /// 
        /// Typical values: 50-200 records.
        #[pallet::constant]
        type MaxPerformanceHistoryLength: Get<u32>;
        
        /// Maximum number of epoch statistics to maintain per validator.
        /// 
        /// Controls the size of the validator's epoch history buffer.
        /// Provides historical context for validator performance evaluation
        /// while limiting storage growth.
        /// 
        /// Typical values: 10-50 epochs.
        #[pallet::constant]
        type MaxValidatorHistoryLength: Get<u32>;
        
        /// Maximum commission rate validators can charge in basis points.
        /// 
        /// Limits the fees validators can charge for their services to
        /// prevent excessive extraction from delegators or network participants.
        /// Expressed in basis points where 10000 = 100%.
        /// 
        /// Typical values: 1000-5000 (10%-50% maximum commission).
        #[pallet::constant]
        type MaxCommissionRate: Get<u32>; // basis points (10000 = 100%)
        
        // Percentage calculation precision
        /// Precision factor for percentage calculations throughout the pallet.
        /// 
        /// Used as the denominator in percentage calculations to provide
        /// fine-grained precision. Higher values enable more precise
        /// calculations but may increase computational overhead.
        /// 
        /// Common values:
        /// - 100: 1% precision (whole percentages only)
        /// - 1000: 0.1% precision (one decimal place)
        /// - 10000: 0.01% precision (basis points, two decimal places)
        #[pallet::constant]
        type PercentagePrecision: Get<u32>; // 10000 for basis points (0.01% precision)
        
        // Misbehavior reporting
        /// Maximum length in bytes for misbehavior evidence submissions.
        /// 
        /// Limits the size of evidence that can be submitted with misbehavior
        /// reports to prevent storage abuse while allowing sufficient space
        /// for cryptographic proofs, screenshots, logs, and other evidence.
        /// 
        /// Typical values: 1024-8192 bytes (1-8 KB per evidence submission).
        #[pallet::constant]
        type MaxEvidenceLength: Get<u32>;
        
        /// Number of independent misbehavior reports required to trigger automatic slashing.
        /// 
        /// Prevents false accusations by requiring multiple independent reports
        /// before taking punitive action against a validator. Higher thresholds
        /// provide more protection against coordinated attacks but may delay
        /// action against genuine misbehavior.
        /// 
        /// Typical values: 3-10 independent reports.
        #[pallet::constant]
        type MisbehaviorSlashThreshold: Get<u32>; // Number of reports needed to trigger slash
        
        // Off-chain worker configuration
        /// Timeout duration for off-chain worker operations in milliseconds.
        /// 
        /// Sets the maximum time off-chain workers can spend on external
        /// operations before timing out. Prevents workers from hanging
        /// indefinitely on slow or unresponsive external services.
        /// 
        /// Typical values: 10000-60000 (10-60 seconds).
        #[pallet::constant]
        type OffchainWorkerTimeout: Get<u64>; // milliseconds
        
        /// Estimated average block production time in milliseconds.
        /// 
        /// Used by off-chain workers and timing calculations to estimate
        /// when certain block-based events will occur. Should match the
        /// network's actual average block time for accurate predictions.
        /// 
        /// Typical values: 6000-12000 (6-12 seconds per block).
        #[pallet::constant]
        type EstimatedBlockTime: Get<u64>; // milliseconds
        
        // Performance thresholds
        /// Minimum performance score threshold for acceptable validator behavior.
        /// 
        /// Validators with scores below this threshold are considered underperforming
        /// and may be subject to penalties, warnings, or temporary removal from
        /// the active set. Sets the baseline for acceptable network participation.
        /// 
        /// Typical values: 20-50 score points.
        #[pallet::constant]
        type MinPerformanceScore: Get<u64>; // Minimum score for good performance (30)
        
        /// Performance score threshold for high-performance validator recognition.
        /// 
        /// Validators with scores above this threshold are eligible for enhanced
        /// rewards, priority treatment, and recognition as high-quality network
        /// participants. Encourages excellence in validator operations.
        /// 
        /// Typical values: 70-90 score points.
        #[pallet::constant]
        type HighPerformanceScore: Get<u64>; // Score for high performance rewards (80)
        
        /// Minimum participation rate percentage for acceptable validator behavior.
        /// 
        /// Validators with participation rates below this threshold are considered
        /// unreliable and may face penalties or removal. Expressed as a percentage
        /// of assigned opportunities that were successfully fulfilled.
        /// 
        /// Typical values: 50-80 (50%-80% minimum participation).
        #[pallet::constant]
        type MinParticipationRate: Get<u32>; // Minimum participation rate percentage (50)
        
        /// High participation rate threshold for exceptional validator recognition.
        /// 
        /// Validators with participation rates above this threshold are eligible
        /// for enhanced rewards and recognition as highly reliable network
        /// participants. Encourages consistent availability and performance.
        /// 
        /// Typical values: 85-95 (85%-95% high participation).
        #[pallet::constant]
        type HighParticipationRate: Get<u32>; // High participation rate percentage (90)
        
        /// Maximum number of blocks a validator can miss before facing penalties.
        /// 
        /// Once a validator misses this many blocks, they become subject to
        /// score penalties, warnings, or temporary removal from the active set.
        /// Balances tolerance for occasional issues with network reliability needs.
        /// 
        /// Typical values: 5-20 blocks.
        #[pallet::constant]
        type MaxMissedBlocks: Get<u32>; // Maximum missed blocks before penalty (10)
        
        /// Stricter missed block limit for high-performing validators.
        /// 
        /// High-performing validators are held to higher standards and face
        /// penalties after missing fewer blocks. Maintains quality expectations
        /// for validators who receive enhanced rewards and recognition.
        /// 
        /// Typical values: 1-5 blocks.
        #[pallet::constant]
        type MaxMissedBlocksHigh: Get<u32>; // Maximum missed blocks for high performers (2)
        
        /// Score threshold defining a "healthy" validator in good standing.
        /// 
        /// Used for network health assessments, validator categorization,
        /// and determining eligibility for various network operations.
        /// Represents the score level for stable, reliable validators.
        /// 
        /// Typical values: 40-70 score points.
        #[pallet::constant]
        type HealthyValidatorScore: Get<u64>; // Score threshold for healthy validator (50)
        
        /// Participation rate threshold defining a "healthy" validator.
        /// 
        /// Used alongside score thresholds to assess overall validator health
        /// and reliability. Validators meeting both score and participation
        /// thresholds are considered healthy network participants.
        /// 
        /// Typical values: 70-90 (70%-90% participation for health).
        #[pallet::constant]
        type HealthyParticipationRate: Get<u32>; // Participation rate for healthy validator (80)
        
        /// Maximum missed blocks allowed for a validator to be considered healthy.
        /// 
        /// Works with other health metrics to provide a comprehensive assessment
        /// of validator reliability. Healthy validators should miss very few
        /// blocks to maintain their good standing status.
        /// 
        /// Typical values: 2-10 blocks.
        #[pallet::constant]
        type HealthyMissedBlocksMax: Get<u32>; // Max missed blocks for healthy validator (5)
        
        // Score calculation thresholds
        /// Minimum absolute score change required to trigger validator set reordering.
        /// 
        /// When a validator's score changes by at least this amount, the system
        /// may reorder the validator set to maintain proper ranking. Prevents
        /// excessive reordering from minor score fluctuations while ensuring
        /// significant changes are reflected in validator rankings.
        /// 
        /// Typical values: 100-2000 score points.
        #[pallet::constant]
        type ScoreChangeThreshold: Get<u64>; // Minimum score change to trigger resort (1000)
        
        /// Minimum percentage score change required to trigger validator set reordering.
        /// 
        /// Works alongside the absolute threshold to determine when reordering
        /// is necessary. Uses percentage to make the threshold adaptive to
        /// different score ranges and validator performance levels.
        /// 
        /// Typical values: 5-20 (5%-20% change required).
        #[pallet::constant]
        type ScoreChangePercentage: Get<u32>; // Percentage change to trigger resort (10%)
        
        /// Minimum score improvement required for validator promotion considerations.
        /// 
        /// When a validator's score improves by at least this amount, they may
        /// be considered for promotion to higher tiers, enhanced rewards, or
        /// priority status. Recognizes significant performance improvements.
        /// 
        /// Typical values: 500-2000 score points.
        #[pallet::constant]
        type ScoreImprovementThreshold: Get<u64>; // Minimum improvement for validator promotion (1000)
        
        /// Minimum percentage score improvement for validator promotion considerations.
        /// 
        /// Works with the absolute improvement threshold to identify validators
        /// who have significantly enhanced their performance and deserve
        /// recognition or advancement in the validator hierarchy.
        /// 
        /// Typical values: 10-25 (10%-25% improvement required).
        #[pallet::constant]
        type ScoreImprovementPercentage: Get<u32>; // Percentage improvement for promotion (10%)
        
        // Contribution balance thresholds
        /// Maximum percentage that PoS (Proof-of-Stake) can contribute to final scores.
        /// 
        /// Prevents the consensus mechanism from becoming too heavily weighted
        /// toward economic stake by capping the PoS influence. Ensures that
        /// AI/ML performance (PoI) maintains meaningful impact on validator rankings.
        /// 
        /// Typical values: 70-95 (70%-95% maximum PoS influence).
        #[pallet::constant]
        type MaxPosContribution: Get<u32>; // Max PoS contribution percentage (90%)
        
        /// Maximum percentage that PoI (Proof-of-Inference) can contribute to final scores.
        /// 
        /// Prevents the consensus mechanism from becoming too heavily weighted
        /// toward AI/ML performance by capping the PoI influence. Ensures that
        /// economic stake (PoS) maintains meaningful impact on validator rankings.
        /// 
        /// Typical values: 70-95 (70%-95% maximum PoI influence).
        #[pallet::constant]
        type MaxPoiContribution: Get<u32>; // Max PoI contribution percentage (90%)
        
        /// Threshold for warning about imbalanced PoS/PoI contribution ratios.
        /// 
        /// When either PoS or PoI contribution exceeds this percentage of the
        /// total score calculation, the system may issue warnings or take
        /// corrective action to maintain balanced consensus participation.
        /// 
        /// Typical values: 75-90 (75%-90% triggers imbalance warnings).
        #[pallet::constant]
        type ImbalanceWarningThreshold: Get<u32>; // Threshold for imbalance warning (85%)
        
        // Block processing intervals
        /// Block interval for checking and processing validator leave requests.
        /// 
        /// Determines how frequently the system checks for expired leave requests
        /// and processes validators who have completed their cooldown periods.
        /// More frequent checks provide faster response but consume more resources.
        /// 
        /// Typical values: 5-50 blocks (30 seconds to 5 minutes at 6s block time).
        #[pallet::constant]
        type LeaveRequestCheckInterval: Get<u32>; // Blocks between leave request checks (10)
        
        /// Block interval for updating general validator and network metrics.
        /// 
        /// Controls how often the system updates various performance metrics,
        /// statistics, and monitoring data. Affects the freshness of metrics
        /// data available through APIs and internal calculations.
        /// 
        /// Typical values: 5-100 blocks (30 seconds to 10 minutes at 6s block time).
        #[pallet::constant]
        type MetricsUpdateInterval: Get<u32>; // Blocks between metrics updates (10)
        
        /// Block interval for refreshing validator score calculations.
        /// 
        /// Determines how often the system recalculates validator scores based
        /// on current performance data, stake amounts, and inference results.
        /// More frequent updates provide better responsiveness to performance changes.
        /// 
        /// Typical values: 10-200 blocks (1-20 minutes at 6s block time).
        #[pallet::constant]
        type ScoreRefreshInterval: Get<u32>; // Blocks between score refreshes (50)
        
        /// Block interval for generating detailed logging and diagnostic information.
        /// 
        /// Controls how frequently the system outputs comprehensive logs about
        /// validator performance, network health, and internal state. Detailed
        /// logging provides valuable debugging information but can be verbose.
        /// 
        /// Typical values: 50-500 blocks (5-50 minutes at 6s block time).
        #[pallet::constant]
        type DetailedLoggingInterval: Get<u32>; // Blocks between detailed logging (100)
        
        /// Block interval for checking PoS/PoI contribution balance and issuing warnings.
        /// 
        /// Determines how often the system evaluates whether the consensus
        /// mechanism is properly balanced between stake-based and inference-based
        /// contributions. Helps maintain the hybrid nature of the consensus.
        /// 
        /// Typical values: 100-1000 blocks (10-100 minutes at 6s block time).
        #[pallet::constant]
        type ImbalanceCheckInterval: Get<u32>; // Blocks between imbalance checks (500)
        
        // Validator set limits
        /// Number of top-performing validators to display in logs and API responses.
        /// 
        /// Limits the verbosity of validator rankings in logs while still providing
        /// visibility into the highest-performing network participants. Used for
        /// monitoring dashboards, API responses, and operational visibility.
        /// 
        /// Typical values: 3-20 validators.
        #[pallet::constant]
        type TopValidatorsDisplayCount: Get<u32>; // Number of top validators to display (5)
        
        /// Number of validators to sample when performing network health checks.
        /// 
        /// Determines the sample size for statistical health assessments of the
        /// validator network. Larger samples provide more accurate health metrics
        /// but require more computational resources for analysis.
        /// 
        /// Typical values: 5-50 validators.
        #[pallet::constant]
        type HealthCheckSampleSize: Get<u32>; // Number of validators to sample for health (5)
        
        // Percentage constants
        /// Value representing 100% for percentage calculations throughout the pallet.
        /// 
        /// Used as the base value for percentage calculations, allowing the system
        /// to work with different precision levels. Should match the precision
        /// factor used elsewhere in the pallet for consistency.
        /// 
        /// Common values: 100 (whole percentages) or 10000 (basis points).
        #[pallet::constant]
        type FullPercentage: Get<u32>; // 100% value
        
        /// Percentage threshold for identifying high-performing validators.
        /// 
        /// Validators scoring above this percentage of the maximum possible score
        /// are considered high performers and eligible for enhanced rewards and
        /// recognition. Sets the bar for exceptional network participation.
        /// 
        /// Typical values: 70-90 (70%-90% of maximum score).
        #[pallet::constant]
        type HighPerformancePercentage: Get<u32>; // 80% for high performance threshold
        
        /// Percentage of validators eligible for top performer rewards.
        /// 
        /// Determines what fraction of the validator set receives the highest
        /// tier of rewards and recognition. Smaller percentages make top performer
        /// status more exclusive and valuable.
        /// 
        /// Typical values: 10-30 (10%-30% of validators).
        #[pallet::constant]
        type TopPerformerPercentage: Get<u32>; // 20% for top performer rewards
        
        // Reward distribution percentages
        /// Percentage of total reward pool allocated to base rewards for all validators.
        /// 
        /// Determines what portion of epoch rewards goes to the base reward pool
        /// that is distributed equally among all active validators. Provides a
        /// foundation reward level regardless of performance differences.
        /// 
        /// Typical values: 50-70 (50%-70% of total rewards).
        #[pallet::constant]
        type BaseRewardPercentage: Get<u32>; // Base reward percentage for all validators
        
        /// Percentage of total reward pool allocated to performance-based rewards.
        /// 
        /// Determines what portion of epoch rewards goes to the performance pool
        /// that is distributed among validators meeting high performance thresholds.
        /// Incentivizes consistent good performance above baseline levels.
        /// 
        /// Typical values: 20-35 (20%-35% of total rewards).
        #[pallet::constant]
        type PerformanceRewardPercentage: Get<u32>; // Additional reward for high performers
        
        /// Percentage of total reward pool allocated to top performer rewards.
        /// 
        /// Determines what portion of epoch rewards goes to the top performer pool
        /// that is distributed among the highest-ranking validators. Provides
        /// maximum incentive for exceptional performance and network contribution.
        /// 
        /// Typical values: 10-25 (10%-25% of total rewards).
        #[pallet::constant]
        type TopPerformerRewardPercentage: Get<u32>; // Additional reward for top performers
        
        // Stake and balance configuration
        /// Minimum stake amount required for validator participation.
        /// 
        /// Sets the economic barrier to entry for becoming a validator, ensuring
        /// participants have sufficient "skin in the game" for network security.
        /// Validators must reserve at least this amount to join and maintain
        /// their position in the validator set.
        /// 
        /// The stake is reserved (locked) from the validator's balance and can
        /// be slashed for misbehavior or poor performance. It is unreserved
        /// when the validator leaves after completing the cooldown period.
        /// 
        /// Typical values: 1000-1000000 units depending on token economics.
        #[pallet::constant]
        type MinStake: Get<<Self as Config>::Balance>;

        // Trust score calculation weights
        /// Weight for uptime component in trust score calculation.
        #[pallet::constant]
        type TrustScoreUptimeWeight: Get<u64>;

        /// Weight for inference success component in trust score calculation.
        #[pallet::constant]
        type TrustScoreInferenceWeight: Get<u64>;

        /// Weight for slashing history component in trust score calculation.
        #[pallet::constant]
        type TrustScoreSlashingWeight: Get<u64>;

        /// Maximum trust score value.
        #[pallet::constant]
        type MaxTrustScore: Get<u64>;

        // Constants for hardcoded values
        /// Maximum size for validator history bounded vector.
        #[pallet::constant]
        type MaxValidatorHistorySize: Get<u32>;

        /// Maximum size for validator name bounded vector.
        #[pallet::constant]
        type MaxValidatorNameSize: Get<u32>;

        /// Maximum size for runtime API bounded vectors.
        #[pallet::constant]
        type MaxRuntimeApiBoundedVecSize: Get<u32>;

        /// Maximum size for proposal action bounded vectors.
        #[pallet::constant]
        type MaxProposalActionBoundedVecSize: Get<u32>;

        /// Error code for author not active validation.
        #[pallet::constant]
        type AuthorNotActiveErrorCode: Get<u8>;

        /// Error code for author mismatch validation.
        #[pallet::constant]
        type AuthorMismatchErrorCode: Get<u8>;

        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + serde::Serialize + for<'de> serde::Deserialize<'de>;
        type Currency: Currency<Self::AccountId, Balance = <Self as pallet::Config>::Balance> + ReservableCurrency<Self::AccountId>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Stores state for each validator.
    /// Comprehensive state tracking for each validator in the network.
    ///
    /// This storage map maintains detailed information about each validator including:
    /// - Current performance scores (stake, inference, final)
    /// - Block authorship statistics (authored and missed blocks)
    /// - Historical performance data across epochs
    /// - Last active epoch for decay calculations
    ///
    /// The ValidatorState struct contains both current metrics and historical data
    /// to enable comprehensive performance evaluation and score calculations.
    ///
    /// # Key: T::AccountId - The validator's account identifier
    /// # Value: ValidatorState<T> - Complete validator state information
    #[pallet::storage]
    #[pallet::getter(fn validator_states)]
    pub type ValidatorStates<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorState<T>,
        OptionQuery,
    >;

    /// Current Proof-of-Stake (PoS) weight used in final score calculations.
    /// 
    /// This value determines how much influence a validator's stake has on their
    /// final consensus score. The final score is calculated as:
    /// `final_score = (stake_score * pos_weight + inference_score * poi_weight) / precision`
    /// 
    /// Higher PoS weight values give more importance to validator stake amounts,
    /// while lower values reduce the influence of stake on consensus participation.
    /// 
    /// Default value is set via `T::DefaultPosWeight` configuration constant.
    /// Can be updated through governance or root calls to `update_consensus_weights`.
    /// 
    /// # Value: u64 - Weight multiplier for PoS scores (typically 0-10000 for percentage-based calculations)
    #[pallet::storage]
    #[pallet::getter(fn pos_weight)]
    pub type PosWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Current Proof-of-Inference (PoI) weight used in final score calculations.
    /// 
    /// This value determines how much influence a validator's inference performance
    /// has on their final consensus score. The final score is calculated as:
    /// `final_score = (stake_score * pos_weight + inference_score * poi_weight) / precision`
    /// 
    /// Higher PoI weight values give more importance to validator inference accuracy
    /// and participation, while lower values reduce the influence of AI/ML performance
    /// on consensus participation.
    /// 
    /// Default value is set via `T::DefaultPoiWeight` configuration constant.
    /// Can be updated through governance or root calls to `update_consensus_weights`.
    /// 
    /// # Value: u64 - Weight multiplier for PoI scores (typically 0-10000 for percentage-based calculations)
    #[pallet::storage]
    #[pallet::getter(fn poi_weight)]
    pub type PoiWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Complete set of all registered validators in the network.
    /// 
    /// This storage contains all validators who have joined the validator set,
    /// regardless of their current active status. Validators remain in this set
    /// until they are explicitly ejected or removed through governance actions.
    /// 
    /// The set includes both active validators (currently participating in consensus)
    /// and inactive validators (temporarily not participating due to low scores,
    /// maintenance, or other reasons).
    /// 
    /// Maximum size is bounded by `T::MaxValidators` to prevent unbounded growth.
    /// New validators are added through `join_validator_set` or `join_validators` calls.
    /// 
    /// # Value: BoundedVec<T::AccountId, T::MaxValidators> - List of all registered validator accounts
    #[pallet::storage]
    #[pallet::getter(fn validator_set)]
    pub type ValidatorSet<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    /// Configuration parameters for epoch management and transitions.
    /// 
    /// Contains essential parameters that control how epochs operate:
    /// - `blocks_per_epoch`: Number of blocks in each epoch (configurable via T::EpochLength)
    /// - `min_stake`: Minimum stake required for validator participation
    /// - `max_validators`: Maximum number of validators allowed in the active set
    /// 
    /// This configuration is initialized during genesis and can be updated through
    /// governance proposals or root calls. Changes typically take effect at the
    /// next epoch boundary to ensure consistency.
    /// 
    /// The epoch system is fundamental to validator set management, score decay,
    /// reward distribution, and network parameter updates.
    /// 
    /// # Value: EpochConfig - Struct containing epoch-related configuration parameters
    #[pallet::storage]
    #[pallet::getter(fn epoch_config)]
    pub type EpochConfigStorage<T: Config> = StorageValue<_, EpochConfig, ValueQuery>;

    /// The current epoch number in the blockchain's lifecycle.
    /// 
    /// Epochs are fundamental time periods used for:
    /// - Validator set updates and rotations
    /// - Score decay calculations for inactive validators
    /// - Reward distribution cycles
    /// - Performance evaluation periods
    /// - Governance proposal execution timing
    /// 
    /// The epoch number starts at 0 during genesis and increments by 1 at each
    /// epoch transition. Epoch transitions occur automatically based on block
    /// numbers (every T::EpochLength blocks) or can be triggered manually in
    /// governance mode.
    /// 
    /// This value is used throughout the pallet for temporal calculations and
    /// ensuring operations occur at appropriate epoch boundaries.
    /// 
    /// # Value: u32 - Current epoch number (starts at 0, increments indefinitely)
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Set of validators currently active and participating in consensus.
    /// 
    /// Active validators are a subset of the total validator set who are currently
    /// eligible to participate in block production and consensus. To be active,
    /// validators must meet several criteria:
    /// - Have a final score above the minimum threshold
    /// - Maintain sufficient stake (above T::MinStake)
    /// - Not be in a cooldown period after leaving
    /// - Not be ejected due to misbehavior or poor performance
    /// 
    /// This set is updated at each epoch transition based on validator performance,
    /// stake amounts, and participation rates. The active validator set is used by
    /// the consensus engine for block author selection and validation.
    /// 
    /// Maximum size is bounded by `T::MaxValidators` to ensure network performance.
    /// 
    /// # Value: BoundedVec<T::AccountId, T::MaxValidators> - List of currently active validator accounts
    #[pallet::storage]
    #[pallet::getter(fn active_validators)]
    pub type ActiveValidators<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    /// Flag indicating whether governance mode is currently enabled.
    /// 
    /// When governance mode is enabled (true), the pallet operates with enhanced
    /// administrative capabilities:
    /// - Manual epoch transitions are allowed via `sudo_advance_epoch`
    /// - Governance proposals can be submitted and voted on
    /// - Root/sudo accounts have additional administrative powers
    /// - Certain automated processes may be disabled or modified
    /// 
    /// When disabled (false), the pallet operates in fully automated mode:
    /// - Epoch transitions occur automatically based on block numbers
    /// - Validator management is handled algorithmically
    /// - Reduced administrative intervention capabilities
    /// 
    /// This mode can be toggled by root accounts through `set_governance_mode`.
    /// Default value is typically false for decentralized operation.
    /// 
    /// # Value: bool - true if governance mode is enabled, false for automated mode
    #[pallet::storage]
    #[pallet::getter(fn governance_mode_enabled)]
    pub type GovernanceModeEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Storage for governance proposals indexed by unique proposal ID.
    /// 
    /// Contains all governance proposals that have been submitted to the network,
    /// including their current status, voting results, and proposed actions.
    /// Proposals can include:
    /// - Validator slashing actions
    /// - Validator reward distributions
    /// - Validator ejection from the network
    /// - Validator addition to the network
    /// - Multiple validator operations
    /// 
    /// Each proposal has a unique ID (generated sequentially) and contains:
    /// - Proposer account
    /// - Proposed action details
    /// - Current status (Pending, Approved, Rejected, Executed)
    /// - Vote counts (for and against)
    /// 
    /// Proposals are created through `submit_proposal` and voted on via `vote_proposal`.
    /// Approved proposals can be executed through `execute_proposal`.
    /// 
    /// # Key: u32 - Unique proposal identifier
    /// # Value: GovernanceProposal<T> - Complete proposal information and status
    #[pallet::storage]
    pub type Proposals<T: Config> = StorageMap<
        _, Blake2_128Concat, u32, GovernanceProposal<T>, OptionQuery
    >;

    /// Counter for generating unique proposal IDs.
    /// 
    /// This value is incremented each time a new governance proposal is created,
    /// ensuring that every proposal has a unique identifier. The counter starts
    /// at 0 during genesis and increments indefinitely.
    /// 
    /// Used internally by the proposal system to:
    /// - Generate unique IDs for new proposals
    /// - Maintain proposal ordering and history
    /// - Enable efficient proposal lookup and management
    /// 
    /// The ID is assigned when a proposal is submitted and never reused,
    /// providing a permanent reference for each governance action.
    /// 
    /// # Value: u32 - Next available proposal ID (starts at 0, increments with each proposal)
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Tracking system for individual votes on governance proposals.
    /// 
    /// This double map records which validators have voted on which proposals
    /// and their vote direction (for or against). Used to:
    /// - Prevent double voting by the same validator
    /// - Track voting participation rates
    /// - Maintain transparency in governance decisions
    /// - Enable vote auditing and verification
    /// 
    /// The boolean value indicates the vote direction:
    /// - true: Vote in favor of the proposal
    /// - false: Vote against the proposal
    /// - None: Validator has not voted on this proposal
    /// 
    /// Votes are cast through the `vote_proposal` extrinsic and are immutable
    /// once recorded to ensure governance integrity.
    /// 
    /// # Key1: u32 - Proposal ID
    /// # Key2: T::AccountId - Validator account who voted
    /// # Value: bool - Vote direction (true = for, false = against)
    #[pallet::storage]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, bool, OptionQuery
    >;

    /// Queue of validator join/leave requests awaiting execution at epoch boundaries.
    /// 
    /// Validator set changes are not applied immediately but are queued and processed
    /// at epoch transitions to maintain consensus stability. This storage tracks:
    /// - Join requests from new validators wanting to participate
    /// - Leave requests from existing validators wanting to exit
    /// 
    /// Actions are submitted through:
    /// - `join_validator_set`: Request to join (requires meeting minimum requirements)
    /// - `leave_validator_set`: Request to leave (subject to cooldown periods)
    /// 
    /// Pending actions are processed during epoch transitions by:
    /// - Validating that join requests still meet requirements
    /// - Applying leave requests after cooldown periods
    /// - Updating the active validator set accordingly
    /// 
    /// This delayed execution ensures validator set stability during epochs
    /// and prevents rapid changes that could destabilize consensus.
    /// 
    /// # Key: T::AccountId - Validator account requesting the action
    /// # Value: ValidatorAction - Type of action requested (Join or Leave)
    #[pallet::storage]
    #[pallet::getter(fn pending_validator_actions)]
    pub type PendingValidatorActions<T: Config> = StorageMap<
        _, Blake2_128Concat, T::AccountId, ValidatorAction, OptionQuery
    >;

    /// Join/leave intent for validators.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
    pub enum ValidatorAction {
        Join,
        Leave,
    }

    /// Data structure for inference data collected by off-chain workers
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct InferenceData<T: Config> {
        pub validator: T::AccountId,
        pub epoch: u32,
        pub inference_result: u32,
        pub confidence_score: u32,
        pub timestamp: u64,
        pub data_sources: Vec<Vec<u8>>,
    }

    /// Off-chain storage structure for PoI scores
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct OffchainPoiScore<T: Config> {
        pub validator: T::AccountId,
        pub score: u64,
        pub block_number: u32,
        pub timestamp: u64,
    }

    /// Historical record of recent epochs stored in a circular buffer.
    /// 
    /// Maintains a rolling history of the most recent epochs, including:
    /// - Epoch number and duration
    /// - Active validator set for each epoch
    /// - Aggregate performance metrics
    /// - Reward distribution summaries
    /// - Significant events and transitions
    /// 
    /// The history is stored as a bounded vector that acts as a ring buffer,
    /// automatically removing the oldest entries when the maximum size is reached.
    /// This provides efficient access to recent historical data while preventing
    /// unbounded storage growth.
    /// 
    /// Used for:
    /// - Performance trend analysis
    /// - Validator evaluation over time
    /// - Network health monitoring
    /// - Governance decision support
    /// - API queries for historical data
    /// 
    /// Maximum size is controlled by `T::MaxEpochHistory` configuration parameter.
    /// 
    /// # Value: BoundedVec<EpochHistory<T>, T::MaxEpochHistory> - Circular buffer of recent epoch records
    #[pallet::storage]
    #[pallet::getter(fn epoch_histories)]
    pub type EpochHistories<T: Config> = StorageValue<_, BoundedVec<EpochHistory<T>, T::MaxEpochHistory>, ValueQuery>;

    /// Human-readable display names for validators.
    /// 
    /// Allows validators to register a display name for better user experience
    /// in interfaces, explorers, and monitoring tools. Names are optional and
    /// purely cosmetic - they do not affect consensus or validator operations.
    /// 
    /// Names must be:
    /// - UTF-8 encoded strings
    /// - Maximum 32 bytes in length
    /// - Set by the validator through `set_validator_name` extrinsic
    /// - Unique per validator (no uniqueness enforcement across validators)
    /// 
    /// Used by:
    /// - Block explorers for validator identification
    /// - Monitoring dashboards and tools
    /// - Governance interfaces
    /// - API responses for better readability
    /// 
    /// If no name is set, interfaces should fall back to displaying the
    /// validator's account ID or a truncated version.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: BoundedVec<u8, ConstU32<32>> - UTF-8 encoded display name (max 32 bytes)
    #[pallet::storage]
    #[pallet::getter(fn validator_names)]
    pub type ValidatorNames<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u8, ConstU32<32>>,
        OptionQuery,
    >;

    /// Cumulative count of epochs each validator has been active.
    /// 
    /// Tracks the total number of epochs a validator has participated in
    /// consensus since joining the network. This metric is used for:
    /// - Long-term reliability assessment
    /// - Validator reputation scoring
    /// - Network stability analysis
    /// - Reward calculations based on participation history
    /// 
    /// The counter increments by 1 for each epoch where the validator:
    /// - Is in the active validator set
    /// - Successfully participates in consensus
    /// - Maintains minimum performance requirements
    /// 
    /// The counter does not increment during epochs where the validator:
    /// - Is inactive due to low scores
    /// - Is in a cooldown period
    /// - Is temporarily ejected
    /// 
    /// This provides a measure of validator commitment and network contribution
    /// over time, independent of short-term performance fluctuations.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u32 - Total number of epochs the validator has been active
    #[pallet::storage]
    #[pallet::getter(fn validator_uptime)]
    pub type ValidatorUptime<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Total count of successful inference operations performed by each validator.
    /// 
    /// Tracks the cumulative number of successful AI/ML inference operations
    /// completed by validators as part of the Proof-of-Inference (PoI) consensus.
    /// This metric is fundamental to the CBC blockchain's AI-focused consensus mechanism.
    /// 
    /// The counter increments when validators:
    /// - Successfully complete inference tasks
    /// - Submit results that meet accuracy thresholds
    /// - Participate in distributed AI computations
    /// - Contribute to the network's AI capabilities
    /// 
    /// Used for:
    /// - PoI score calculations and validator ranking
    /// - AI contribution assessment and rewards
    /// - Network AI capacity monitoring
    /// - Validator specialization tracking
    /// 
    /// Higher inference counts indicate validators who are actively contributing
    /// to the network's AI/ML capabilities and should receive higher PoI scores
    /// in the consensus algorithm.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u32 - Total number of successful inference operations completed
    #[pallet::storage]
    #[pallet::getter(fn validator_inference_count)]
    pub type ValidatorInferenceCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Extended metadata and contact information for validators.
    /// 
    /// Stores comprehensive information about validators beyond their basic
    /// operational metrics. This metadata helps with:
    /// - Validator identification and branding
    /// - Community engagement and transparency
    /// - Technical support and communication
    /// - Network governance and coordination
    /// 
    /// The ValidatorMetadataInfo struct typically includes:
    /// - Website URL for the validator's homepage
    /// - Contact information (email, social media)
    /// - Description of the validator's services
    /// - Geographic location information
    /// - Technical specifications and capabilities
    /// - Commission rates and fee structures
    /// 
    /// All metadata is optional and self-reported by validators through
    /// the `set_validator_metadata` extrinsic. The information is not
    /// validated by the protocol but serves as a public registry.
    /// 
    /// Used by block explorers, staking interfaces, and community tools
    /// to provide richer validator information to users and delegators.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: ValidatorMetadataInfo - Comprehensive metadata structure
    #[pallet::storage]
    #[pallet::getter(fn validator_metadata)]
    pub type ValidatorMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorMetadataInfo,
        OptionQuery,
    >;

    /// Historical performance records for each validator over time.
    /// 
    /// Maintains a time-series of performance metrics for each validator,
    /// stored as a bounded vector of PerformanceRecord entries. This data
    /// enables trend analysis and long-term validator evaluation.
    /// 
    /// Each PerformanceRecord typically contains:
    /// - Timestamp or epoch number
    /// - Performance scores (PoS, PoI, combined)
    /// - Block production statistics
    /// - Participation rates and uptime
    /// - Inference accuracy metrics
    /// 
    /// The history is maintained as a circular buffer with a maximum of 100
    /// entries per validator. When the limit is reached, the oldest records
    /// are automatically removed to make space for new ones.
    /// 
    /// Used for:
    /// - Performance trend analysis and prediction
    /// - Validator reliability assessment
    /// - Reward calculation based on historical performance
    /// - Network health monitoring and reporting
    /// - API endpoints for performance charts and graphs
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: BoundedVec<PerformanceRecord, ConstU32<100>> - Circular buffer of performance records
    #[pallet::storage]
    #[pallet::getter(fn validator_performance_history)]
    pub type ValidatorPerformanceHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<PerformanceRecord, ConstU32<100>>,
        ValueQuery,
    >;

    /// Block number when each validator was last observed to be active.
    /// 
    /// Tracks the most recent block number where each validator demonstrated
    /// activity in the network. This is used for:
    /// - Detecting inactive or offline validators
    /// - Calculating activity-based score decay
    /// - Determining when to apply penalties for inactivity
    /// - Network health monitoring and alerting
    /// 
    /// A validator is considered "seen" when they:
    /// - Successfully author a block
    /// - Submit valid transactions or extrinsics
    /// - Participate in consensus voting
    /// - Complete inference operations
    /// - Respond to network challenges
    /// 
    /// The block number is updated automatically by the pallet's hooks
    /// and monitoring systems. Validators with stale "last seen" values
    /// may be subject to:
    /// - Score decay penalties
    /// - Temporary removal from active set
    /// - Reduced reward eligibility
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u32 - Block number of last observed activity
    #[pallet::storage]
    #[pallet::getter(fn validator_last_seen)]
    pub type ValidatorLastSeen<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Block number
        ValueQuery,
    >;

    /// Cumulative count of blocks successfully authored by each validator.
    /// 
    /// Tracks the total number of blocks each validator has successfully
    /// produced and added to the blockchain since joining the network.
    /// This is a key metric for validator performance evaluation.
    /// 
    /// The counter increments when a validator:
    /// - Is selected as the block author for a slot
    /// - Successfully produces a valid block
    /// - Has their block accepted by the network
    /// - Contributes to chain progression
    /// 
    /// Used for:
    /// - Block production performance assessment
    /// - Validator reliability scoring
    /// - Reward calculations based on contribution
    /// - Network statistics and monitoring
    /// - Participation rate calculations (authored vs. assigned)
    /// 
    /// Higher block counts indicate validators who are consistently
    /// available and capable of producing blocks when selected,
    /// contributing to network stability and throughput.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u32 - Total number of blocks successfully authored
    #[pallet::storage]
    #[pallet::getter(fn validator_blocks_authored)]
    pub type ValidatorBlocksAuthored<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Cumulative count of blocks missed by each validator when they were selected.
    /// 
    /// Tracks the total number of block production opportunities that each
    /// validator failed to fulfill since joining the network. This is a
    /// critical metric for identifying unreliable validators.
    /// 
    /// A block is considered "missed" when:
    /// - The validator was selected as the block author
    /// - They failed to produce a block within the allocated time
    /// - Their produced block was invalid or rejected
    /// - They were offline or unresponsive during their slot
    /// 
    /// Used for:
    /// - Validator reliability assessment and penalties
    /// - Performance scoring and ranking
    /// - Automatic ejection decisions for poor performers
    /// - Network health monitoring and alerting
    /// - Participation rate calculations (missed vs. assigned)
    /// 
    /// High miss counts may result in:
    /// - Score penalties and reduced rewards
    /// - Temporary removal from active validator set
    /// - Permanent ejection for chronic poor performance
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u32 - Total number of blocks missed when selected as author
    #[pallet::storage]
    #[pallet::getter(fn validator_blocks_missed)]
    pub type ValidatorBlocksMissed<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Timestamp recording when each validator joined the network.
    /// 
    /// Stores the exact time (in milliseconds since Unix epoch) when each
    /// validator successfully joined the validator set. This information
    /// is used for various temporal calculations and historical analysis.
    /// 
    /// The timestamp is set when a validator:
    /// - Successfully completes the `join_validator_set` process
    /// - Meets all minimum requirements (stake, performance, etc.)
    /// - Is officially added to the validator registry
    /// 
    /// Used for:
    /// - Calculating validator tenure and experience
    /// - Age-based reward calculations or bonuses
    /// - Network growth analysis and statistics
    /// - Validator lifecycle tracking
    /// - API responses for validator information
    /// 
    /// The value is None for validators who joined before this tracking
    /// was implemented, and Some(timestamp) for all validators who joined
    /// after the feature was activated.
    /// 
    /// # Key: T::AccountId - Validator account
    /// # Value: u64 - Join timestamp in milliseconds since Unix epoch (optional)
    #[pallet::storage]
    #[pallet::getter(fn validator_join_time)]
    pub type ValidatorJoinTime<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u64, // Timestamp in milliseconds
        OptionQuery,
    >;

    /// Pending leave requests from validators with the block number when requested.
    /// 
    /// Tracks validators who have requested to leave the validator set but are
    /// still in a cooldown period. The stored block number indicates when the
    /// leave request was submitted, which is used to calculate when the cooldown
    /// period expires and the validator can actually leave.
    /// 
    /// The leave process works as follows:
    /// 1. Validator calls `leave_validators` or `leave_validator_set`
    /// 2. Request is recorded with current block number
    /// 3. Validator enters cooldown period (T::LeaveCooldown blocks)
    /// 4. After cooldown, validator is automatically removed and stake unreserved
    /// 5. Entry is removed from this storage
    /// 
    /// During the cooldown period:
    /// - Validator remains active and must continue participating
    /// - Stake remains reserved and locked
    /// - Request can be cancelled via `cancel_leave_request`
    /// - Validator cannot submit new leave requests
    /// 
    /// Used for:
    /// - Enforcing cooldown periods to prevent rapid validator set changes
    /// - Calculating when leave requests can be executed
    /// - Preventing abuse of the leave mechanism
    /// 
    /// # Key: T::AccountId - Validator account requesting to leave
    /// # Value: u32 - Block number when the leave request was submitted
    #[pallet::storage]
    #[pallet::getter(fn validator_leave_requests)]
    pub type ValidatorLeaveRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Block number when leave request was made
        OptionQuery,
    >;

    /// Block number of the most recently finalized block in the network.
    /// 
    /// Tracks the highest block number that has achieved finality through
    /// the consensus mechanism. Finalization typically occurs at epoch
    /// boundaries or through explicit finality gadgets.
    /// 
    /// Finalized blocks are considered:
    /// - Permanently part of the canonical chain
    /// - Safe from reorganization or rollback
    /// - Confirmed by sufficient validator consensus
    /// - Available for state pruning and archival
    /// 
    /// Used for:
    /// - Determining safe block heights for critical operations
    /// - State pruning and storage optimization
    /// - API responses about network finality status
    /// - Calculating finality lag and network health metrics
    /// - Ensuring transaction irreversibility guarantees
    /// 
    /// The value is updated through:
    /// - Automatic finalization at epoch boundaries
    /// - Explicit finality signals from consensus mechanisms
    /// - Manual finalization through governance actions
    /// 
    /// # Value: u32 - Block number of the last finalized block
    #[pallet::storage]
    #[pallet::getter(fn last_finalized_block)]
    pub type LastFinalizedBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Evidence storage for validator misbehavior reports.
    /// 
    /// Stores cryptographic evidence and proof of validator misbehavior
    /// submitted by other network participants. This system enables
    /// decentralized monitoring and accountability for validator actions.
    /// 
    /// The double map structure allows multiple reporters to submit
    /// evidence against the same validator, building a comprehensive
    /// case for potential slashing or ejection actions.
    /// 
    /// Evidence can include:
    /// - Cryptographic proofs of equivocation (double-signing)
    /// - Invalid block production attempts
    /// - Consensus rule violations
    /// - Off-chain misbehavior with on-chain impact
    /// - Inference result manipulation or fraud
    /// 
    /// The evidence is bounded by `T::MaxEvidenceLength` to prevent
    /// storage abuse while allowing sufficient space for cryptographic
    /// proofs and detailed documentation.
    /// 
    /// Used for:
    /// - Building cases for validator slashing
    /// - Automatic ejection of malicious validators
    /// - Network security and integrity maintenance
    /// - Governance decision support for disciplinary actions
    /// 
    /// # Key1: T::AccountId - Validator being reported for misbehavior
    /// # Key2: T::AccountId - Account submitting the evidence report
    /// # Value: BoundedVec<u8, T::MaxEvidenceLength> - Cryptographic evidence and proof data
    #[pallet::storage]
    #[pallet::getter(fn misbehavior_reports)]
    pub type MisbehaviorReports<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // reported validator
        Blake2_128Concat,
        T::AccountId, // reporter
        BoundedVec<u8, T::MaxEvidenceLength>,
        OptionQuery,
    >;

    /// Reserved balance amounts staked by each validator for network participation.
    ///
    /// Tracks the amount of tokens each validator has locked/reserved as stake
    /// to participate in the consensus mechanism. This stake serves multiple purposes:
    /// - Economic security through slashing risk
    /// - Proof-of-Stake scoring and validator ranking
    /// - Minimum participation requirements enforcement
    /// - Incentive alignment with network health
    ///
    /// Stake amounts are:
    /// - Reserved from the validator's free balance when joining
    /// - Used in PoS score calculations for consensus weight
    /// - Subject to slashing for misbehavior or poor performance
    /// - Unreserved when the validator leaves (after cooldown)
    /// - Adjustable through `increase_validator_stake` and `decrease_validator_stake`
    ///
    /// Minimum stake requirements are enforced through `T::MinStake` configuration.
    /// Validators with insufficient stake are automatically removed from the active set.
    ///
    /// The reserved balance cannot be spent or transferred while staked,
    /// ensuring validators have "skin in the game" for network security.
    ///
    /// # Key: T::AccountId - Validator account
    /// # Value: T::Balance - Amount of tokens reserved as stake
    #[pallet::storage]
    #[pallet::getter(fn validator_stake)]
    pub type ValidatorStake<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        <T as pallet::Config>::Balance,
        ValueQuery,
    >;

    /// Tracks validators who recently left and are in cooldown period to prevent immediate re-entry.
    ///
    /// This storage prevents validators from gaming the system by leaving and immediately
    /// rejoining to reset their performance metrics or avoid penalties. It enforces a
    /// mandatory cooldown period during which validators cannot rejoin the network.
    ///
    /// The cooldown mechanism works as follows:
    /// 1. When a validator completes the leave process (after LeaveCooldown expires)
    /// 2. Their account is added to this storage with the block number when they left
    /// 3. During validator selection and proposal generation, accounts in this storage are excluded
    /// 4. After an additional cooldown period (T::LeaveCooldown), the entry is automatically removed
    /// 5. Only then can the validator attempt to rejoin the network
    ///
    /// This prevents:
    /// - Rapid validator set changes that could destabilize consensus
    /// - Gaming of performance metrics by leaving and rejoining
    /// - Circumvention of penalties through validator cycling
    /// - Sybil attacks through rapid validator rotation
    ///
    /// The storage is automatically cleaned up during on_initialize() processing
    /// to remove expired cooldown entries and maintain storage efficiency.
    ///
    /// # Key: T::AccountId - Validator account that recently left
    /// # Value: u32 - Block number when the validator completed the leave process
    #[pallet::storage]
    #[pallet::getter(fn recently_removed_validators)]
    pub type RecentlyRemovedValidators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Block number when validator left
        OptionQuery,
    >;



    // --- Events --- //
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorScoreUpdated {
            validator: T::AccountId,
            stake_score: u64,
            inference_score: u64,
            final_score: u64,
        },
        ConsensusWeightsUpdated {
            pos_weight: u64,
            poi_weight: u64,
        },
        EpochStarted {
            epoch: u32,
            validators: Vec<T::AccountId>,
        },
        EpochEnded {
            epoch: u32,
        },
        ValidatorScoreDecayed {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
        },
        ValidatorScoreBoosted {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
            reason: ScoreBoostReason,
        },
        ValidatorEjected {
            validator: T::AccountId,
            reason: EjectionReason,
        },
        ValidatorReEntered {
            validator: T::AccountId,
            score: u64,
        },

        /// Block author mismatch detected - expected author differs from actual author
        AuthorMismatch {
            block_number: u32,
            expected_author: T::AccountId,
            actual_author: T::AccountId,
        },

        /// Validator attempted to join but has insufficient stake
        InsufficientStake {
            validator: T::AccountId,
            required: <T as pallet::Config>::Balance,
            available: <T as pallet::Config>::Balance,
        },




        InvalidAuthor {
            block_number: u32,
            author: T::AccountId,
        },
        GovernanceModeToggled {
            enabled: bool,
        },
        ProposalSubmitted {
            proposal_id: u32,
            proposer: T::AccountId,
            action: ProposalAction<T>,
        },
        ProposalExecuted {
            proposal_id: u32,
            status: ProposalStatus,
        },
        ProposalVoted {
            proposal_id: u32,
            voter: T::AccountId,
            approve: bool,
        },
        ProposalPassed { proposal_id: u32 },
        ProposalRejected { proposal_id: u32 },
        ValidatorJoined { validator: T::AccountId },
        ValidatorLeft { validator: T::AccountId },
        ValidatorLeaveRequested { 
            validator: T::AccountId,
            cooldown_expires_at: u32,
        },
        ValidatorLeaveCancelled {
            validator: T::AccountId,
        },

        ValidatorPoiScoreUpdated {
            validator: T::AccountId,
            poi_score: u64,
        },
        ValidatorMetadataUpdated {
            validator: T::AccountId,
            name: BoundedVec<u8, ConstU32<32>>,
        },
        ValidatorActivityUpdated {
            validator: T::AccountId,
            blocks_authored: u32,
            blocks_missed: u32,
            uptime_percentage: u32,
        },
        AutomaticValidatorProposal {
            validator: T::AccountId,
            action: ValidatorAction,
            score: u64,
            reason: BoundedVec<u8, ConstU32<64>>,
        },
        ValidatorAdded {
            validator: T::AccountId,
        },
        ValidatorRemoved {
            validator: T::AccountId,
        },
        ValidatorRewarded {
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        },
        ValidatorSlashed {
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        },
        ValidatorMisbehaviorReported {
            reported_validator: T::AccountId,
            reporter: T::AccountId,
            evidence: BoundedVec<u8, T::MaxEvidenceLength>,
            total_reports: u32,
        },
        ValidatorAutoSlashed {
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
            report_count: u32,
        },
        InferenceSimulated {
            validator: T::AccountId,
            triggered_by: T::AccountId,
        },
        BlockFinalized {
            block_number: u32,
        },
        EpochTransitioned {
            old_epoch: u32,
            new_epoch: u32,
            active_validators: Vec<T::AccountId>,
            total_validators: u32,
        },
        EpochBoundaryDetected {
            block_number: u32,
            epoch: u32,
            governance_mode: bool,
        },
        ValidatorNameRegistered {
            validator: T::AccountId,
            name: BoundedVec<u8, ConstU32<32>>,
        },
        ProposalCreated {
            proposal_id: u32,
            proposer: T::AccountId,
            action: ProposalAction<T>,
            description: BoundedVec<u8, ConstU32<128>>,
        },
        ValidatorScoreUpdatedDetailed {
            validator: T::AccountId,
            old_stake_score: u64,
            new_stake_score: u64,
            old_inference_score: u64,
            new_inference_score: u64,
            old_final_score: u64,
            new_final_score: u64,
            epoch: u32,
        },
        /// Stake was reserved when validator joined
        ValidatorStakeReserved {
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        },
        /// Stake was unreserved when validator left/was removed
        ValidatorStakeUnreserved {
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        },
        /// Epoch rewards were distributed using modular logic
        EpochRewardsDistributed {
            total_pool: <T as pallet::Config>::Balance,
            base_pool: <T as pallet::Config>::Balance,
            performance_pool: <T as pallet::Config>::Balance,
            top_performer_pool: <T as pallet::Config>::Balance,
        },

        // Additional high-priority events for complete coverage

        /// Emitted when validator stake is increased
        ValidatorStakeIncreased {
            validator: T::AccountId,
            old_amount: <T as pallet::Config>::Balance,
            new_amount: <T as pallet::Config>::Balance,
            increase: <T as pallet::Config>::Balance,
        },

        /// Emitted when validator stake is decreased
        ValidatorStakeDecreased {
            validator: T::AccountId,
            old_amount: <T as pallet::Config>::Balance,
            new_amount: <T as pallet::Config>::Balance,
            decrease: <T as pallet::Config>::Balance,
        },

        /// Emitted when trust score is updated
        TrustScoreUpdated {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
            uptime_component: u64,
            inference_component: u64,
            slashing_component: u64,
        },

        /// Emitted when cooldown period expires
        CooldownExpired {
            validator: T::AccountId,
            cooldown_type: BoundedVec<u8, ConstU32<32>>, // "Leave" or "RecentlyRemoved"
            expired_at_block: u32,
        },

        /// Emitted when stake operation fails
        StakeOperationFailed {
            validator: T::AccountId,
            operation: BoundedVec<u8, ConstU32<32>>, // "Increase", "Decrease", "Reserve", "Unreserve"
            reason: BoundedVec<u8, ConstU32<64>>,
        },

        /// Emitted when trust score decays over time
        TrustScoreDecayed {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
            decay_factor: u64,
        },
    }

    // --- Errors --- //
    #[pallet::error]
    pub enum Error<T> {
        ValidatorNotFound,
        InvalidWeight,
        InvalidEpochConfig,
        NotEnoughValidators,
        NotAllowedInGovernanceMode,
        NotValidator,
        AlreadyVoted,
        ProposalNotApproved,
        ProposalAlreadyExecuted,
        InvalidScore,
        ValidatorAlreadyExists,
        ValidatorNotInSet,
        LeaveCooldownActive,
        /// Validator is in cooldown period after leaving and cannot rejoin yet
        ValidatorInCooldown,
        /// Block author validation failed - author mismatch detected
        AuthorValidationFailed,
        /// Validator does not have sufficient stake to join
        InsufficientStake,
    }

    // --- Dispatchable Calls --- //
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update a validator's stake score (PoS).
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::update_validator_stake_score())]
        pub fn update_validator_stake_score(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            if GovernanceModeEnabled::<T>::get() {
                ensure_root(origin)?;
            } else {
                ensure_signed(origin)?;
            }
            let stake = pos::Pallet::<T>::stake(&validator);
            let stake_score = stake.saturated_into::<u64>();
            ValidatorStates::<T>::try_mutate(&validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.stake_score = stake_score;
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;
            Self::update_final_score(&validator)?;
            Ok(())
        }

        /// Update a validator's inference score (PoI).
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_validator_inference_score())]
        pub fn update_validator_inference_score(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            if GovernanceModeEnabled::<T>::get() {
                ensure_root(origin)?;
            } else {
                ensure_signed(origin)?;
            }
            if let Some((result, _)) = poi::Pallet::<T>::inference_results(&validator) {
                let inference_score = result as u64;
                ValidatorStates::<T>::try_mutate(&validator, |maybe_state| {
                    let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                    state.current.inference_score = inference_score;
                    Ok::<(), Error<T>>(())
                }).map_err(|e| sp_runtime::DispatchError::from(e))?;
                Self::update_final_score(&validator)?;
            }
            Ok(())
        }

        /// Apply PoI scores computed by off-chain worker (signed transaction).
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn apply_offchain_poi_scores(
            origin: OriginFor<T>,
            block_number: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            let validators = Self::validator_set();
            let mut _updated_count = 0u32;
            
            for validator in validators.iter() {
                // Try to retrieve computed PoI score from off-chain storage
                if let Ok(Some(poi_score)) = Self::get_offchain_poi_score(validator, block_number) {
                    // Validate the score is within acceptable range
                    if poi_score <= T::MaxValidatorScore::get() {
                        // Update the validator's PoI score
                        if let Ok(()) = ValidatorStates::<T>::try_mutate(&validator, |maybe_state| {
                            let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                            state.current.inference_score = poi_score;
                            Ok::<(), Error<T>>(())
                        }).map_err(|e| sp_runtime::DispatchError::from(e)) {
                            // Recalculate final score
                            let _ = Self::update_final_score(&validator);
                            _updated_count += 1;
                            
                            // Emit event
                            Self::deposit_event(Event::ValidatorPoiScoreUpdated {
                                validator: validator.clone(),
                                poi_score,
                            });
                        }
                    }
                }
            }
            

            Ok(())
        }

        /// Update consensus weights for PoS and PoI.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::update_consensus_weights())]
        pub fn update_consensus_weights(
            origin: OriginFor<T>,
            pos_weight: u64,
            poi_weight: u64,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(pos_weight + poi_weight == T::PercentagePrecision::get() as u64, Error::<T>::InvalidWeight);
            PosWeight::<T>::put(pos_weight);
            PoiWeight::<T>::put(poi_weight);
            Self::deposit_event(Event::ConsensusWeightsUpdated {
                pos_weight,
                poi_weight,
            });
            Ok(())
        }

        /// Toggle governance mode (sudo-like).
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::set_governance_mode())]
        pub fn set_governance_mode(origin: OriginFor<T>, enabled: bool) -> DispatchResult {
            ensure_root(origin)?;
            GovernanceModeEnabled::<T>::put(enabled);
            Self::deposit_event(Event::GovernanceModeToggled { enabled });
            Ok(())
        }

        /// Sudo: advance epoch manually (governance mode only).
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::sudo_advance_epoch())]
        pub fn sudo_advance_epoch(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(GovernanceModeEnabled::<T>::get(), Error::<T>::NotAllowedInGovernanceMode);
            let _ = Self::handle_epoch_transition();
            Ok(())
        }

        /// Submit a governance proposal (slash, reward, eject).
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_proposal())]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            action: ProposalAction<T>,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer: proposer.clone(),
                action: action.clone(),
            });
            
            // Emit detailed proposal created event
            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer,
                action,
                description: description.unwrap_or_else(|| BoundedVec::truncate_from(b"No description provided".to_vec())),
            });
            
            Ok(())
        }

        /// Vote on a governance proposal.
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_proposal())]
        pub fn vote_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotApproved)?;
                ensure!(matches!(prop.status, ProposalStatus::Pending), Error::<T>::ProposalAlreadyExecuted);
                ensure!(!ProposalVotes::<T>::contains_key(proposal_id, &who), Error::<T>::AlreadyVoted);

                if approve {
                    prop.votes_for += 1;
                } else {
                    prop.votes_against += 1;
                }
                ProposalVotes::<T>::insert(proposal_id, &who, approve);
                Self::deposit_event(Event::ProposalVoted {
                    proposal_id,
                    voter: who,
                    approve,
                });

                // --- Quorum logic: require at least half of active validators to vote ---
                let quorum = (ActiveValidators::<T>::get().len() as u32 + 1) / 2;
                let total_votes = prop.votes_for + prop.votes_against;
                if total_votes >= quorum {
                    if prop.votes_for > prop.votes_against {
                        prop.status = ProposalStatus::Approved;
                        Self::deposit_event(Event::ProposalPassed { proposal_id });
                    } else {
                        prop.status = ProposalStatus::Rejected;
                        Self::deposit_event(Event::ProposalRejected { proposal_id });
                    }
                }
                Ok::<(), Error<T>>(())
            })?;
            Ok(())
        }

        /// Execute an approved governance proposal (sudo only).
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_proposal())]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotApproved)?;
                ensure!(matches!(prop.status, ProposalStatus::Approved), Error::<T>::ProposalNotApproved);

                // Execute action with proper implementation
                match &prop.action {
                    ProposalAction::Slash { validator, amount } => {
                        // Implement actual slashing logic
                        let _ = Self::execute_slash_validator(validator, *amount);
                    }
                    ProposalAction::Reward { validator, amount } => {
                        // Implement actual reward logic
                        let _ = Self::execute_reward_validator(validator, *amount);
                    }
                    ProposalAction::RewardMultiple { validators, amount } => {
                        // Reward multiple validators with the same amount
                        for validator in validators.iter() {
                            let _ = Self::execute_reward_validator(validator, *amount);
                        }
                        log::info!("Rewarded {} validators with amount {:?} each", validators.len(), amount);
                    }
                    ProposalAction::Eject { validator, reason } => {
                        // Implement actual ejection logic
                        let _ = Self::execute_eject_validator(validator, reason.clone());
                    }
                    ProposalAction::AddValidator { validator } => {
                        // Add validator to the validator set
                        let _ = Self::execute_add_validator(validator);
                    }
                    ProposalAction::RemoveValidator { validator } => {
                        // Remove validator from the validator set
                        let _ = Self::execute_remove_validator(validator);
                    }
                }
                prop.status = ProposalStatus::Executed;
                Self::deposit_event(Event::ProposalExecuted {
                    proposal_id,
                    status: prop.status.clone(),
                });
                Ok::<(), Error<T>>(())
            })?;
            Ok(())
        }

        /// Sudo propose to slash a validator.
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_slash_validator(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let action = ProposalAction::Slash { validator, amount };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            Ok(())
        }

        /// Sudo propose to reward a validator.
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_reward_validator(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validator: T::AccountId,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let action = ProposalAction::Reward { validator, amount };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            Ok(())
        }

        /// Propose to reward a validator with the default reward amount
        #[pallet::call_index(26)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_default_reward_validator(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validator: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let default_reward = T::ValidatorReward::get();
            let action = ProposalAction::Reward { validator: validator.clone(), amount: default_reward };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            log::info!("Default reward proposal created for validator {:?} with amount {:?}", validator, default_reward);
            Ok(())
        }

        /// Propose to reward multiple validators with the same amount
        #[pallet::call_index(27)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_reward_multiple_validators(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validators: Vec<T::AccountId>,
            amount: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            // Validate validators list length
            ensure!(
                validators.len() <= <T as Config>::MaxValidators::get() as usize,
                Error::<T>::NotEnoughValidators
            );
            ensure!(!validators.is_empty(), Error::<T>::NotEnoughValidators);
            
            // Validate that all validators exist
            for validator in validators.iter() {
                ensure!(
                    ValidatorStates::<T>::contains_key(validator),
                    Error::<T>::ValidatorNotFound
                );
            }
            
            let bounded_validators = BoundedVec::try_from(validators.clone())
                .map_err(|_| Error::<T>::NotEnoughValidators)?;
            let action = ProposalAction::RewardMultiple { validators: bounded_validators, amount };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            log::info!("Multiple validator reward proposal created for {} validators with amount {:?} each", 
                      validators.len(), amount);
            Ok(())
        }

        /// Propose to reward multiple validators with the default reward amount
        #[pallet::call_index(28)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_default_reward_multiple_validators(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validators: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            // Validate validators list length
            ensure!(
                validators.len() <= <T as Config>::MaxValidators::get() as usize,
                Error::<T>::NotEnoughValidators
            );
            ensure!(!validators.is_empty(), Error::<T>::NotEnoughValidators);
            
            // Validate that all validators exist
            for validator in validators.iter() {
                ensure!(
                    ValidatorStates::<T>::contains_key(validator),
                    Error::<T>::ValidatorNotFound
                );
            }
            
            let default_reward = T::ValidatorReward::get();
            let bounded_validators = BoundedVec::try_from(validators.clone())
                .map_err(|_| Error::<T>::NotEnoughValidators)?;
            let action = ProposalAction::RewardMultiple { validators: bounded_validators, amount: default_reward };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            log::info!("Default multiple validator reward proposal created for {} validators with amount {:?} each", 
                      validators.len(), default_reward);
            Ok(())
        }

        /// Propose to reward all active validators with the default reward amount
        #[pallet::call_index(29)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_reward_all_active_validators(
            origin: OriginFor<T>,
            proposer: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let active_validators = ActiveValidators::<T>::get();
            ensure!(!active_validators.is_empty(), Error::<T>::NotEnoughValidators);
            
            let default_reward = T::ValidatorReward::get();
            let bounded_validators = BoundedVec::try_from(active_validators.clone().into_inner())
                .map_err(|_| Error::<T>::NotEnoughValidators)?;
            let action = ProposalAction::RewardMultiple { 
                validators: bounded_validators, 
                amount: default_reward 
            };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            log::info!("Reward proposal created for all {} active validators with amount {:?} each", 
                      active_validators.len(), default_reward);
            Ok(())
        }

        /// Slash a validator's reserved stake directly (Root only)
        /// This function slashes from the validator's reserved stake, not free balance
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn slash_validator(
            origin: OriginFor<T>,
            target: T::AccountId,
            amount: <T as Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Check if validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(&target),
                Error::<T>::ValidatorNotFound
            );

            // Check if validator has reserved stake
            let reserved_balance = T::Currency::reserved_balance(&target);
            ensure!(
                reserved_balance > <T as Config>::Balance::default(),
                Error::<T>::InsufficientStake
            );

            // Calculate actual slash amount (cannot exceed reserved balance)
            let actual_slash_amount = amount.min(reserved_balance);

            // Slash from reserved balance
            let (negative_imbalance, remaining_slash) = T::Currency::slash_reserved(&target, actual_slash_amount);
            let slashed_amount = actual_slash_amount.saturating_sub(remaining_slash);

            // Update the ValidatorStake storage to reflect the reduced stake
            ValidatorStake::<T>::mutate(&target, |current_stake| {
                *current_stake = current_stake.saturating_sub(slashed_amount);
            });

            // Option 1: Burn the slashed funds (remove from total supply)
            // T::Currency::burn(negative_imbalance);

            // Option 2: Transfer to treasury (if treasury pallet is available)
            // For now, we'll burn the funds as it's simpler and doesn't require treasury integration
            drop(negative_imbalance); // This effectively burns the slashed amount

            // Reduce validator's DCF score based on slash amount
            let score_penalty = (slashed_amount.saturated_into::<u64>() / T::SlashPenaltyDivisor::get())
                .min(T::MaxSlashPenalty::get());
            
            ValidatorStates::<T>::try_mutate(&target, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let old_score = state.current.final_score;
                state.current.final_score = state.current.final_score.saturating_sub(score_penalty);
                
                // Update stake score to reflect reduced stake
                let new_stake = ValidatorStake::<T>::get(&target);
                state.current.stake_score = new_stake.saturated_into::<u64>();
                
                // Update last active epoch
                state.last_active_epoch = Self::current_epoch();
                
                // Add to history
                if state.history.len() == state.history.capacity() {
                    state.history.remove(0);
                }
                let _ = state.history.try_push(EpochStats {
                    epoch: Self::current_epoch(),
                    stake_score: state.current.stake_score,
                    inference_score: state.current.inference_score,
                    final_score: state.current.final_score,
                    authored_blocks: state.current.authored_blocks,
                    missed_blocks: state.current.missed_blocks,
                });
                
                log::info!("Validator {:?} slashed from reserved stake: amount {:?}, score {} -> {}, penalty: {}", 
                          target, slashed_amount, old_score, state.current.final_score, score_penalty);
                
                Ok::<(), Error<T>>(())
            })?;

            // Check if validator should be ejected due to low score
            let current_score = ValidatorStates::<T>::get(&target)
                .map(|s| s.current.final_score)
                .unwrap_or(0);
                
            if current_score < <T as Config>::MinValidatorScore::get() as u64 {
                let _ = Self::eject_validator(&target, EjectionReason::ScoreBelowThreshold);
                log::info!("Validator {:?} ejected due to low score after slashing", target);
            }

            // Check if remaining stake is below minimum requirement
            let remaining_stake = ValidatorStake::<T>::get(&target);
            if remaining_stake < <T as Config>::MinStake::get() {
                log::warn!("Validator {:?} stake below minimum after slashing. Consider ejection.", target);
                // Optionally auto-eject validator if stake is too low
                let _ = Self::eject_validator(&target, EjectionReason::InsufficientStake);
            }

            // Emit slashing event
            Self::deposit_event(Event::ValidatorSlashed {
                validator: target.clone(),
                amount: slashed_amount,
            });

            // Emit score update event
            Self::deposit_event(Event::ValidatorScoreUpdated {
                validator: target.clone(),
                stake_score: ValidatorStates::<T>::get(&target).map(|s| s.current.stake_score).unwrap_or(0),
                inference_score: ValidatorStates::<T>::get(&target).map(|s| s.current.inference_score).unwrap_or(0),
                final_score: current_score,
            });

            log::info!("Successfully slashed validator {:?} for amount {:?} from reserved stake", target, slashed_amount);
            Ok(())
        }

        /// Slash a validator's reserved stake by percentage (Root only)
        /// This function slashes a percentage of the validator's reserved stake
        #[pallet::call_index(31)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn slash_validator_percentage(
            origin: OriginFor<T>,
            target: T::AccountId,
            percentage: u32, // Percentage (0-100)
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate percentage
            ensure!(percentage <= T::FullPercentage::get(), Error::<T>::InvalidScore);

            // Check if validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(&target),
                Error::<T>::ValidatorNotFound
            );

            // Calculate slash amount based on percentage of reserved balance
            let reserved_balance = T::Currency::reserved_balance(&target);
            ensure!(
                reserved_balance > <T as Config>::Balance::default(),
                Error::<T>::InsufficientStake
            );

            let slash_amount = reserved_balance * <T as Config>::Balance::from(percentage) / <T as Config>::Balance::from(100u32);

            // Call the main slash_validator function
            Self::slash_validator(
                frame_system::RawOrigin::Root.into(),
                target.clone(),
                slash_amount,
            )?;

            log::info!("Slashed validator {:?} by {}% of reserved stake (amount: {:?})", target, percentage, slash_amount);
            Ok(())
        }

        /// Slash multiple validators' reserved stakes (Root only)
        /// This function slashes the same amount from multiple validators
        #[pallet::call_index(32)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn slash_multiple_validators(
            origin: OriginFor<T>,
            targets: Vec<T::AccountId>,
            amount: <T as Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate targets list
            ensure!(!targets.is_empty(), Error::<T>::NotEnoughValidators);
            ensure!(
                targets.len() <= <T as Config>::MaxValidators::get() as usize,
                Error::<T>::NotEnoughValidators
            );

            let mut slashed_count = 0u32;
            let mut total_slashed = <T as Config>::Balance::default();

            // Slash each validator
            for target in targets.iter() {
                match Self::slash_validator(
                    frame_system::RawOrigin::Root.into(),
                    target.clone(),
                    amount,
                ) {
                    Ok(()) => {
                        slashed_count += 1;
                        total_slashed = total_slashed.saturating_add(amount);
                    }
                    Err(e) => {
                        log::warn!("Failed to slash validator {:?}: {:?}", target, e);
                        // Continue with other validators instead of failing the entire operation
                    }
                }
            }

            log::info!("Slashed {} out of {} validators for amount {:?} each (total: {:?})", 
                      slashed_count, targets.len(), amount, total_slashed);

            // Emit a summary event (we could add this as a new event type)
            // For now, individual ValidatorSlashed events are emitted by slash_validator

            Ok(())
        }

        /// Sudo propose to eject a validator.
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn propose_eject_validator(
            origin: OriginFor<T>,
            proposer: T::AccountId,
            validator: T::AccountId,
            reason: EjectionReason,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let action = ProposalAction::Eject { validator, reason: reason.clone() };
            let proposal_id = NextProposalId::<T>::get();
            let proposal = GovernanceProposal {
                proposer: proposer.clone(),
                action: action.clone(),
                status: ProposalStatus::Pending,
                votes_for: 0,
                votes_against: 0,
            };
            Proposals::<T>::insert(proposal_id, proposal);
            NextProposalId::<T>::put(proposal_id + 1);
            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id,
                proposer,
                action,
            });
            Ok(())
        }

        /// Request to join the validator set (opt-in, effective next epoch).
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::join_validator_set())]
        pub fn join_validator_set(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Already pending join or already active
            ensure!(
                !Self::active_validators().contains(&who),
                Error::<T>::NotAllowedInGovernanceMode 
            );
            ensure!(
                PendingValidatorActions::<T>::get(&who) != Some(ValidatorAction::Join),
                Error::<T>::NotAllowedInGovernanceMode
            );

            // Check minimum stake and score now, but actual addition is at epoch
            let stake = pos::Pallet::<T>::stake(&who);
            ensure!(
                stake >= <T as pallet_cbc_pos::Config>::MinStake::get(),
                Error::<T>::NotEnoughValidators
            );
            let state = ValidatorStates::<T>::get(&who).ok_or(Error::<T>::ValidatorNotFound)?;
            ensure!(
                state.current.final_score >= <T as pallet::Config>::MinValidatorScore::get() as u64,
                Error::<T>::NotValidator
            );

            PendingValidatorActions::<T>::insert(&who, ValidatorAction::Join);
            Self::deposit_event(Event::ValidatorJoined { validator: who });
            Ok(())
        }

        /// Join the validator set by meeting minimum stake requirements.
        /// This adds the validator to the main ValidatorSet and initializes their state.
        #[pallet::call_index(18)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn join_validators(
            origin: OriginFor<T>,
            name: Option<BoundedVec<u8, ConstU32<32>>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if validator is in cooldown period after recently leaving
            if let Some(left_at_block) = RecentlyRemovedValidators::<T>::get(&who) {
                let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
                let cooldown_period = T::LeaveCooldown::get();
                let blocks_since_left = current_block.saturating_sub(left_at_block);

                ensure!(
                    blocks_since_left >= cooldown_period,
                    Error::<T>::ValidatorInCooldown
                );

                // Cooldown has expired, remove from recently removed list
                RecentlyRemovedValidators::<T>::remove(&who);
            }

            // Check if validator is already in the validator set
            let mut validator_set = ValidatorSet::<T>::get();
            ensure!(
                !validator_set.contains(&who),
                Error::<T>::ValidatorAlreadyExists
            );

            // Check minimum stake requirement using DCF pallet's MinStake
            let min_stake = <T as Config>::MinStake::get();
            let free_balance = T::Currency::free_balance(&who);
            ensure!(
                free_balance >= min_stake,
                Error::<T>::InsufficientStake
            );

            // Reserve the minimum stake to lock it for validator participation
            T::Currency::reserve(&who, min_stake)
                .map_err(|_| Error::<T>::InsufficientStake)?;

            // Store the actual locked stake amount
            ValidatorStake::<T>::insert(&who, min_stake);

            // Emit stake reservation event
            Self::deposit_event(Event::ValidatorStakeReserved {
                validator: who.clone(),
                amount: min_stake,
            });

            // Check that we haven't exceeded the maximum validators limit
            ensure!(
                validator_set.len() < validator_set.capacity(),
                Error::<T>::NotEnoughValidators
            );

            // Add validator to the validator set
            validator_set.try_push(who.clone())
                .map_err(|_| Error::<T>::NotEnoughValidators)?;
            ValidatorSet::<T>::put(validator_set);

            // Initialize validator state if it doesn't exist
            if !ValidatorStates::<T>::contains_key(&who) {
                let current_epoch = Self::current_epoch();
                let stake_score = min_stake.saturated_into::<u64>();
                
                // Get initial inference score from PoI pallet
                let inference_score = poi::Pallet::<T>::inference_results(&who)
                    .map(|(result, _)| result as u64)
                    .unwrap_or(0);

                // Calculate initial final score
                let pos_weight = if !PosWeight::<T>::exists() {
                    let weight = T::DefaultPosWeight::get();
                    PosWeight::<T>::put(weight);
                    weight
                } else {
                    PosWeight::<T>::get()
                };
                let poi_weight = if !PoiWeight::<T>::exists() {
                    let weight = T::DefaultPoiWeight::get();
                    PoiWeight::<T>::put(weight);
                    weight
                } else {
                    PoiWeight::<T>::get()
                };

                let mut final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / T::PercentagePrecision::get() as u64;
                if final_score > T::MaxValidatorScore::get() {
                    final_score = T::MaxValidatorScore::get();
                }

                let initial_stats = EpochStats {
                    epoch: current_epoch,
                    stake_score,
                    inference_score,
                    final_score,
                    authored_blocks: 0,
                    missed_blocks: 0,
                };

                let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
                
                let validator_state = ValidatorState {
                    last_active_epoch: current_epoch,
                    current: initial_stats.clone(),
                    history: BoundedVec::try_from(vec![initial_stats]).unwrap_or_default(),
                    uptime: 1, // Starting with 1 epoch
                    inference_success_count: 0,
                    participation_rate: 0,
                    inference_count: 0, // Starting with 0 inferences
                    last_active_block: current_block, // Current block number
                    name: name.as_ref().map(|n| BoundedVec::truncate_from(n.to_vec())), // Optional validator name
                    trust_score: 0, // Will be calculated later
                };

                ValidatorStates::<T>::insert(&who, validator_state);

                // Set join time
                let current_time = sp_io::offchain::timestamp().unix_millis();
                ValidatorJoinTime::<T>::insert(&who, current_time);
            }

            // Emit event
            Self::deposit_event(Event::ValidatorJoined { validator: who });

            Ok(())
        }

        /// Leave the validator set with cooldown period.
        /// Marks validator for leaving but keeps funds reserved until cooldown expires.
        /// Funds are automatically unreserved by on_initialize() hook after cooldown.
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn leave_validators(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if validator is in the validator set
            let validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&who),
                Error::<T>::ValidatorNotInSet
            );

            // Check if there's already a pending leave request
            ensure!(
                !ValidatorLeaveRequests::<T>::contains_key(&who),
                Error::<T>::LeaveCooldownActive
            );

            let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();

            // Mark validator as leaving - funds remain reserved until cooldown expires
            ValidatorLeaveRequests::<T>::insert(&who, current_block);

            // Remove from active validators immediately to stop them from participating
            let mut active_validators = ActiveValidators::<T>::get();
            if let Some(pos) = active_validators.iter().position(|v| v == &who) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
            }

            // Emit event to indicate leave request has been made
            Self::deposit_event(Event::ValidatorLeaveRequested {
                validator: who.clone(),
                cooldown_expires_at: current_block + T::LeaveCooldown::get(),
            });

            log::info!("Validator {:?} requested to leave. Cooldown expires at block {}", 
                      who, current_block + T::LeaveCooldown::get());

            Ok(())
        }

        /// Cancel a pending leave request (before cooldown expires)
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn cancel_leave_request(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if there's a pending leave request
            ensure!(
                ValidatorLeaveRequests::<T>::contains_key(&who),
                Error::<T>::ValidatorNotFound
            );

            // Remove the leave request
            ValidatorLeaveRequests::<T>::remove(&who);

            // Re-add to active validators if they're still in the validator set
            let validator_set = ValidatorSet::<T>::get();
            if validator_set.contains(&who) {
                let mut active_validators = ActiveValidators::<T>::get();
                if !active_validators.contains(&who) {
                    // Only add if there's space and they're not already active
                    if active_validators.len() < active_validators.capacity() {
                        let _ = active_validators.try_push(who.clone());
                        ActiveValidators::<T>::put(active_validators);
                    }
                }
            }

            // Emit event
            Self::deposit_event(Event::ValidatorLeaveCancelled {
                validator: who.clone(),
            });

            log::info!("Validator {:?} cancelled their leave request", who);

            Ok(())
        }

        /// Request to leave the validator set (opt-out, effective next epoch).
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::leave_validator_set())]
        pub fn leave_validator_set(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Already pending leave or not active
            ensure!(
                Self::active_validators().contains(&who),
                Error::<T>::NotValidator
            );
            ensure!(
                PendingValidatorActions::<T>::get(&who) != Some(ValidatorAction::Leave),
                Error::<T>::NotAllowedInGovernanceMode
            );

            PendingValidatorActions::<T>::insert(&who, ValidatorAction::Leave);
            Self::deposit_event(Event::ValidatorLeft { validator: who });
            Ok(())
        }

        /// Set validator display name
        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::set_validator_name())]
        pub fn set_validator_name(
            origin: OriginFor<T>,
            name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure the validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(&who),
                Error::<T>::ValidatorNotFound
            );
            
            // Validate name length
            let bounded_name = BoundedVec::try_from(name)
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            ValidatorNames::<T>::insert(&who, &bounded_name);
            
            // Emit event for validator name registration
            Self::deposit_event(Event::ValidatorNameRegistered {
                validator: who,
                name: bounded_name,
            });
            
            Ok(())
        }

        /// Set comprehensive validator metadata
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_validator_metadata(
            origin: OriginFor<T>,
            name: Vec<u8>,
            website: Option<Vec<u8>>,
            contact: Option<Vec<u8>>,
            description: Option<Vec<u8>>,
            location: Option<Vec<u8>>,
            commission_rate: Option<u32>,
            min_stake_required: Option<<T as pallet::Config>::Balance>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure the validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(&who),
                Error::<T>::ValidatorNotFound
            );
            
            // Validate and convert inputs
            let bounded_name = BoundedVec::try_from(name)
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            let bounded_website = website.map(|w| BoundedVec::try_from(w))
                .transpose()
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            let bounded_contact = contact.map(|c| BoundedVec::try_from(c))
                .transpose()
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            let bounded_description = description.map(|d| BoundedVec::try_from(d))
                .transpose()
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            let bounded_location = location.map(|l| BoundedVec::try_from(l))
                .transpose()
                .map_err(|_| Error::<T>::InvalidEpochConfig)?;
            
            // Validate commission rate
            if let Some(rate) = commission_rate {
                ensure!(rate <= T::MaxCommissionRate::get(), Error::<T>::InvalidEpochConfig);
            }
            
            let now = Self::get_current_timestamp();
            let metadata = ValidatorMetadataInfo {
                name: bounded_name.clone(),
                website: bounded_website,
                contact: bounded_contact,
                description: bounded_description,
                location: bounded_location,
                commission_rate,
                min_stake_required: min_stake_required.map(|s| s.saturated_into()),
                created_at: ValidatorMetadata::<T>::get(&who)
                    .map(|m| m.created_at)
                    .unwrap_or(now),
                updated_at: now,
            };
            
            ValidatorMetadata::<T>::insert(&who, metadata);
            ValidatorNames::<T>::insert(&who, &bounded_name);
            
            // Emit event for validator name registration
            Self::deposit_event(Event::ValidatorNameRegistered {
                validator: who.clone(),
                name: bounded_name.clone(),
            });
            
            // Set join time if not already set
            if !ValidatorJoinTime::<T>::contains_key(&who) {
                ValidatorJoinTime::<T>::insert(&who, now);
            }
            
            // Emit event
            Self::deposit_event(Event::ValidatorMetadataUpdated {
                validator: who,
                name: bounded_name,
            });
            
            Ok(())
        }

        /// Update validator activity metrics (called internally)
        #[pallet::call_index(17)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn update_validator_activity(
            origin: OriginFor<T>,
            validator: T::AccountId,
            blocks_authored: u32,
            blocks_missed: u32,
        ) -> DispatchResult {
            ensure_root(origin)?; // Only callable by root or internal logic
            
            let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
            
            // Update last seen
            ValidatorLastSeen::<T>::insert(&validator, current_block);
            
            // Update total counters
            ValidatorBlocksAuthored::<T>::mutate(&validator, |total| *total += blocks_authored);
            ValidatorBlocksMissed::<T>::mutate(&validator, |total| *total += blocks_missed);
            
            // Update performance history
            let current_epoch = Self::current_epoch();
            let now = Self::get_current_timestamp();
            
            if let Some(state) = ValidatorStates::<T>::get(&validator) {
                let uptime_percentage = Self::calculate_uptime_percentage(&validator);
                
                let performance_record = PerformanceRecord {
                    epoch: current_epoch,
                    blocks_authored,
                    blocks_missed,
                    uptime_percentage,
                    inference_score: state.current.inference_score,
                    stake_score: state.current.stake_score,
                    final_score: state.current.final_score,
                    participation_rate: state.participation_rate,
                    timestamp: now,
                };
                
                ValidatorPerformanceHistory::<T>::mutate(&validator, |history| {
                    if history.len() >= T::MaxPerformanceHistoryLength::get() as usize {
                        history.remove(0); // Remove oldest record
                    }
                    let _ = history.try_push(performance_record);
                });
                
                // Emit event
                Self::deposit_event(Event::ValidatorActivityUpdated {
                    validator: validator.clone(),
                    blocks_authored,
                    blocks_missed,
                    uptime_percentage,
                });
            }
            
            Ok(())
        }

        /// Report validator misbehavior with evidence
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn report_validator_misbehavior(
            origin: OriginFor<T>,
            validator: T::AccountId,
            evidence: BoundedVec<u8, T::MaxEvidenceLength>,
        ) -> DispatchResult {
            let reporter = ensure_signed(origin)?;
            
            // Ensure the reporter is in the ValidatorSet
            let validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&reporter),
                Error::<T>::ValidatorNotInSet
            );
            
            // Ensure the reported validator exists in the ValidatorSet
            ensure!(
                validator_set.contains(&validator),
                Error::<T>::ValidatorNotFound
            );
            
            // Ensure reporter is not reporting themselves
            ensure!(reporter != validator, Error::<T>::InvalidScore); // Reuse existing error
            
            // Check if this reporter has already reported this validator
            ensure!(
                !MisbehaviorReports::<T>::contains_key(&validator, &reporter),
                Error::<T>::InvalidScore // Reuse existing error for duplicate report
            );
            
            // Store the report
            MisbehaviorReports::<T>::insert(&validator, &reporter, &evidence);
            
            // Count total reports for this validator
            let mut report_count = 0u32;
            for (_, _) in MisbehaviorReports::<T>::iter_prefix(&validator) {
                report_count = report_count.saturating_add(1);
            }
            
            // Emit event
            Self::deposit_event(Event::ValidatorMisbehaviorReported {
                reported_validator: validator.clone(),
                reporter: reporter.clone(),
                evidence: evidence.clone(),
                total_reports: report_count,
            });
            
            // Check if we've reached the threshold for automatic slashing
            let slash_threshold = T::MisbehaviorSlashThreshold::get();
            if report_count >= slash_threshold {
                // Execute automatic slash
                let slash_amount = Self::calculate_slash_amount(&validator)?;
                
                // Perform the slash
                let _imbalance = T::Currency::slash(&validator, slash_amount);
                
                // Unreserve the minimum stake that was locked when joining
                let min_stake = <T as Config>::MinStake::get();
                T::Currency::unreserve(&validator, min_stake);

                // Remove the stake record since it's no longer reserved
                ValidatorStake::<T>::remove(&validator);

                // Emit stake unreservation event
                Self::deposit_event(Event::ValidatorStakeUnreserved {
                    validator: validator.clone(),
                    amount: min_stake,
                });
                
                // Remove the validator from the set
                ValidatorSet::<T>::mutate(|set| {
                    set.retain(|v| v != &validator);
                });
                
                // Clear all reports for this validator since they've been slashed
                let _ = MisbehaviorReports::<T>::clear_prefix(&validator, u32::MAX, None);
                
                // Emit auto-slash event
                Self::deposit_event(Event::ValidatorAutoSlashed {
                    validator,
                    amount: slash_amount,
                    report_count,
                });
            }
            
            Ok(())
        }

        /// Simulate an inference event for a validator (for testing/development)
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn simulate_inference(
            origin: OriginFor<T>,
            validator: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if a specific validator is targeted before moving the value
            let is_targeting_specific_validator = validator.is_some();
            
            // Use the caller as validator if none specified, otherwise use the specified validator
            let target_validator = validator.unwrap_or(who.clone());
            
            // Ensure the target validator is in the ValidatorSet
            let validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&target_validator),
                Error::<T>::ValidatorNotInSet
            );
            
            // If a specific validator is targeted, ensure the caller is also a validator (for governance)
            if is_targeting_specific_validator {
                ensure!(
                    validator_set.contains(&who),
                    Error::<T>::ValidatorNotInSet
                );
            }
            
            // Increment the inference count for the target validator
            Self::record_inference_activity(&target_validator)?;
            
            // Emit event
            Self::deposit_event(Event::InferenceSimulated {
                validator: target_validator.clone(),
                triggered_by: who,
            });
            
            Ok(())
        }

        /// Increase validator stake by reserving additional balance
        #[pallet::call_index(23)]
        #[pallet::weight(<T as Config>::WeightInfo::increase_validator_stake())]
        pub fn increase_validator_stake(
            origin: OriginFor<T>,
            additional_amount: <T as Config>::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if validator is in the validator set
            let validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&who),
                Error::<T>::ValidatorNotInSet
            );

            // Check if validator has sufficient free balance
            let free_balance = T::Currency::free_balance(&who);
            ensure!(
                free_balance >= additional_amount,
                Error::<T>::InsufficientStake
            );

            // Reserve the additional amount
            T::Currency::reserve(&who, additional_amount)
                .map_err(|_| Error::<T>::InsufficientStake)?;

            // Update the stored stake amount
            ValidatorStake::<T>::mutate(&who, |current_stake| {
                *current_stake = current_stake.saturating_add(additional_amount);
            });

            // Emit stake increase event
            Self::deposit_event(Event::ValidatorStakeReserved {
                validator: who.clone(),
                amount: additional_amount,
            });

            // Update validator's stake score based on new total stake
            let new_total_stake = ValidatorStake::<T>::get(&who);
            let new_stake_score = new_total_stake.saturated_into::<u64>();
            
            ValidatorStates::<T>::try_mutate(&who, |maybe_state| {
                if let Some(state) = maybe_state.as_mut() {
                    let old_stake_score = state.current.stake_score;
                    state.current.stake_score = new_stake_score;
                    
                    // Recalculate final score
                    let _ = Self::update_final_score(&who);
                    
                    log::info!("Validator {:?} increased stake: {} -> {}", 
                              who, old_stake_score, new_stake_score);
                }
                Ok::<(), Error<T>>(())
            })?;

            Ok(())
        }

        /// Decrease validator stake by unreserving some balance (must maintain minimum)
        #[pallet::call_index(24)]
        #[pallet::weight(<T as Config>::WeightInfo::decrease_validator_stake())]
        pub fn decrease_validator_stake(
            origin: OriginFor<T>,
            decrease_amount: <T as Config>::Balance,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if validator is in the validator set
            let validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&who),
                Error::<T>::ValidatorNotInSet
            );

            let current_stake = ValidatorStake::<T>::get(&who);
            let min_stake = <T as Config>::MinStake::get();

            // Ensure the remaining stake after decrease is at least the minimum
            ensure!(
                current_stake.saturating_sub(decrease_amount) >= min_stake,
                Error::<T>::InsufficientStake
            );

            // Unreserve the specified amount
            let unreserved = T::Currency::unreserve(&who, decrease_amount);

            // Update the stored stake amount
            ValidatorStake::<T>::mutate(&who, |current_stake| {
                *current_stake = current_stake.saturating_sub(unreserved);
            });

            // Emit stake decrease event
            Self::deposit_event(Event::ValidatorStakeUnreserved {
                validator: who.clone(),
                amount: unreserved,
            });

            // Update validator's stake score based on new total stake
            let new_total_stake = ValidatorStake::<T>::get(&who);
            let new_stake_score = new_total_stake.saturated_into::<u64>();
            
            ValidatorStates::<T>::try_mutate(&who, |maybe_state| {
                if let Some(state) = maybe_state.as_mut() {
                    let old_stake_score = state.current.stake_score;
                    state.current.stake_score = new_stake_score;
                    
                    // Recalculate final score
                    let _ = Self::update_final_score(&who);
                    
                    log::info!("Validator {:?} decreased stake: {} -> {}", 
                              who, old_stake_score, new_stake_score);
                }
                Ok::<(), Error<T>>(())
            })?;

            Ok(())
        }

        /// Distribute epoch rewards using modular reward logic (Root only)
        /// This function distributes rewards based on performance tiers
        #[pallet::call_index(33)]
        #[pallet::weight(Weight::from_parts(50_000, 0))]
        pub fn distribute_epoch_rewards(
            origin: OriginFor<T>,
            total_reward_pool: <T as Config>::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Calculate reward pools based on configured percentages
            let base_pool = (total_reward_pool * T::BaseRewardPercentage::get().into()) / T::FullPercentage::get().into();
            let performance_pool = (total_reward_pool * T::PerformanceRewardPercentage::get().into()) / T::FullPercentage::get().into();
            let top_performer_pool = (total_reward_pool * T::TopPerformerRewardPercentage::get().into()) / T::FullPercentage::get().into();

            // Distribute rewards using the modular logic
            Self::distribute_rewards(base_pool, performance_pool, top_performer_pool)?;

            // Emit event
            Self::deposit_event(Event::EpochRewardsDistributed {
                total_pool: total_reward_pool,
                base_pool,
                performance_pool,
                top_performer_pool,
            });

            log::info!("Epoch rewards distributed: total {:?}, base {:?}, performance {:?}, top performer {:?}",
                      total_reward_pool, base_pool, performance_pool, top_performer_pool);

            Ok(())
        }
    }

    // --- Internal Logic --- //
    impl<T: Config> Pallet<T> {
        /// Recalculate and update the final score for a validator.
        fn update_final_score(validator: &T::AccountId) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let stake_score = state.current.stake_score;
                let inference_score = state.current.inference_score;
                let pos_weight = if !PosWeight::<T>::exists() {
                    let weight = T::DefaultPosWeight::get();
                    PosWeight::<T>::put(weight);
                    weight
                } else {
                    PosWeight::<T>::get()
                };
                let poi_weight = if !PoiWeight::<T>::exists() {
                    let weight = T::DefaultPoiWeight::get();
                    PoiWeight::<T>::put(weight);
                    weight
                } else {
                    PoiWeight::<T>::get()
                };
                let mut final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / T::PercentagePrecision::get() as u64;
                if final_score > T::MaxValidatorScore::get() {
                    final_score = T::MaxValidatorScore::get();
                }
                let old_stake_score = state.current.stake_score;
                let old_inference_score = state.current.inference_score;
                let old_final_score = state.current.final_score;
                state.current.final_score = final_score;
                state.last_active_epoch = Self::current_epoch();
                
                // If score changed significantly, trigger validator reordering
                let score_change = if final_score > old_final_score {
                    final_score - old_final_score
                } else {
                    old_final_score - final_score
                };
                
                // Trigger resort if score changed by more than configured percentage or threshold
                let percentage_threshold = old_final_score / (T::FullPercentage::get() as u64 / T::ScoreChangePercentage::get() as u64);
                let significant_change = score_change > percentage_threshold.max(T::ScoreChangeThreshold::get());
                if significant_change && Self::active_validators().contains(validator) {
                    // Schedule a resort by updating a flag or doing it immediately
                    let mut active_validators = Self::active_validators();
                    Self::sort_validators_by_score(&mut active_validators);
                    ActiveValidators::<T>::put(active_validators);
                }
                if state.history.len() == state.history.capacity() && !state.history.is_empty() {
                    state.history.remove(0);
                }
                let _ = state.history.try_push(EpochStats {
                    epoch: Self::current_epoch(),
                    stake_score,
                    inference_score,
                    final_score,
                    authored_blocks: state.current.authored_blocks,
                    missed_blocks: state.current.missed_blocks,
                });
                Self::deposit_event(Event::ValidatorScoreUpdated {
                    validator: validator.clone(),
                    stake_score,
                    inference_score,
                    final_score,
                });
                
                // Emit detailed score update event
                Self::deposit_event(Event::ValidatorScoreUpdatedDetailed {
                    validator: validator.clone(),
                    old_stake_score,
                    new_stake_score: stake_score,
                    old_inference_score,
                    new_inference_score: inference_score,
                    old_final_score,
                    new_final_score: final_score,
                    epoch: Self::current_epoch(),
                });
                Ok::<(), Error<T>>(())
            }).map_err(Into::into)
        }

        /// Apply score decay to a validator if inactive.
        pub fn apply_score_decay(validator: &T::AccountId, current_epoch: u32) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let last_active = state.last_active_epoch;
                let inactive_epochs = current_epoch.saturating_sub(last_active);
                if inactive_epochs > 0 {
                    let decay_rate = <T as pallet::Config>::ValidatorScoreDecay::get();
                    let decay_amount = state.current.final_score.saturating_mul(decay_rate as u64) / T::PercentagePrecision::get() as u64;
                    let old_score = state.current.final_score;
                    state.current.final_score = state.current.final_score.saturating_sub(decay_amount);
                    if state.current.final_score > T::MaxValidatorScore::get() {
                        state.current.final_score = T::MaxValidatorScore::get();
                    }
                    Self::deposit_event(Event::ValidatorScoreDecayed {
                        validator: validator.clone(),
                        old_score,
                        new_score: state.current.final_score,
                    });
                    if state.current.final_score < <T as Config>::MinValidatorScore::get() as u64 {
                        Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold)
                            .map_err(|_| Error::<T>::ValidatorNotFound)?;
                    }
                }
                Ok::<(), Error<T>>(())
            }).map_err(Into::into)
        }

        /// Boost a validator's score for a given reason.
        fn boost_score(
            validator: &T::AccountId,
            amount: u64,
            reason: ScoreBoostReason,
        ) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let old_score = state.current.final_score;
                state.current.final_score = state.current.final_score.saturating_add(amount);
                if state.current.final_score > T::MaxValidatorScore::get() {
                    state.current.final_score = T::MaxValidatorScore::get();
                }
                state.last_active_epoch = Self::current_epoch();
                if state.history.len() == state.history.capacity() {
                    state.history.remove(0);
                }
                let _ = state.history.try_push(EpochStats {
                    epoch: Self::current_epoch(),
                    stake_score: state.current.stake_score,
                    inference_score: state.current.inference_score,
                    final_score: state.current.final_score,
                    authored_blocks: state.current.authored_blocks,
                    missed_blocks: state.current.missed_blocks,
                });
                Self::deposit_event(Event::ValidatorScoreBoosted {
                    validator: validator.clone(),
                    old_score,
                    new_score: state.current.final_score,
                    reason,
                });
                Ok::<(), Error<T>>(())
            }).map_err(Into::into)
        }

        /// Calculate the slash amount for a validator based on their stake
        fn calculate_slash_amount(validator: &T::AccountId) -> Result<<T as pallet::Config>::Balance, DispatchError> {
            // Ensure validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(validator),
                Error::<T>::ValidatorNotFound
            );
            
            // Calculate slash amount as a percentage of their stake
            let stake_balance = T::Currency::free_balance(validator);
            let slash_percent = T::SlashPercent::get();
            
            // Use saturating operations to avoid overflow
            let slash_amount = stake_balance / 100u32.into() * slash_percent.into();
            
            Ok(slash_amount)
        }

        /// Record inference activity for a validator
        fn record_inference_activity(validator: &T::AccountId) -> DispatchResult {
            let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u32>();
            
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                
                // Increment inference count
                state.inference_count = state.inference_count.saturating_add(1);
                
                // Update last active block
                state.last_active_block = current_block;
                
                // Update last active epoch
                state.last_active_epoch = Self::current_epoch();
                
                Ok::<(), Error<T>>(())
            })?;
            
            // Also update the separate ValidatorInferenceCount storage for compatibility
            ValidatorInferenceCount::<T>::mutate(validator, |count| {
                *count = count.saturating_add(1);
            });
            
            Ok(())
        }

        /// Record a missed block for a validator.
        pub fn record_missed_block(validator: &T::AccountId) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.missed_blocks = state.current.missed_blocks.saturating_add(1);
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;

            Ok(())
        }

        /// Record block authorship for a validator and boost score.
        pub fn record_block_authorship(validator: &T::AccountId) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.authored_blocks = state.current.authored_blocks.saturating_add(1);
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;

            Self::boost_score(
                validator,
                T::BlockAuthorshipBoost::get(),
                ScoreBoostReason::ValidBlockAuthored,
            )
        }

        /// Handle a valid inference result for a validator.
        fn handle_valid_inference(
            validator: &T::AccountId,
            confidence: u32,
        ) -> DispatchResult {
            let boost_amount = if confidence >= T::InferenceConfidenceThresholdHigh::get() {
                T::InferenceBoostHigh::get()
            } else if confidence >= T::InferenceConfidenceThresholdLow::get() {
                T::InferenceBoostMedium::get()
            } else {
                T::InferenceBoostLow::get()
            };
            
            // Update inference count and last active block
            let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                if let Some(state) = maybe_state {
                    state.inference_count = state.inference_count.saturating_add(1);
                    state.last_active_block = current_block;
                }
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;
            
            Self::boost_score(
                validator,
                boost_amount,
                ScoreBoostReason::ValidInference,
            )
        }

        /// Handle an invalid inference result for a validator.
        fn handle_invalid_inference(
            validator: &T::AccountId,
            severity: InferenceErrorSeverity,
        ) -> DispatchResult {
            let penalty = match severity {
                InferenceErrorSeverity::High => T::InferencePenaltyHigh::get(),
                InferenceErrorSeverity::Medium => T::InferencePenaltyMedium::get(),
                InferenceErrorSeverity::Low => T::InferencePenaltyLow::get(),
            };
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let new_score = state.current.final_score.saturating_sub(penalty);
                state.current.final_score = new_score;
                if new_score < <T as Config>::MinValidatorScore::get() as u64 {
                    Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold)
                        .map_err(|_| Error::<T>::ValidatorNotFound)?;
                }
                Ok::<(), Error<T>>(())
            }).map_err(Into::into)
        }

        /// Eject a validator from the active set for a given reason.
        fn eject_validator(validator: &T::AccountId, reason: EjectionReason) -> DispatchResult {
            let mut active_validators = ActiveValidators::<T>::get();
            if let Some(pos) = active_validators.iter().position(|v| v == validator) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
            }
            Self::deposit_event(Event::ValidatorEjected {
                validator: validator.clone(),
                reason,
            });
            Ok(())
        }

        /// Check if the current block number is an epoch boundary.
        fn is_epoch_boundary(block_number: u32) -> bool {
            let epoch_length = T::EpochLength::get();
            block_number > 0 && block_number % epoch_length == 0
        }

        /// Handle comprehensive epoch transition with all required steps.
        fn handle_comprehensive_epoch_transition(block_number: u32) -> Weight {
            let mut weight = Weight::zero();
            
            // Step 1: Run score aggregation for all validators
            weight = weight.saturating_add(Self::run_comprehensive_score_aggregation());
            
            // Step 2: Generate automatic proposals based on current state
            weight = weight.saturating_add(Self::generate_epoch_proposals());
            
            // Step 3: Execute or queue proposals
            weight = weight.saturating_add(Self::process_epoch_proposals());
            
            // Step 4: Handle epoch transition (always run, regardless of governance mode)
            weight = weight.saturating_add(Self::handle_epoch_transition());
            
            // Step 5: Finalize epoch (optional - finalize last epoch's best block)
            weight = weight.saturating_add(Self::finalize_previous_epoch(block_number));
            
            // Step 6: Update epoch-specific metrics and cleanup
            weight = weight.saturating_add(Self::update_epoch_metrics(block_number));
            
            weight
        }

        /// Handle regular block processing for non-epoch boundary blocks.
        fn handle_regular_block_processing(block_number: u32, current_epoch: u32) -> Weight {
            let mut weight = Weight::zero();
            
            // 1. Validate block authorship and update validator metrics (every block)
            Self::process_block_authorship(block_number);
            
            // 2. Apply score decay for inactive validators
            if block_number % T::ScoreDecayInterval::get() == 0 {
                let decay_weight = Self::apply_validator_score_decay(current_epoch);
                weight = weight.saturating_add(decay_weight);
            }
            
            // 3. Update validator participation rates
            if block_number % T::ParticipationUpdateInterval::get() == 0 {
                let participation_weight = Self::update_validator_participation_rates();
                weight = weight.saturating_add(participation_weight);
                
                // Resort validators by updated scores
                let mut active_validators = Self::active_validators();
                Self::sort_validators_by_score(&mut active_validators);
                ActiveValidators::<T>::put(active_validators);
            }
            
            // 4. Check for low-performing validators
            if block_number % T::UnderperformanceCheckInterval::get() == 0 {
                Self::check_and_handle_underperforming_validators();
            }
            
            // 5. Generate automatic validator proposals based on scores
            if block_number % T::ValidatorProposalInterval::get() == 0 {
                Self::generate_automatic_validator_proposals();
            }
            
            // 6. Process expired leave requests (check every N blocks for timely processing)
            if block_number % T::LeaveRequestCheckInterval::get() == 0 {
                Self::process_expired_leave_requests(block_number);
                // Also cleanup recently removed validators whose cooldown has expired
                Self::cleanup_recently_removed_validators(block_number);
            }

            // 7. Emit periodic health metrics
            if block_number % T::HealthMetricsInterval::get() == 0 {
                Self::emit_dcf_health_metrics(block_number);
            }
            
            weight
        }

        /// Determine if an epoch transition should occur.
        fn should_transition_epoch(
            now: BlockNumberFor<T>,
            current_epoch: u32,
            epoch_config: &EpochConfig,
        ) -> bool {
            let blocks_per_epoch = epoch_config.blocks_per_epoch;
            let epoch_start_block = current_epoch.saturating_mul(blocks_per_epoch);
            let now_u32: u32 = now.saturated_into();
            now_u32 >= epoch_start_block + blocks_per_epoch
        }

        /// Handle the logic for transitioning to a new epoch.
        fn handle_epoch_transition() -> Weight {
            // Always allow automatic epoch transitions, but behavior differs based on governance mode
            let governance_mode = GovernanceModeEnabled::<T>::get();
            let current_epoch = Self::current_epoch();
            let next_epoch = current_epoch.saturating_add(1);
            CurrentEpoch::<T>::put(next_epoch);

            // Apply pending join/leave requests (always allowed)
            Self::apply_pending_validator_actions();

            // Ensure validators are sorted by final score for the new epoch
            let mut active_validators = ActiveValidators::<T>::get();
            Self::sort_validators_by_score(&mut active_validators);
            ActiveValidators::<T>::put(active_validators.clone());
            
            // Log epoch transition details
            if governance_mode {
                log::info!("DCF: Auto epoch transition {} -> {} (governance mode enabled) - {} active validators", 
                          current_epoch, next_epoch, active_validators.len());
            } else {
                log::info!("DCF: Auto epoch transition {} -> {} (standard mode) - {} active validators", 
                          current_epoch, next_epoch, active_validators.len());
            }
            
            // Always emit epoch events regardless of governance mode
            Self::deposit_event(Event::EpochStarted {
                epoch: next_epoch,
                validators: active_validators.clone().into_inner(),
            });
            
            // Emit comprehensive epoch transition event with governance mode info
            Self::deposit_event(Event::EpochTransitioned {
                old_epoch: current_epoch,
                new_epoch: next_epoch,
                active_validators: active_validators.clone().into_inner(),
                total_validators: ValidatorSet::<T>::get().len() as u32,
            });



            // --- EpochHistory recording ---
            let score_snapshot: BoundedVec<_, <T as Config>::MaxValidators> =
                BoundedVec::truncate_from(active_validators.iter().map(|v| {
                    let score = ValidatorStates::<T>::get(v).map(|s| s.current.final_score).unwrap_or_default();
                    (v.clone(), score)
                }).collect::<Vec<_>>());
            let inference_summary: BoundedVec<_, <T as Config>::MaxValidators> =
                BoundedVec::truncate_from(active_validators.iter().map(|v| {
                    let inf = poi::Pallet::<T>::inference_results(v).map(|(result, _)| result as u64);
                    (v.clone(), inf)
                }).collect::<Vec<_>>());

            let mut histories = EpochHistories::<T>::get();
            let new_history = EpochHistory::<T> {
                epoch_number: next_epoch,
                active_validators: active_validators.clone(),
                score_snapshot,
                inference_summary,
            };
            if histories.len() == <T as Config>::MaxEpochHistory::get() as usize {
                histories.remove(0);
            }
            let _ = histories.try_push(new_history);
            EpochHistories::<T>::put(histories);

            <T as Config>::WeightInfo::on_initialize()
        }
    }

    // --- Runtime Hooks --- //
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Called at the beginning of each block.
        /// This is the main entry point for DCF's automatic consensus management.
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut weight = <T as Config>::WeightInfo::on_initialize();
            let block_number = now.saturated_into::<u32>();
            let current_epoch = Self::current_epoch();
            
            // Check if this is an epoch boundary
            if Self::is_epoch_boundary(block_number) {
                // Emit epoch boundary detection event
                Self::deposit_event(Event::EpochBoundaryDetected {
                    block_number,
                    epoch: current_epoch,
                    governance_mode: Self::governance_mode_enabled(),
                });
                
                // Handle comprehensive epoch transition
                weight = weight.saturating_add(Self::handle_comprehensive_epoch_transition(block_number));
                

            } else {
                // Regular block processing (non-epoch boundary)
                weight = weight.saturating_add(Self::handle_regular_block_processing(block_number, current_epoch));
            }

            weight
        }

        /// Called at the end of each block.
        fn on_finalize(_n: BlockNumberFor<T>) {
            let validators = ValidatorSet::<T>::get();
            for validator in validators.iter() {
                let _ = Self::update_final_score(validator);
            }
        }

        /// Off-chain worker for automatic PoI score computation and submission
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            
            // Run off-chain worker at configured intervals to reduce overhead
            if (block_number.saturated_into::<u32>()) % T::OffchainWorkerInterval::get() != 0 {
                return;
            }

            let result = Self::run_offchain_computation(block_number);
            let _ = result;
        }


    }



    // --- Off-chain Worker Implementation --- //
    impl<T: Config> Pallet<T> {
        /// Run off-chain computation for PoI score collection and submission
        fn run_offchain_computation(block_number: BlockNumberFor<T>) -> Result<(), &'static str> {
            // Create a lock to prevent multiple workers from running simultaneously
            let mut lock = StorageLock::<BlockAndTime<frame_system::Pallet<T>>>::with_block_and_time_deadline(
                b"dcf::offchain_worker",
                block_number.saturated_into::<u32>(),
                Duration::from_millis(T::OffchainWorkerTimeout::get()),
            );

            let _guard = lock.try_lock().map_err(|_| "Failed to acquire lock")?;



            // Get current validators
            let validators = Self::validator_set();
            let current_epoch = Self::current_epoch();

            for validator in validators.iter() {
                // Collect inference data for this validator
                if let Ok(inference_data) = Self::collect_inference_data(validator, current_epoch) {
                    // Compute PoI score based on collected data
                    let poi_score = Self::compute_poi_score(&inference_data);
                    
                    // Submit unsigned transaction to update the score
                    let _ = Self::submit_poi_score_update(validator.clone(), poi_score, block_number);
                }
            }

            log::info!("DCF off-chain worker completed computation");
            Ok(())
        }

        /// Collect inference data for a validator from external sources
        fn collect_inference_data(
            validator: &T::AccountId,
            epoch: u32,
        ) -> Result<InferenceData<T>, &'static str> {
            // Check if we have cached inference results from PoI pallet
            if let Some((result, confidence)) = poi::Pallet::<T>::inference_results(validator) {
                return Ok(InferenceData {
                    validator: validator.clone(),
                    epoch,
                    inference_result: result,
                    confidence_score: confidence,
                    timestamp: Self::get_offchain_timestamp(),
                    data_sources: vec![b"poi_pallet".to_vec()],
                });
            }

            // Try to collect from external inference endpoints
            Self::collect_from_external_sources(validator, epoch)
        }

        /// Collect inference data from external sources via HTTP requests
        fn collect_from_external_sources(
            validator: &T::AccountId,
            epoch: u32,
        ) -> Result<InferenceData<T>, &'static str> {
            // This is a placeholder for external data collection
            // In a real implementation, you would make HTTP requests to inference providers
            
            // For now, we'll simulate inference data collection
            let simulated_result = Self::simulate_inference_computation(validator, epoch);
            
            Ok(InferenceData {
                validator: validator.clone(),
                epoch,
                inference_result: simulated_result.0,
                confidence_score: simulated_result.1,
                timestamp: Self::get_offchain_timestamp(),
                data_sources: vec![b"simulation".to_vec()],
            })
        }

        /// Simulate inference computation (placeholder for real implementation)
        fn simulate_inference_computation(validator: &T::AccountId, epoch: u32) -> (u32, u32) {
            // Use validator account and epoch to generate deterministic but varied results
            let validator_bytes = validator.encode();
            let mut hash_input = validator_bytes;
            hash_input.extend_from_slice(&epoch.to_le_bytes());
            
            // Simple hash-based simulation
            let hash = sp_core::hashing::blake2_256(&hash_input);
            let result = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]) % T::PercentagePrecision::get();
            let confidence = T::InferenceConfidenceThresholdLow::get() + 
                (u32::from_le_bytes([hash[4], hash[5], hash[6], hash[7]]) % 
                 (T::InferenceConfidenceThresholdHigh::get() - T::InferenceConfidenceThresholdLow::get()));
            
            (result, confidence)
        }

        /// Compute PoI score based on inference data
        fn compute_poi_score(data: &InferenceData<T>) -> u64 {
            let base_score = data.inference_result as u64;
            let confidence_multiplier = data.confidence_score as u64;
            
            // Apply confidence weighting: higher confidence = higher score
            let weighted_score = (base_score * confidence_multiplier) / T::PercentagePrecision::get() as u64;
            
            // Cap the score at maximum allowed
            weighted_score.min(<T as Config>::MaxValidatorScore::get())
        }

        /// Get current timestamp for off-chain operations
        fn get_offchain_timestamp() -> u64 {
            sp_io::offchain::timestamp().unix_millis()
        }

        /// Store computed PoI score in off-chain storage for later retrieval
        fn submit_poi_score_update(
            validator: T::AccountId,
            poi_score: u64,
            block_number: BlockNumberFor<T>,
        ) -> Result<(), &'static str> {
            log::info!(
                "DCF: Computed PoI score for validator={:?}, score={}, block={}",
                validator,
                poi_score,
                block_number.saturated_into::<u32>()
            );
            
            // Store the computed score in off-chain storage for later retrieval
            Self::store_offchain_poi_score(&validator, poi_score, block_number)?;
            
            Ok(())
        }

        /// Store computed PoI score in off-chain storage
        fn store_offchain_poi_score(
            validator: &T::AccountId,
            score: u64,
            block_number: BlockNumberFor<T>,
        ) -> Result<(), &'static str> {
            let key = format!("dcf::poi_score::{:?}::{}", validator, block_number.saturated_into::<u32>());
            let storage_ref = StorageValueRef::persistent(key.as_bytes());
            
            let score_data: OffchainPoiScore<T> = OffchainPoiScore {
                validator: validator.clone(),
                score,
                block_number: block_number.saturated_into::<u32>(),
                timestamp: Self::get_offchain_timestamp(),
            };
            
            storage_ref.set(&score_data);
            Ok(())
        }

        /// Retrieve computed PoI score from off-chain storage
        fn get_offchain_poi_score(
            validator: &T::AccountId,
            block_number: u32,
        ) -> Result<Option<u64>, &'static str> {
            let key = format!("dcf::poi_score::{:?}::{}", validator, block_number);
            let storage_ref = StorageValueRef::persistent(key.as_bytes());
            
            match storage_ref.get::<OffchainPoiScore<T>>() {
                Ok(Some(score_data)) => Ok(Some(score_data.score)),
                Ok(None) => Ok(None),
                Err(_) => Err("Failed to retrieve PoI score from off-chain storage"),
            }
        }

        /// Get current timestamp from the timestamp pallet
        fn get_current_timestamp() -> u64 {
            // Use a simple timestamp for now - in production this would be from timestamp pallet
            sp_io::offchain::timestamp().unix_millis()
        }

        /// Calculate validator uptime percentage
        fn calculate_uptime_percentage(validator: &T::AccountId) -> u32 {
            let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
            let last_seen = Self::validator_last_seen(validator);
            let join_time_block = ValidatorJoinTime::<T>::get(validator)
                .map(|timestamp| {
                    // Convert timestamp to approximate block number
                    // This is a rough approximation - in production you'd want more precise tracking
                    let blocks_since_genesis = current_block;
                    let time_since_genesis = Self::get_current_timestamp().saturating_sub(timestamp);
                    let estimated_block_time = T::EstimatedBlockTime::get();
                    let estimated_blocks_since_join = time_since_genesis / estimated_block_time;
                    blocks_since_genesis.saturating_sub(estimated_blocks_since_join as u32)
                })
                .unwrap_or(0);
            
            if current_block <= join_time_block {
                return T::PercentagePrecision::get(); // Full percentage if just joined
            }
            
            let total_blocks_since_join = current_block.saturating_sub(join_time_block);
            let blocks_since_last_seen = current_block.saturating_sub(last_seen);
            
            if total_blocks_since_join == 0 {
                return T::PercentagePrecision::get(); // Full percentage
            }
            
            let active_blocks = total_blocks_since_join.saturating_sub(blocks_since_last_seen);
            let uptime_percentage = (active_blocks as u64 * T::PercentagePrecision::get() as u64) / total_blocks_since_join as u64;
            
            uptime_percentage.min(T::PercentagePrecision::get() as u64) as u32 // Cap at full percentage
        }
    }

    // --- Genesis Configuration --- //
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Initial validators to be added at genesis
        pub validators: Vec<T::AccountId>,
        /// Initial inference scores for validators (must match validators length or be empty)
        pub validator_scores: Vec<u32>,
        /// Initial stakes for validators (must match validators length or be empty for default MinStake)
        pub validator_stakes: Vec<<T as pallet::Config>::Balance>,
        /// Starting epoch number (usually 0)
        pub current_epoch: u32,
        /// Epoch configuration parameters
        pub epoch_config: EpochConfig,
        /// Optional validator names (must match validators length or be empty)
        pub validator_names: Vec<Option<Vec<u8>>>,
        /// Whether to perform strict validation of genesis parameters
        pub strict_validation: bool,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Perform comprehensive validation if strict_validation is enabled
            if self.strict_validation {
                Self::validate_genesis_config(self);
            }

            // Ensure validators don't exceed MaxValidators
            if self.validators.len() > <T as pallet::Config>::MaxValidators::get() as usize {
                panic!("Genesis validators ({}) exceed MaxValidators ({})",
                       self.validators.len(), <T as pallet::Config>::MaxValidators::get());
            }

            // Check for duplicate validators
            let mut unique_validators = sp_std::collections::btree_set::BTreeSet::new();
            for validator in &self.validators {
                if !unique_validators.insert(validator) {
                    panic!("Duplicate validator found in genesis config: {:?}", validator);
                }
            }

            // Prepare stakes - use provided stakes or default to MinStake
            let stakes = if self.validator_stakes.is_empty() {
                // Use minimum stake for all validators if stakes not specified
                vec![<T as pallet::Config>::MinStake::get(); self.validators.len()]
            } else if self.validator_stakes.len() == self.validators.len() {
                // Validate all stakes meet minimum requirement
                for (i, stake) in self.validator_stakes.iter().enumerate() {
                    if *stake < <T as pallet::Config>::MinStake::get() {
                        panic!("Genesis validator {} stake ({:?}) below MinStake ({:?})",
                               i, stake, <T as pallet::Config>::MinStake::get());
                    }
                }
                self.validator_stakes.clone()
            } else {
                panic!("validator_stakes length ({}) must match validators length ({}) or be empty",
                       self.validator_stakes.len(), self.validators.len());
            };

            // Prepare scores - use provided scores or default to 0
            let scores = if self.validator_scores.is_empty() {
                vec![0u32; self.validators.len()]
            } else if self.validator_scores.len() == self.validators.len() {
                self.validator_scores.clone()
            } else {
                panic!("validator_scores length ({}) must match validators length ({}) or be empty",
                       self.validator_scores.len(), self.validators.len());
            };

            // Prepare names - use provided names or default to None
            let names = if self.validator_names.is_empty() {
                vec![None; self.validators.len()]
            } else if self.validator_names.len() == self.validators.len() {
                self.validator_names.clone()
            } else {
                panic!("validator_names length ({}) must match validators length ({}) or be empty",
                       self.validator_names.len(), self.validators.len());
            };

            // Set up validator set
            ValidatorSet::<T>::put(
                BoundedVec::try_from(self.validators.clone())
                    .expect("Initial validators exceed MaxValidators"),
            );

            // Initialize each validator with their stake, score, and name
            for (((validator, score), stake), name) in self.validators.iter()
                .zip(scores.iter())
                .zip(stakes.iter())
                .zip(names.iter()) {

                // Verify validator has sufficient balance for stake reservation
                let free_balance = T::Currency::free_balance(validator);
                if free_balance < *stake {
                    if self.strict_validation {
                        panic!("Genesis validator {:?} has insufficient balance ({:?}) for stake ({:?})",
                               validator, free_balance, stake);
                    } else {
                        log::warn!("Genesis validator {:?} has insufficient balance ({:?}) for stake ({:?}), skipping",
                                  validator, free_balance, stake);
                        continue;
                    }
                }

                // Reserve the stake for the validator
                if let Err(e) = T::Currency::reserve(validator, *stake) {
                    if self.strict_validation {
                        panic!("Failed to reserve stake for genesis validator {:?}: {:?}", validator, e);
                    } else {
                        log::warn!("Failed to reserve stake for genesis validator {:?}: {:?}, skipping", validator, e);
                        continue;
                    }
                }

                // Store the stake amount in ValidatorStake storage
                ValidatorStake::<T>::insert(validator, *stake);

                // Calculate scores based on stake and initial score
                let stake_score = (*stake).saturated_into::<u64>() / T::RewardBoostDivisor::get();
                let inference_score = *score as u64;

                let pos_weight = T::DefaultPosWeight::get();
                let poi_weight = T::DefaultPoiWeight::get();
                let final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / T::PercentagePrecision::get() as u64;

                // Prepare validator name if provided
                let validator_name = name.as_ref().map(|n| {
                    BoundedVec::try_from(n.clone())
                        .unwrap_or_else(|_| {
                            log::warn!("Genesis validator name too long, truncating");
                            BoundedVec::truncate_from(n.clone())
                        })
                });

                // Create initial history entry
                let mut history = BoundedVec::<EpochStats, T::MaxValidatorHistorySize>::default();
                let _ = history.try_push(EpochStats {
                    epoch: self.current_epoch,
                    stake_score,
                    inference_score,
                    final_score,
                    authored_blocks: 0,
                    missed_blocks: 0,
                });

                // Initialize validator state
                ValidatorStates::<T>::insert(
                    validator,
                    ValidatorState {
                        last_active_epoch: self.current_epoch,
                        current: EpochStats {
                            epoch: self.current_epoch,
                            stake_score,
                            inference_score,
                            final_score,
                            authored_blocks: 0,
                            missed_blocks: 0,
                        },
                        history,
                        uptime: 1, // Start with 1 epoch of uptime
                        inference_success_count: 0,
                        participation_rate: T::FullPercentage::get(), // Start with 100% participation
                        inference_count: 0,
                        last_active_block: 1, // Genesis block
                        name: validator_name,
                        trust_score: T::MaxTrustScore::get() / 2, // Start with 50% trust score
                    },
                );

                log::info!("Genesis validator {:?} initialized with stake: {:?}, final_score: {}, name: {:?}",
                          validator, stake, final_score, name);
            }

            // Initialize active validators with all genesis validators
            ActiveValidators::<T>::put(
                BoundedVec::try_from(self.validators.clone())
                    .expect("Initial validators exceed MaxValidators"),
            );

            CurrentEpoch::<T>::put(self.current_epoch);
            EpochConfigStorage::<T>::put(self.epoch_config.clone());
            PosWeight::<T>::put(T::DefaultPosWeight::get());
            PoiWeight::<T>::put(T::DefaultPoiWeight::get());
            
            // Initialize finalized block to genesis block
            LastFinalizedBlock::<T>::put(1);

            log::info!("DCF Genesis completed: {} validators initialized in epoch {}",
                      self.validators.len(), self.current_epoch);
        }
    }

    impl<T: Config> GenesisConfig<T> {
        /// Validate genesis configuration for consistency and correctness
        fn validate_genesis_config(&self) {
            // Validate validator count
            if self.validators.is_empty() {
                panic!("Genesis must have at least one validator");
            }

            if self.validators.len() > <T as pallet::Config>::MaxValidators::get() as usize {
                panic!("Genesis validators ({}) exceed MaxValidators ({})",
                       self.validators.len(), <T as pallet::Config>::MaxValidators::get());
            }

            // Validate array lengths match or are empty (handled in build())
            if !self.validator_scores.is_empty() && self.validator_scores.len() != self.validators.len() {
                panic!("validator_scores length must match validators length or be empty");
            }

            if !self.validator_stakes.is_empty() && self.validator_stakes.len() != self.validators.len() {
                panic!("validator_stakes length must match validators length or be empty");
            }

            if !self.validator_names.is_empty() && self.validator_names.len() != self.validators.len() {
                panic!("validator_names length must match validators length or be empty");
            }

            // Validate stakes meet minimum requirements
            for (i, stake) in self.validator_stakes.iter().enumerate() {
                if *stake < <T as pallet::Config>::MinStake::get() {
                    panic!("Genesis validator {} stake below minimum: {:?} < {:?}",
                           i, stake, <T as pallet::Config>::MinStake::get());
                }
            }

            // Validate names don't exceed maximum length
            for (i, name_opt) in self.validator_names.iter().enumerate() {
                if let Some(name) = name_opt {
                    if name.len() > T::MaxValidatorNameSize::get() as usize {
                        panic!("Genesis validator {} name too long: {} > {}",
                               i, name.len(), T::MaxValidatorNameSize::get());
                    }
                }
            }

            // Validate epoch configuration
            if self.epoch_config.blocks_per_epoch == 0 {
                panic!("Epoch length cannot be zero");
            }

            log::info!("Genesis configuration validation passed");
        }
    }

    // --- Public Helper Functions --- //
    impl<T: Config> Pallet<T> {
        /// Validate if a block author is an active validator.
        pub fn validate_block_author(block_number: u32, author: T::AccountId) {
            if !Self::is_validator_active(&author) {
                Self::deposit_event(Event::InvalidAuthor {
                    block_number,
                    author,
                });
            }
        }

        /// Validate if the block author matches the expected author for the given block number.
        /// Returns true if valid, false if mismatch (and emits AuthorMismatch event).
        pub fn validate_expected_author(block_number: u32, actual_author: T::AccountId) -> bool {
            if let Some(expected_author) = Self::get_expected_author(block_number) {
                if actual_author != expected_author {
                    // Emit AuthorMismatch event before returning false
                    Self::deposit_event(Event::AuthorMismatch {
                        block_number,
                        expected_author,
                        actual_author,
                    });
                    return false;
                }
            }
            true
        }

        /// Check if an account is an active validator.
        pub fn is_validator_active(author: &T::AccountId) -> bool {
            Self::active_validators().contains(author)
        }

        /// Get the expected author for a given block number using weighted selection based on final scores.
        pub fn get_expected_author(block_number: u32) -> Option<T::AccountId> {
            let validators = Self::active_validators();
            if validators.is_empty() {
                log::warn!("DCF: No active validators available for block authorship at block {}", block_number);
                return None;
            }

            // Calculate total weighted score using fresh PoS and PoI scores
            let mut total_weight = 0u64;
            let mut validator_weights: Vec<(T::AccountId, u64, u64, u64)> = Vec::new(); // (validator, combined_score, pos_score, poi_score)
            
            let pos_weight = Self::pos_weight();
            let poi_weight = Self::poi_weight();
            
            for validator in validators.iter() {
                // Get fresh PoS score from stake
                let stake = pos::Pallet::<T>::stake(validator);
                let pos_score = stake.saturated_into::<u64>();
                
                // Get fresh PoI score from inference results
                let poi_score = poi::Pallet::<T>::inference_results(validator)
                    .map(|(result, _)| result as u64)
                    .unwrap_or(0);
                
                // Calculate combined score using current weights
                let mut combined_score = (pos_score.saturating_mul(pos_weight) + poi_score.saturating_mul(poi_weight)) 
                    / T::PercentagePrecision::get() as u64;
                
                // Cap at maximum score
                if combined_score > T::MaxValidatorScore::get() {
                    combined_score = T::MaxValidatorScore::get();
                }
                
                // Ensure minimum weight of 1 for all validators
                let weight = combined_score.max(1);
                total_weight = total_weight.saturating_add(weight);
                validator_weights.push((validator.clone(), weight, pos_score, poi_score));
                
                log::debug!("DCF: Validator {:?} - Combined: {}, PoS: {} (weight: {}%), PoI: {} (weight: {}%)", 
                           validator, combined_score, pos_score, pos_weight, poi_score, poi_weight);
            }

            if total_weight == 0 {
                // Fallback to round-robin if all scores are zero
                let idx = (block_number as usize) % validators.len();
                return validators.get(idx).cloned();
            }

            // Use block number as seed for deterministic selection
            let target = (block_number as u64 * 2654435761u64) % total_weight; // Using a large prime for better distribution
            let mut cumulative_weight = 0u64;

            for (validator, weight, pos_score, poi_score) in validator_weights {
                cumulative_weight = cumulative_weight.saturating_add(weight);
                if target < cumulative_weight {
                    log::info!("DCF: Selected author {:?} for block {} (Combined: {}, PoS: {}, PoI: {}, Target: {}/{})", 
                               validator, block_number, weight, pos_score, poi_score, target, total_weight);
                    return Some(validator);
                }
            }

            // Fallback to first validator if something goes wrong
            validators.get(0).cloned()
        }

        /// Generate validator proposals based on combined PoS and PoI scores
        /// This function evaluates all validators and suggests actions based on their performance
        pub fn generate_validator_proposals() -> Vec<(T::AccountId, ProposalAction<T>, u64, u64, u64)> {
            let mut proposals = Vec::new();
            let validators = Self::validator_set();
            let pos_weight = Self::pos_weight();
            let poi_weight = Self::poi_weight();
            let min_score_threshold = <T as Config>::MinValidatorScore::get() as u64;
            
            for validator in validators.iter() {
                // Get fresh PoS and PoI scores
                let stake = pos::Pallet::<T>::stake(validator);
                let pos_score = stake.saturated_into::<u64>();
                
                let poi_score = poi::Pallet::<T>::inference_results(validator)
                    .map(|(result, _)| result as u64)
                    .unwrap_or(0);
                
                // Calculate combined score
                let mut combined_score = (pos_score.saturating_mul(pos_weight) + poi_score.saturating_mul(poi_weight)) 
                    / T::PercentagePrecision::get() as u64;
                
                if combined_score > T::MaxValidatorScore::get() {
                    combined_score = T::MaxValidatorScore::get();
                }
                
                // Generate proposals based on performance
                if combined_score < min_score_threshold {
                    // Propose ejection for underperforming validators
                    let action = ProposalAction::Eject { 
                        validator: validator.clone(), 
                        reason: EjectionReason::ScoreBelowThreshold 
                    };
                    proposals.push((validator.clone(), action, combined_score, pos_score, poi_score));
                } else if combined_score >= T::MaxValidatorScore::get() * T::HighPerformancePercentage::get() as u64 / T::FullPercentage::get() as u64 {
                    // Propose reward for high-performing validators
                    let reward_amount = T::ValidatorReward::get();
                    let action = ProposalAction::Reward { 
                        validator: validator.clone(), 
                        amount: reward_amount 
                    };
                    proposals.push((validator.clone(), action, combined_score, pos_score, poi_score));
                }
                
                // Check for imbalanced scores (too much reliance on one component)
                let total_weighted = pos_score.saturating_mul(pos_weight) + poi_score.saturating_mul(poi_weight);
                if total_weighted > 0 {
                    let pos_contribution = (pos_score.saturating_mul(pos_weight) * T::FullPercentage::get() as u64) / total_weighted;
                    let poi_contribution = (poi_score.saturating_mul(poi_weight) * T::FullPercentage::get() as u64) / total_weighted;
                    
                    // If one component dominates too much, log a warning
                    if pos_contribution > T::MaxPosContribution::get() as u64 {
                        log::warn!("DCF: Validator {:?} relies heavily on PoS ({}%) - PoI score: {}", 
                                  validator, pos_contribution, poi_score);
                    } else if poi_contribution > T::MaxPoiContribution::get() as u64 {
                        log::warn!("DCF: Validator {:?} relies heavily on PoI ({}%) - PoS score: {}", 
                                  validator, poi_contribution, pos_score);
                    }
                }
            }
            
            log::info!("DCF: Generated {} validator proposals based on combined PoS/PoI scores", proposals.len());
            proposals
        }

        /// Get validator profile information with fresh PoS and PoI scores.
        /// Returns: (combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks)
        pub fn get_validator_profile(account_id: T::AccountId) -> Option<(u64, u64, u64, u32, u32, u32, u32)> {
            ValidatorStates::<T>::get(&account_id).map(|state| {
                // Fetch fresh PoS (stake) score from the PoS pallet
                let stake = pos::Pallet::<T>::stake(&account_id);
                let pos_score = stake.saturated_into::<u64>();
                
                // Fetch fresh PoI score from the PoI pallet
                let poi_score = if let Some((result, _)) = poi::Pallet::<T>::inference_results(&account_id) {
                    result as u64
                } else {
                    // Fallback to stored inference score if no fresh result available
                    state.current.inference_score
                };
                
                // Get current weights for score calculation
                let pos_weight = if !PosWeight::<T>::exists() {
                    T::DefaultPosWeight::get()
                } else {
                    PosWeight::<T>::get()
                };
                let poi_weight = if !PoiWeight::<T>::exists() {
                    T::DefaultPoiWeight::get()
                } else {
                    PoiWeight::<T>::get()
                };
                
                // Calculate combined score using configured weights
                let mut combined_score = (pos_score.saturating_mul(pos_weight) + poi_score.saturating_mul(poi_weight)) / T::PercentagePrecision::get() as u64;
                
                // Cap the combined score at maximum allowed
                if combined_score > T::MaxValidatorScore::get() {
                    combined_score = T::MaxValidatorScore::get();
                }
                
                // Get additional profile information
                let uptime = Self::validator_uptime(&account_id);
                let inference_count = Self::validator_inference_count(&account_id);
                
                (
                    combined_score,      // Fresh calculated combined score
                    pos_score,          // Fresh PoS (stake) score
                    poi_score,          // Fresh PoI score
                    uptime,             // Validator uptime
                    inference_count,    // Number of inferences
                    state.participation_rate, // Participation rate
                    state.current.missed_blocks, // Missed blocks count
                )
            })
        }

        /// Get validator name
        pub fn get_validator_name(account_id: &T::AccountId) -> Option<Vec<u8>> {
            Self::validator_names(account_id).map(|name| name.into_inner())
        }

        /// Get the last finalized block number.
        pub fn get_last_finalized_block() -> u32 {
            Self::last_finalized_block()
        }

        /// Check if a block is finalized.
        pub fn is_block_finalized(block_number: u32) -> bool {
            block_number <= Self::last_finalized_block()
        }

        /// Get finality information including last finalized block and current epoch.
        pub fn get_finality_info() -> (u32, u32) {
            (Self::last_finalized_block(), Self::current_epoch())
        }

        /// Get the number of blocks since last finalization.
        pub fn blocks_since_finalization(current_block: u32) -> u32 {
            current_block.saturating_sub(Self::last_finalized_block())
        }

        /// Get the inference result for a validator.
        pub fn get_inference_result(account_id: T::AccountId) -> Option<u64> {
            ValidatorStates::<T>::get(&account_id).map(|state| state.current.inference_score)
        }

        /// Execute slashing action on a validator
        fn execute_slash_validator(validator: &T::AccountId, amount: <T as pallet::Config>::Balance) -> DispatchResult {
            // Check if validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(validator),
                Error::<T>::ValidatorNotFound
            );

            // 1. Calculate slash amount based on percentage or fixed amount
            let slash_amount = if amount == <T as pallet::Config>::Balance::default() {
                // Use percentage-based slashing if no specific amount provided
                let validator_balance = T::Currency::free_balance(validator);
                let slash_percent = T::SlashPercent::get();
                validator_balance * <T as pallet::Config>::Balance::from(slash_percent) / <T as pallet::Config>::Balance::from(100u32)
            } else {
                amount
            };

            // 2. Slash tokens from validator's balance
            let (_negative_imbalance, _) = T::Currency::slash(validator, slash_amount);
            let slashed_amount = slash_amount; // Use the intended slash amount for scoring

            // 3. Reduce validator's DCF score based on slash amount
            let score_penalty = (slashed_amount.saturated_into::<u64>() / T::SlashPenaltyDivisor::get()).min(T::MaxSlashPenalty::get());
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let old_score = state.current.final_score;
                state.current.final_score = state.current.final_score.saturating_sub(score_penalty);
                
                // Update last active epoch
                state.last_active_epoch = Self::current_epoch();
                
                // Add to history
                if state.history.len() == state.history.capacity() {
                    state.history.remove(0);
                }
                let _ = state.history.try_push(EpochStats {
                    epoch: Self::current_epoch(),
                    stake_score: state.current.stake_score,
                    inference_score: state.current.inference_score,
                    final_score: state.current.final_score,
                    authored_blocks: state.current.authored_blocks,
                    missed_blocks: state.current.missed_blocks,
                });
                
                log::info!("Validator {:?} slashed: amount {:?}, score {} -> {}, penalty: {}", 
                          validator, slashed_amount, old_score, state.current.final_score, score_penalty);
                
                Ok::<(), Error<T>>(())
            })?;
            
            // 4. Check if validator should be ejected due to low score
            let current_score = ValidatorStates::<T>::get(validator)
                .map(|s| s.current.final_score)
                .unwrap_or(0);
                
            if current_score < <T as pallet::Config>::MinValidatorScore::get() as u64 {
                let _ = Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold);
                log::info!("Validator {:?} ejected due to low score after slashing", validator);
            }
            
            // 5. Emit events
            Self::deposit_event(Event::ValidatorSlashed {
                validator: validator.clone(),
                amount: slashed_amount,
            });
            
            Self::deposit_event(Event::ValidatorScoreUpdated {
                validator: validator.clone(),
                stake_score: ValidatorStates::<T>::get(validator).map(|s| s.current.stake_score).unwrap_or(0),
                inference_score: ValidatorStates::<T>::get(validator).map(|s| s.current.inference_score).unwrap_or(0),
                final_score: current_score,
            });
            
            Ok(())
        }

        /// Distribute rewards to validators based on performance tiers
        fn distribute_rewards(
            base_reward_pool: <T as pallet::Config>::Balance,
            performance_reward_pool: <T as pallet::Config>::Balance,
            top_performer_reward_pool: <T as pallet::Config>::Balance,
        ) -> DispatchResult {
            let active_validators = Self::active_validators();
            if active_validators.is_empty() {
                return Ok(());
            }

            // Get validator scores and categorize them
            let mut validator_scores: Vec<(T::AccountId, u64)> = Vec::new();
            for validator in &active_validators {
                if let Some(state) = ValidatorStates::<T>::get(validator) {
                    validator_scores.push((validator.clone(), state.current.final_score));
                }
            }

            // Sort by score (descending)
            validator_scores.sort_by(|a, b| b.1.cmp(&a.1));

            let total_validators = validator_scores.len();
            let high_performance_threshold = T::HighPerformanceScore::get();
            let top_performer_count = (total_validators * T::TopPerformerPercentage::get() as usize) / 100;

            // Distribute base rewards to all active validators
            let base_reward_per_validator = if total_validators > 0 {
                base_reward_pool / (total_validators as u32).into()
            } else {
                <T as pallet::Config>::Balance::default()
            };

            // Count high performers and top performers
            let high_performers: Vec<_> = validator_scores.iter()
                .filter(|(_, score)| *score >= high_performance_threshold)
                .collect();
            
            let top_performers = &validator_scores[..top_performer_count.min(total_validators)];

            // Distribute performance rewards
            let performance_reward_per_validator = if !high_performers.is_empty() {
                performance_reward_pool / (high_performers.len() as u32).into()
            } else {
                <T as pallet::Config>::Balance::default()
            };

            // Distribute top performer rewards
            let top_performer_reward_per_validator = if !top_performers.is_empty() {
                top_performer_reward_pool / (top_performers.len() as u32).into()
            } else {
                <T as pallet::Config>::Balance::default()
            };

            // Execute reward distribution
            for (validator, score) in &validator_scores {
                let mut total_reward = base_reward_per_validator;

                // Add performance bonus
                if *score >= high_performance_threshold {
                    total_reward = total_reward.saturating_add(performance_reward_per_validator);
                }

                // Add top performer bonus
                if top_performers.iter().any(|(v, _)| v == validator) {
                    total_reward = total_reward.saturating_add(top_performer_reward_per_validator);
                }

                // Distribute the reward
                Self::execute_reward_validator(validator, total_reward)?;
            }

            log::info!("Distributed rewards: {} validators, base pool: {:?}, performance pool: {:?}, top performer pool: {:?}",
                      total_validators, base_reward_pool, performance_reward_pool, top_performer_reward_pool);

            Ok(())
        }

        /// Execute reward action on a validator
        fn execute_reward_validator(validator: &T::AccountId, amount: <T as pallet::Config>::Balance) -> DispatchResult {
            // Check if validator exists
            ensure!(
                ValidatorStates::<T>::contains_key(validator),
                Error::<T>::ValidatorNotFound
            );

            // 1. Reward tokens to validator's balance
            let reward_amount = if amount > <T as pallet::Config>::Balance::default() {
                // Use the specified amount
                amount
            } else {
                // Use the configured default reward amount
                T::ValidatorReward::get()
            };

            // Issue the reward (mint new tokens to the validator)
            let _ = T::Currency::deposit_creating(validator, reward_amount);

            // 2. Boost validator's DCF score based on reward amount
            let score_boost = (reward_amount.saturated_into::<u64>() / T::RewardBoostDivisor::get()).min(T::MaxRewardBoost::get());
            
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                let old_score = state.current.final_score;
                state.current.final_score = state.current.final_score.saturating_add(score_boost);
                
                // Cap at maximum score
                if state.current.final_score > <T as pallet::Config>::MaxValidatorScore::get() {
                    state.current.final_score = <T as pallet::Config>::MaxValidatorScore::get();
                }
                
                // Update last active epoch
                state.last_active_epoch = Self::current_epoch();
                
                // Add to history
                if state.history.len() == state.history.capacity() {
                    state.history.remove(0);
                }
                let _ = state.history.try_push(EpochStats {
                    epoch: Self::current_epoch(),
                    stake_score: state.current.stake_score,
                    inference_score: state.current.inference_score,
                    final_score: state.current.final_score,
                    authored_blocks: state.current.authored_blocks,
                    missed_blocks: state.current.missed_blocks,
                });
                
                log::debug!("Validator {:?} rewarded: amount {:?}, score {} -> {}, boost: {}", 
                           validator, reward_amount, old_score, state.current.final_score, score_boost);
                
                Ok::<(), Error<T>>(())
            })?;
            
            // 3. Emit events
            Self::deposit_event(Event::ValidatorRewarded {
                validator: validator.clone(),
                amount: reward_amount,
            });

            let current_score = ValidatorStates::<T>::get(validator)
                .map(|s| s.current.final_score)
                .unwrap_or(0);
                
            Self::deposit_event(Event::ValidatorScoreBoosted {
                validator: validator.clone(),
                old_score: current_score.saturating_sub(score_boost),
                new_score: current_score,
                reason: ScoreBoostReason::ManualBoost,
            });
            
            Ok(())
        }

        /// Execute ejection action on a validator
        fn execute_eject_validator(validator: &T::AccountId, reason: EjectionReason) -> DispatchResult {
            // 1. Unreserve the minimum stake that was locked when joining
            let min_stake = <T as Config>::MinStake::get();
            T::Currency::unreserve(validator, min_stake);

            // Remove the stake record since it's no longer reserved
            ValidatorStake::<T>::remove(validator);

            // Emit stake unreservation event
            Self::deposit_event(Event::ValidatorStakeUnreserved {
                validator: validator.clone(),
                amount: min_stake,
            });

            // 2. Remove from active validator set
            let mut active_validators = ActiveValidators::<T>::get();
            let was_active = if let Some(pos) = active_validators.iter().position(|v| v == validator) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
                true
            } else {
                false
            };
            
            // 4. Update validator state to reflect ejection
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                if let Some(state) = maybe_state.as_mut() {
                    let old_score = state.current.final_score;
                    state.current.final_score = 0; // Set score to 0 upon ejection
                    state.last_active_epoch = Self::current_epoch();
                    
                    // Add ejection to history
                    if state.history.len() == state.history.capacity() {
                        state.history.remove(0);
                    }
                    let _ = state.history.try_push(EpochStats {
                        epoch: Self::current_epoch(),
                        stake_score: state.current.stake_score,
                        inference_score: state.current.inference_score,
                        final_score: 0, // Ejected validators have 0 score
                        authored_blocks: state.current.authored_blocks,
                        missed_blocks: state.current.missed_blocks,
                    });
                    
                    log::info!("Ejected validator {:?}: score {} -> 0, reason: {:?}", 
                              validator, old_score, reason);
                }
                Ok::<(), Error<T>>(())
            })?;
            
            // 5. Remove any pending validator actions
            PendingValidatorActions::<T>::remove(validator);
            
            // 6. Emit ejection event
            Self::deposit_event(Event::ValidatorEjected {
                validator: validator.clone(),
                reason: reason.clone(),
            });
            
            // 7. Log the ejection with details
            log::info!("Successfully executed ejection on validator {:?}, reason: {:?}, was_active: {}", 
                      validator, reason, was_active);
            
            Ok(())
        }

        /// Execute add validator action
        fn execute_add_validator(validator: &T::AccountId) -> DispatchResult {
            // Check if validator already exists in the validator set
            let mut validator_set = ValidatorSet::<T>::get();
            ensure!(
                !validator_set.contains(validator),
                Error::<T>::ValidatorAlreadyExists
            );

            // Check that we haven't exceeded the maximum validators limit
            ensure!(
                validator_set.len() < validator_set.capacity(),
                Error::<T>::NotEnoughValidators
            );

            // Check minimum stake requirement
            let stake = pos::Pallet::<T>::stake(validator);
            ensure!(
                stake >= <T as pallet_cbc_pos::Config>::MinStake::get(),
                Error::<T>::InsufficientStake
            );

            // Add validator to the validator set
            validator_set.try_push(validator.clone())
                .map_err(|_| Error::<T>::NotEnoughValidators)?;
            ValidatorSet::<T>::put(validator_set);

            // Initialize validator state if it doesn't exist
            if !ValidatorStates::<T>::contains_key(validator) {
                let current_epoch = Self::current_epoch();
                let stake_score = stake.saturated_into::<u64>();
                
                // Get initial inference score from PoI pallet
                let inference_score = poi::Pallet::<T>::inference_results(validator)
                    .map(|(result, _)| result as u64)
                    .unwrap_or(0);

                // Calculate initial final score
                let pos_weight = Self::pos_weight();
                let poi_weight = Self::poi_weight();
                let mut final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / T::PercentagePrecision::get() as u64;
                if final_score > T::MaxValidatorScore::get() {
                    final_score = T::MaxValidatorScore::get();
                }

                let initial_stats = EpochStats {
                    epoch: current_epoch,
                    stake_score,
                    inference_score,
                    final_score,
                    authored_blocks: 0,
                    missed_blocks: 0,
                };

                let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
                
                let validator_state = ValidatorState {
                    last_active_epoch: current_epoch,
                    current: initial_stats.clone(),
                    history: BoundedVec::try_from(vec![initial_stats]).unwrap_or_default(),
                    uptime: 1,
                    inference_success_count: 0,
                    participation_rate: 0,
                    inference_count: 0, // Starting with 0 inferences
                    last_active_block: current_block, // Current block number
                    name: None, // No name set for governance-added validators
                    trust_score: 0, // Will be calculated later
                };

                ValidatorStates::<T>::insert(validator, validator_state);

                // Set join time
                let current_time = sp_io::offchain::timestamp().unix_millis();
                ValidatorJoinTime::<T>::insert(validator, current_time);
            }

            // Emit event
            Self::deposit_event(Event::ValidatorAdded { validator: validator.clone() });

            log::info!("DCF: Validator {:?} added via governance proposal", validator);
            Ok(())
        }

        /// Execute remove validator action
        fn execute_remove_validator(validator: &T::AccountId) -> DispatchResult {
            // Check if validator exists in the validator set
            let mut validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(validator),
                Error::<T>::ValidatorNotInSet
            );

            // Unreserve the minimum stake that was locked when joining
            let min_stake = <T as Config>::MinStake::get();
            T::Currency::unreserve(validator, min_stake);

            // Emit stake unreservation event
            Self::deposit_event(Event::ValidatorStakeUnreserved {
                validator: validator.clone(),
                amount: min_stake,
            });

            // Remove from validator set
            if let Some(pos) = validator_set.iter().position(|v| v == validator) {
                validator_set.remove(pos);
                ValidatorSet::<T>::put(validator_set);
            }

            // Remove from active validators if present
            let mut active_validators = ActiveValidators::<T>::get();
            if let Some(pos) = active_validators.iter().position(|v| v == validator) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
            }

            // Clean up validator state and related data
            ValidatorStates::<T>::remove(validator);
            ValidatorNames::<T>::remove(validator);
            ValidatorMetadata::<T>::remove(validator);
            ValidatorPerformanceHistory::<T>::remove(validator);
            ValidatorLastSeen::<T>::remove(validator);
            ValidatorBlocksAuthored::<T>::remove(validator);
            ValidatorBlocksMissed::<T>::remove(validator);
            ValidatorJoinTime::<T>::remove(validator);
            ValidatorUptime::<T>::remove(validator);
            ValidatorInferenceCount::<T>::remove(validator);
            ValidatorStake::<T>::remove(validator);

            // Remove any pending validator actions
            PendingValidatorActions::<T>::remove(validator);

            // Remove any leave requests
            ValidatorLeaveRequests::<T>::remove(validator);

            // Emit event
            Self::deposit_event(Event::ValidatorRemoved { validator: validator.clone() });

            log::info!("DCF: Validator {:?} removed via governance proposal", validator);
            Ok(())
        }

        /// Sort validators by their final weighted score (highest first).
        fn sort_validators_by_score(validators: &mut BoundedVec<T::AccountId, <T as Config>::MaxValidators>) {
            validators.sort_by(|a, b| {
                let score_a = ValidatorStates::<T>::get(a)
                    .map(|state| state.current.final_score)
                    .unwrap_or(0);
                let score_b = ValidatorStates::<T>::get(b)
                    .map(|state| state.current.final_score)
                    .unwrap_or(0);
                // Sort in descending order (highest score first)
                score_b.cmp(&score_a)
            });
        }

        /// Apply pending join/leave actions at epoch transition.
        fn apply_pending_validator_actions() {
            let mut active = ActiveValidators::<T>::get();
            let mut changed = false;

            // Collect all actions to avoid double borrow
            let actions: Vec<(T::AccountId, ValidatorAction)> =
                PendingValidatorActions::<T>::iter().collect();

            // Separate join and leave actions
            let mut join_requests: Vec<T::AccountId> = Vec::new();
            let mut leave_requests: Vec<T::AccountId> = Vec::new();

            for (who, action) in actions {
                match action {
                    ValidatorAction::Join => {
                        if !active.contains(&who) {
                            join_requests.push(who.clone());
                        }
                    }
                    ValidatorAction::Leave => {
                        if active.contains(&who) {
                            leave_requests.push(who.clone());
                        }
                    }
                }
                PendingValidatorActions::<T>::remove(&who);
            }

            // Process leave requests first
            for who in leave_requests {
                if let Some(pos) = active.iter().position(|v| v == &who) {
                    active.remove(pos);
                    changed = true;
                    log::debug!("DCF: Validator {:?} left the active set", who);
                }
            }

            // Process join requests based on scores and available capacity
            if !join_requests.is_empty() {
                // Sort join requests by their scores (highest first)
                join_requests.sort_by(|a, b| {
                    let score_a = ValidatorStates::<T>::get(a)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    let score_b = ValidatorStates::<T>::get(b)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    score_b.cmp(&score_a) // Descending order
                });

                let available_slots = active.capacity() - active.len();
                let min_score_threshold = <T as pallet::Config>::MinValidatorScore::get() as u64;

                for who in join_requests.iter().take(available_slots) {
                    let score = ValidatorStates::<T>::get(who)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    
                    // Only add validators that meet the minimum score threshold
                    if score >= min_score_threshold {
                        if active.try_push(who.clone()).is_ok() {
                            changed = true;
                            log::debug!("DCF: Validator {:?} joined the active set (score: {})", who, score);
                        }
                    } else {
                        log::warn!("DCF: Rejected join request for validator {:?} due to low score: {}", who, score);
                    }
                }

                // Log any rejected join requests due to capacity
                if join_requests.len() > available_slots {
                    let rejected_count = join_requests.len() - available_slots;
                    log::info!("DCF: Rejected {} join requests due to capacity constraints", rejected_count);
                }
            }

            if changed {
                // Sort active validators by their final weighted score (highest first)
                Self::sort_validators_by_score(&mut active);
                ActiveValidators::<T>::put(active);
            }
        }

        /// Get the epoch history for a given epoch number (for runtime API).
        pub fn get_epoch_history_api(epoch_number: u32) -> Option<RuntimeEpochHistory<T::AccountId>> {
            let histories = Self::epoch_histories();
            histories.iter().find(|h| h.epoch_number == epoch_number).cloned().map(|h| h.into())
        }

        /// Get the most recent n epoch histories (for runtime API).
        pub fn get_recent_epochs_api(n: u32) -> Vec<RuntimeEpochHistory<T::AccountId>> {
            let histories = Self::epoch_histories();
            let len = histories.len().min(n as usize);
            histories.iter().rev().take(len).cloned().map(|h| h.into()).collect::<Vec<_>>().into_iter().rev().collect()
        }

        /// Get governance mode status (for runtime API).
        pub fn get_governance_mode() -> bool {
            Self::governance_mode_enabled()
        }

        /// Get validators sorted by their final weighted score (highest first).
        pub fn get_validators_by_score() -> Vec<(T::AccountId, u64)> {
            let mut validators_with_scores: Vec<(T::AccountId, u64)> = Self::active_validators()
                .iter()
                .map(|validator| {
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    (validator.clone(), score)
                })
                .collect();
            
            // Sort by score (highest first)
            validators_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
            validators_with_scores
        }

        /// Get proposal details by ID (for runtime API).
        pub fn get_proposal_details(proposal_id: u32) -> Option<GovernanceProposal<T>> {
            Proposals::<T>::get(proposal_id)
        }

        /// Get all active proposals (for runtime API).
        pub fn get_active_proposals() -> Vec<(u32, GovernanceProposal<T>)> {
            Proposals::<T>::iter()
                .filter(|(_, proposal)| proposal.status == ProposalStatus::Pending)
                .collect()
        }

        /// Get validator's current consensus weights contribution (for runtime API).
        pub fn get_validator_consensus_contribution(validator: &T::AccountId) -> Option<(u64, u64, u64)> {
            ValidatorStates::<T>::get(validator).map(|state| {
                let pos_weight = Self::pos_weight();
                let poi_weight = Self::poi_weight();
                let pos_contribution = (state.current.stake_score * pos_weight) / T::PercentagePrecision::get() as u64;
                let poi_contribution = (state.current.inference_score * poi_weight) / T::PercentagePrecision::get() as u64;
                (pos_contribution, poi_contribution, state.current.final_score)
            })
        }

        /// Get epoch configuration (for runtime API).
        pub fn get_epoch_config() -> EpochConfig {
            Self::epoch_config()
        }

        /// Get validator statistics for a specific epoch (for runtime API).
        pub fn get_validator_epoch_stats(validator: &T::AccountId, epoch: u32) -> Option<EpochStats> {
            ValidatorStates::<T>::get(validator).and_then(|state| {
                if state.current.epoch == epoch {
                    Some(state.current.clone())
                } else {
                    state.history.iter().find(|stats| stats.epoch == epoch).cloned()
                }
            })
        }

        /// Get total number of validators in the system (for runtime API).
        pub fn get_total_validators_count() -> u32 {
            ValidatorSet::<T>::get().len() as u32
        }

        /// Get validator set capacity and current usage (for runtime API).
        pub fn get_validator_set_info() -> (u32, u32, u32) {
            let current_count = ValidatorSet::<T>::get().len() as u32;
            let active_count = ActiveValidators::<T>::get().len() as u32;
            let max_validators = <T as pallet::Config>::MaxValidators::get();
            (current_count, active_count, max_validators)
        }

        /// Get the number of misbehavior reports for a validator
        pub fn get_misbehavior_report_count(validator: &T::AccountId) -> u32 {
            let mut count = 0u32;
            for (_, _) in MisbehaviorReports::<T>::iter_prefix(validator) {
                count = count.saturating_add(1);
            }
            count
        }

        /// Get all reporters who have reported a specific validator
        pub fn get_misbehavior_reporters(validator: &T::AccountId) -> Vec<T::AccountId> {
            MisbehaviorReports::<T>::iter_prefix(validator)
                .map(|(reporter, _)| reporter)
                .collect()
        }

        /// Get misbehavior evidence from a specific reporter about a validator
        pub fn get_misbehavior_evidence(
            validator: &T::AccountId,
            reporter: &T::AccountId,
        ) -> Option<Vec<u8>> {
            MisbehaviorReports::<T>::get(validator, reporter)
                .map(|evidence| evidence.into_inner())
        }

        /// Check if a validator is close to the slashing threshold
        pub fn is_validator_at_risk(validator: &T::AccountId) -> bool {
            let report_count = Self::get_misbehavior_report_count(validator);
            let threshold = T::MisbehaviorSlashThreshold::get();
            report_count >= threshold.saturating_sub(1) // At risk if one report away from threshold
        }

        /// Check for underperforming validators and take action
        fn check_and_handle_underperforming_validators() {
            let min_score = <T as pallet::Config>::MinValidatorScore::get() as u64;
            let current_epoch = Self::current_epoch();
            
            let validators_to_check: Vec<T::AccountId> = ActiveValidators::<T>::get().into_inner();
            
            for validator in validators_to_check {
                if let Some(state) = ValidatorStates::<T>::get(&validator) {
                    // Check if validator score is below threshold
                    if state.current.final_score < min_score {
                        log::warn!("DCF: Validator {:?} has low score: {}, ejecting", 
                                  validator, state.current.final_score);
                        let _ = Self::eject_validator(&validator, EjectionReason::ScoreBelowThreshold);
                    }
                    
                    // Check if validator has been inactive for too long
                    let inactive_epochs = current_epoch.saturating_sub(state.last_active_epoch);
                    if inactive_epochs > T::MaxInactiveEpochs::get() {
                        log::warn!("DCF: Validator {:?} inactive for {} epochs, ejecting", 
                                  validator, inactive_epochs);
                        let _ = Self::eject_validator(&validator, EjectionReason::ScoreBelowThreshold);
                    }
                }
            }
        }

        /// Generate automatic proposals for validator set optimization based on scores
        fn generate_automatic_validator_proposals() {
            let active_validators = ActiveValidators::<T>::get();
            let all_validators = ValidatorSet::<T>::get();
            let max_validators = <T as pallet::Config>::MaxValidators::get() as usize;
            let _min_active_validators = <T as pallet::Config>::MinActiveValidators::get() as usize;
            
            // Don't generate proposals if governance mode is disabled
            if !Self::governance_mode_enabled() {
                return;
            }
            
            // Get all validators with their scores, sorted by score (highest first)
            let mut all_validators_with_scores: Vec<(T::AccountId, u64)> = all_validators
                .iter()
                .map(|validator| {
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    (validator.clone(), score)
                })
                .collect();
            
            // Sort by score (highest first)
            all_validators_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Case 1: We have space for more validators
            if active_validators.len() < max_validators {
                Self::propose_add_high_scoring_validators(&active_validators, &all_validators_with_scores, max_validators);
            }
            // Case 2: We're at capacity, consider replacing low-scoring active validators
            else if active_validators.len() == max_validators {
                Self::propose_replace_low_scoring_validators(&active_validators, &all_validators_with_scores);
            }
            // Case 3: We have too many validators (shouldn't happen, but handle it)
            else if active_validators.len() > max_validators {
                Self::propose_remove_excess_validators(&active_validators, max_validators);
            }
        }

        /// Propose adding high-scoring validators when there's space
        fn propose_add_high_scoring_validators(
            active_validators: &BoundedVec<T::AccountId, <T as pallet::Config>::MaxValidators>,
            all_validators_with_scores: &[(T::AccountId, u64)],
            max_validators: usize,
        ) {
            let available_slots = max_validators - active_validators.len();
            let min_score_threshold = <T as pallet::Config>::MinValidatorScore::get() as u64;
            
            // Find inactive validators with high scores
            let mut candidates = Vec::new();
            for (validator, score) in all_validators_with_scores {
                if !active_validators.contains(validator) && 
                   *score >= min_score_threshold &&
                   !Self::has_pending_join_request(validator) &&
                   !Self::was_recently_ejected(validator) {
                    candidates.push((validator.clone(), *score));
                    if candidates.len() >= available_slots {
                        break;
                    }
                }
            }
            
            // Create proposals for the best candidates
            for (validator, score) in candidates {
                if let Err(e) = Self::create_automatic_join_proposal(&validator, score) {
                    log::warn!("Failed to create join proposal for validator {:?}: {:?}", validator, e);
                }
            }
        }

        /// Propose replacing low-scoring active validators with higher-scoring inactive ones
        fn propose_replace_low_scoring_validators(
            active_validators: &BoundedVec<T::AccountId, <T as pallet::Config>::MaxValidators>,
            all_validators_with_scores: &[(T::AccountId, u64)],
        ) {
            // Get active validators with their scores, sorted by score (lowest first)
            let mut active_with_scores: Vec<(T::AccountId, u64)> = active_validators
                .iter()
                .map(|validator| {
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    (validator.clone(), score)
                })
                .collect();
            
            active_with_scores.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by score (lowest first)
            
            // Find inactive validators with higher scores than the lowest active ones
            for (inactive_validator, inactive_score) in all_validators_with_scores {
                if active_validators.contains(inactive_validator) {
                    continue; // Skip active validators
                }
                
                // Check if this inactive validator has a significantly higher score than the lowest active validator
                if let Some((lowest_active_validator, lowest_active_score)) = active_with_scores.first() {
                    let score_improvement = inactive_score.saturating_sub(*lowest_active_score);
                    let percentage_threshold = *lowest_active_score / (T::FullPercentage::get() as u64 / T::ScoreImprovementPercentage::get() as u64);
                    let min_improvement_threshold = percentage_threshold.max(T::ScoreImprovementThreshold::get()); // Configurable improvement threshold
                    
                    if score_improvement >= min_improvement_threshold &&
                       *inactive_score >= <T as pallet::Config>::MinValidatorScore::get() as u64 &&
                       !Self::has_pending_join_request(inactive_validator) &&
                       !Self::was_recently_ejected(inactive_validator) {
                        
                        // Propose to eject the lowest scoring active validator
                        if let Err(e) = Self::create_automatic_eject_proposal(lowest_active_validator, EjectionReason::ScoreBelowThreshold) {
                            log::warn!("Failed to create eject proposal for validator {:?}: {:?}", lowest_active_validator, e);
                        }
                        
                        // Propose to add the higher scoring inactive validator
                        if let Err(e) = Self::create_automatic_join_proposal(inactive_validator, *inactive_score) {
                            log::warn!("Failed to create join proposal for validator {:?}: {:?}", inactive_validator, e);
                        }
                        
                        // Only propose one replacement at a time to avoid too many simultaneous changes
                        break;
                    }
                }
            }
        }

        /// Propose removing excess validators (when somehow we have more than MaxValidators)
        fn propose_remove_excess_validators(
            active_validators: &BoundedVec<T::AccountId, <T as pallet::Config>::MaxValidators>,
            max_validators: usize,
        ) {
            let excess_count = active_validators.len() - max_validators;
            
            // Get active validators sorted by score (lowest first)
            let mut active_with_scores: Vec<(T::AccountId, u64)> = active_validators
                .iter()
                .map(|validator| {
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    (validator.clone(), score)
                })
                .collect();
            
            active_with_scores.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by score (lowest first)
            
            // Propose to eject the lowest scoring validators
            for (validator, _score) in active_with_scores.iter().take(excess_count) {
                if let Err(e) = Self::create_automatic_eject_proposal(validator, EjectionReason::ExcessValidators) {
                    log::warn!("Failed to create eject proposal for excess validator {:?}: {:?}", validator, e);
                }
            }
        }

        /// Check if a validator has a pending join request
        fn has_pending_join_request(validator: &T::AccountId) -> bool {
            PendingValidatorActions::<T>::get(validator) == Some(ValidatorAction::Join)
        }

        /// Check if a validator was recently ejected or removed (to avoid immediate re-addition)
        pub fn was_recently_ejected(validator: &T::AccountId) -> bool {
            // First check if validator is in cooldown period after leaving
            if let Some(left_at_block) = RecentlyRemovedValidators::<T>::get(validator) {
                let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();
                let cooldown_period = T::LeaveCooldown::get();
                let blocks_since_left = current_block.saturating_sub(left_at_block);

                // If still in cooldown, they cannot be re-added
                if blocks_since_left < cooldown_period {
                    return true;
                }
            }

            // Check if validator was ejected in the last few epochs
            let current_epoch = Self::current_epoch();
            let grace_period = T::MaxInactiveEpochs::get(); // Don't re-add validators ejected in the last few epochs

            if let Some(state) = ValidatorStates::<T>::get(validator) {
                // If the validator has a very low score or was recently active, they might have been ejected
                let epochs_since_active = current_epoch.saturating_sub(state.last_active_epoch);
                epochs_since_active <= grace_period && state.current.final_score == 0
            } else {
                false
            }
        }

        /// Create an automatic proposal to add a validator
        fn create_automatic_join_proposal(validator: &T::AccountId, score: u64) -> DispatchResult {
            // Instead of creating a governance proposal, directly add to pending actions
            // This is more efficient and works with the existing epoch transition logic
            
            // Check if there's already a pending join action
            if PendingValidatorActions::<T>::get(validator) == Some(ValidatorAction::Join) {
                return Ok(()); // Already has a pending join request
            }
            
            // Add the validator to pending join actions
            PendingValidatorActions::<T>::insert(validator, ValidatorAction::Join);
            
            // Emit events to indicate automatic addition
            Self::deposit_event(Event::ValidatorJoined { validator: validator.clone() });
            Self::deposit_event(Event::AutomaticValidatorProposal {
                validator: validator.clone(),
                action: ValidatorAction::Join,
                score,
                reason: BoundedVec::truncate_from(b"High score automatic addition".to_vec()),
            });
            
            log::info!("DCF: Automatically scheduled validator {:?} for addition (score: {})", validator, score);
            Ok(())
        }

        /// Create an automatic proposal to eject a validator
        fn create_automatic_eject_proposal(validator: &T::AccountId, reason: EjectionReason) -> DispatchResult {
            // For automatic ejections based on score, we can directly eject
            // For governance-based ejections, we create proposals
            
            match reason {
                EjectionReason::ScoreBelowThreshold | EjectionReason::ExcessValidators => {
                    // Get the validator's score before ejection for the event
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(0);
                    
                    // Direct ejection for performance-based reasons
                    Self::eject_validator(validator, reason.clone())?;
                    
                    // Emit automatic proposal event
                    let reason_text = match reason {
                        EjectionReason::ScoreBelowThreshold => "Low score automatic ejection",
                        EjectionReason::ExcessValidators => "Excess validators automatic ejection",
                        _ => "Automatic ejection",
                    };
                    
                    Self::deposit_event(Event::AutomaticValidatorProposal {
                        validator: validator.clone(),
                        action: ValidatorAction::Leave,
                        score,
                        reason: BoundedVec::truncate_from(reason_text.as_bytes().to_vec()),
                    });
                    
                    log::info!("DCF: Automatically ejected validator {:?} (reason: {:?})", validator, reason);
                    Ok(())
                },
                _ => {
                    // Create governance proposal for manual/slashing-based ejections
                    let proposal_id = NextProposalId::<T>::get();
                    let system_account = T::AccountId::decode(&mut &[0u8; 32][..]).unwrap_or_else(|_| {
                        // Fallback: use the first validator as proposer
                        Self::active_validators().get(0).cloned().unwrap_or_else(|| {
                            // Ultimate fallback: decode from a known pattern
                            T::AccountId::decode(&mut &[1u8; 32][..]).unwrap()
                        })
                    });
                    
                    let action = ProposalAction::Eject { 
                        validator: validator.clone(), 
                        reason: reason.clone()
                    };
                    
                    let proposal = GovernanceProposal {
                        proposer: system_account,
                        action,
                        status: ProposalStatus::Pending,
                        votes_for: 0,
                        votes_against: 0,
                    };
                    
                    Proposals::<T>::insert(proposal_id, &proposal);
                    NextProposalId::<T>::put(proposal_id + 1);
                    
                    Self::deposit_event(Event::ProposalSubmitted {
                        proposal_id,
                        proposer: proposal.proposer,
                        action: proposal.action,
                    });
                    
                    log::info!("DCF: Created governance eject proposal for validator {:?} (reason: {:?})", validator, reason);
                    Ok(())
                }
            }
        }

        /// Clean up expired entries from RecentlyRemovedValidators storage
        fn cleanup_recently_removed_validators(current_block: u32) {
            let cooldown_period = T::LeaveCooldown::get();
            let mut expired_entries = Vec::new();

            // Collect expired recently removed validators
            for (validator, removed_at_block) in RecentlyRemovedValidators::<T>::iter() {
                let blocks_passed = current_block.saturating_sub(removed_at_block);
                if blocks_passed >= cooldown_period {
                    expired_entries.push(validator);
                }
            }

            // Remove expired entries
            for validator in expired_entries {
                RecentlyRemovedValidators::<T>::remove(&validator);
                log::debug!("DCF: Removed validator {:?} from recently removed list after cooldown", validator);
            }
        }

        /// Process expired leave requests and remove validators after cooldown
        fn process_expired_leave_requests(current_block: u32) {
            let cooldown_period = T::LeaveCooldown::get();
            let mut expired_requests = Vec::new();

            // Collect expired leave requests
            for (validator, request_block) in ValidatorLeaveRequests::<T>::iter() {
                let blocks_passed = current_block.saturating_sub(request_block);
                if blocks_passed >= cooldown_period {
                    expired_requests.push(validator);
                }
            }

            // Process expired requests
            for validator in expired_requests {
                // Unreserve the validator's stake now that cooldown has expired
                let stake_amount = ValidatorStake::<T>::get(&validator);
                if stake_amount > <T as Config>::Balance::default() {
                    T::Currency::unreserve(&validator, stake_amount);
                    
                    // Emit stake unreservation event
                    Self::deposit_event(Event::ValidatorStakeUnreserved {
                        validator: validator.clone(),
                        amount: stake_amount,
                    });
                }

                // Remove from validator set
                let mut validator_set = ValidatorSet::<T>::get();
                if let Some(pos) = validator_set.iter().position(|v| v == &validator) {
                    validator_set.remove(pos);
                    ValidatorSet::<T>::put(validator_set);
                }

                // Remove from active validators if present (should already be removed)
                let mut active_validators = ActiveValidators::<T>::get();
                if let Some(pos) = active_validators.iter().position(|v| v == &validator) {
                    active_validators.remove(pos);
                    ActiveValidators::<T>::put(active_validators);
                }

                // Clean up validator state and related data
                ValidatorStates::<T>::remove(&validator);
                ValidatorNames::<T>::remove(&validator);
                ValidatorMetadata::<T>::remove(&validator);
                ValidatorPerformanceHistory::<T>::remove(&validator);
                ValidatorLastSeen::<T>::remove(&validator);
                ValidatorBlocksAuthored::<T>::remove(&validator);
                ValidatorBlocksMissed::<T>::remove(&validator);
                ValidatorJoinTime::<T>::remove(&validator);
                ValidatorUptime::<T>::remove(&validator);
                ValidatorInferenceCount::<T>::remove(&validator);
                ValidatorStake::<T>::remove(&validator);

                // Remove the leave request
                ValidatorLeaveRequests::<T>::remove(&validator);

                // Add to recently removed validators to prevent immediate re-entry
                RecentlyRemovedValidators::<T>::insert(&validator, current_block);

                // Remove any pending validator actions
                PendingValidatorActions::<T>::remove(&validator);

                // Emit event
                Self::deposit_event(Event::ValidatorLeft { validator: validator.clone() });

                log::info!("DCF: Validator {:?} left after cooldown period expired", validator);
            }
        }

        /// Emit DCF health metrics for monitoring
        fn emit_dcf_health_metrics(block_number: u32) {
            let total_validators = Self::validator_set().len();
            let active_validators = Self::active_validators().len();
            let current_epoch = Self::current_epoch();
            let governance_mode = Self::governance_mode_enabled();
            
            // Calculate average validator score
            let validator_scores: Vec<u64> = ValidatorSet::<T>::get()
                .iter()
                .filter_map(|v| ValidatorStates::<T>::get(v).map(|s| s.current.final_score))
                .collect();
            
            let avg_score = if !validator_scores.is_empty() {
                validator_scores.iter().sum::<u64>() / validator_scores.len() as u64
            } else {
                0
            };
            
            // Only log health metrics at configured intervals to reduce verbosity
            if block_number % T::DetailedLoggingInterval::get() == 0 {
                log::info!("DCF Health: epoch={}, validators={}/{}, avg_score={}", 
                          current_epoch, active_validators, total_validators, avg_score);
            } else {
                log::debug!("DCF Health Metrics at block {}: epoch={}, total_validators={}, active_validators={}, avg_score={}, governance_mode={}", 
                           block_number, current_epoch, total_validators, active_validators, avg_score, governance_mode);
            }
            
            // Emit telemetry metrics (reduced frequency)
            if block_number % T::ScoreRefreshInterval::get() == 0 {
                log::debug!("[cerulea::dcf][prometheus] dcf_health_check{{block={}}} 1", block_number);
                log::debug!("[cerulea::dcf][prometheus] total_validators{{}} {}", total_validators);
                log::debug!("[cerulea::dcf][prometheus] active_validators{{}} {}", active_validators);
                log::debug!("[cerulea::dcf][prometheus] average_validator_score{{}} {}", avg_score);
                log::debug!("[cerulea::dcf][prometheus] current_epoch{{}} {}", current_epoch);
            }
        }

        /// Run comprehensive score aggregation for all validators at epoch boundary.
        fn run_comprehensive_score_aggregation() -> Weight {
            let mut weight = Weight::zero();
            let validators = ValidatorSet::<T>::get();
            let mut updated_count = 0u32;
            
            for validator in validators.iter() {
                // Update PoS score from stake
                let stake = pos::Pallet::<T>::stake(validator);
                let stake_score = stake.saturated_into::<u64>();
                
                // Update PoI score from inference results
                let inference_score = poi::Pallet::<T>::inference_results(validator)
                    .map(|(result, _)| result as u64)
                    .unwrap_or(0);
                
                // Update validator state with fresh scores
                if let Ok(()) = ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                    let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                    state.current.stake_score = stake_score;
                    state.current.inference_score = inference_score;
                    Ok::<(), Error<T>>(())
                }).map_err(|e| sp_runtime::DispatchError::from(e)) {
                    // Recalculate final score
                    let _ = Self::update_final_score(validator);
                    _=updated_count += 1;
                    weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
                }
            }
            
            log::info!("DCF: Score aggregation completed - updated {} validators", updated_count);
            weight
        }

        /// Generate epoch-specific proposals based on comprehensive analysis.
        fn generate_epoch_proposals() -> Weight {
            let mut weight = Weight::zero();
            
            // Generate validator management proposals
            Self::generate_automatic_validator_proposals();
            weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            
            // Generate performance-based proposals
            Self::generate_performance_proposals();
            weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            
            // Generate governance proposals if needed
            Self::generate_governance_proposals();
            weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            
            log::info!("DCF: Epoch proposal generation completed");
            weight
        }

        /// Process epoch proposals - execute immediately or queue them.
        fn process_epoch_proposals() -> Weight {
            let mut weight = Weight::zero();
            let mut processed_count = 0u32;
            
            // Get all pending proposals
            let pending_proposals: Vec<_> = Proposals::<T>::iter()
                .filter(|(_, proposal)| proposal.status == ProposalStatus::Pending)
                .collect();
            
            for (proposal_id, proposal) in pending_proposals {
                // For epoch boundary, we can auto-approve certain types of proposals
                if Self::should_auto_approve_proposal(&proposal) {
                    // Auto-approve and execute
                    Proposals::<T>::try_mutate(proposal_id, |maybe_prop| {
                        if let Some(prop) = maybe_prop {
                            prop.status = ProposalStatus::Approved;
                            
                            // Execute the proposal immediately
                            match &prop.action {
                                ProposalAction::AddValidator { validator } => {
                                    let _ = Self::execute_add_validator(validator);
                                }
                                ProposalAction::RemoveValidator { validator } => {
                                    let _ = Self::execute_remove_validator(validator);
                                }
                                ProposalAction::Eject { validator, reason } => {
                                    let _ = Self::execute_eject_validator(validator, reason.clone());
                                }
                                _ => {
                                    // Other proposals require manual approval
                                }
                            }
                            
                            prop.status = ProposalStatus::Executed;
                            processed_count += 1;
                        }
                        Ok::<(), Error<T>>(())
                    }).ok();
                }
                
                weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            }
            
            log::info!("DCF: Processed {} epoch proposals", processed_count);
            weight
        }

        /// Finalize the previous epoch's best block.
        fn finalize_previous_epoch(block_number: u32) -> Weight {
            let weight = <T as Config>::WeightInfo::on_initialize();
            
            let current_epoch = Self::current_epoch();
            let previous_epoch = current_epoch.saturating_sub(1);
            
            if previous_epoch > 0 {
                // Calculate the best block of the previous epoch
                // Since we're at an epoch boundary, the best block of the previous epoch
                // is the block just before the current epoch started
                let _epoch_length = T::EpochLength::get();
                let previous_epoch_end_block = block_number.saturating_sub(1);
                
                // Update the last finalized block
                let current_finalized = Self::last_finalized_block();
                
                // Only update if this block is newer than the current finalized block
                if previous_epoch_end_block > current_finalized {
                    LastFinalizedBlock::<T>::put(previous_epoch_end_block);
                    
                    // Emit finalization event
                    Self::deposit_event(Event::BlockFinalized {
                        block_number: previous_epoch_end_block,
                    });
                    
                    log::info!("DCF: Finalized block {} (end of epoch {})", 
                              previous_epoch_end_block, previous_epoch);
                } else {
                    log::debug!("DCF: Block {} already finalized, skipping finalization of block {}", 
                               current_finalized, previous_epoch_end_block);
                }
                
                // Emit epoch ended event for the previous epoch
                Self::deposit_event(Event::EpochEnded {
                    epoch: previous_epoch,
                });

                // Emit epoch started event for the new epoch
                Self::deposit_event(Event::EpochStarted {
                    epoch: current_epoch,
                    validators: Self::active_validators().to_vec(),
                });
                
                log::info!("DCF: Epoch {} finalization completed, started epoch {}", 
                          previous_epoch, current_epoch);
            } else {
                // First epoch - initialize finalized block to genesis
                if Self::last_finalized_block() == 0 {
                    LastFinalizedBlock::<T>::put(1); // Genesis block
                    Self::deposit_event(Event::BlockFinalized {
                        block_number: 1,
                    });
                    log::info!("DCF: Initialized finalized block to genesis (block 1)");
                }
            }
            
            weight
        }

        /// Update epoch-specific metrics and perform cleanup.
        fn update_epoch_metrics(block_number: u32) -> Weight {
            let mut weight = <T as Config>::WeightInfo::on_initialize();
            
            // Update validator performance metrics
            let validators = Self::active_validators();
            for validator in validators.iter() {
                if let Some(mut state) = ValidatorStates::<T>::get(validator) {
                    // Update uptime
                    state.uptime = state.uptime.saturating_add(1);
                    
                    // Calculate participation rate
                    let total_blocks = state.current.authored_blocks + state.current.missed_blocks;
                    if total_blocks > 0 {
                        state.participation_rate = (state.current.authored_blocks * T::PercentagePrecision::get()) / total_blocks;
                    }
                    
                    ValidatorStates::<T>::insert(validator, state);
                }
                weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            }
            
            // Emit comprehensive health metrics
            Self::emit_dcf_health_metrics(block_number);
            
            log::info!("DCF: Epoch metrics updated at block {}", block_number);
            weight
        }

        /// Generate performance-based proposals.
        fn generate_performance_proposals() -> Weight {
            let weight = <T as Config>::WeightInfo::on_initialize();
            
            // This could include:
            // 1. Reward proposals for high-performing validators
            // 2. Slash proposals for underperforming validators
            // 3. Ejection proposals for consistently poor performers
            
            let validators = Self::active_validators();
            let mut high_performers = Vec::new();
            let mut low_performers = Vec::new();
            
            for validator in validators.iter() {
                if let Some(state) = ValidatorStates::<T>::get(validator) {
                    let score = state.current.final_score;
                    let _max_score = T::MaxValidatorScore::get();
                    let _min_score = <T as pallet::Config>::MinValidatorScore::get() as u64;
                    
                    // High performer: score > configured high performance threshold
                    if score > T::HighPerformanceScore::get() {
                        high_performers.push(validator.clone());
                    }
                    // Low performer: score below minimum performance threshold
                    else if score < T::MinPerformanceScore::get() {
                        low_performers.push(validator.clone());
                    }
                }
            }
            
            log::info!("DCF: Performance analysis - {} high performers, {} low performers", 
                      high_performers.len(), low_performers.len());
            
            weight
        }

        /// Generate governance-related proposals.
        fn generate_governance_proposals() -> Weight {
            let weight = <T as Config>::WeightInfo::on_initialize();
            
            // This could include:
            // 1. Parameter adjustment proposals
            // 2. Emergency governance proposals
            // 3. System upgrade proposals
            
            log::info!("DCF: Governance proposal generation completed");
            weight
        }

        /// Determine if a proposal should be auto-approved at epoch boundary.
        fn should_auto_approve_proposal(proposal: &GovernanceProposal<T>) -> bool {
            match &proposal.action {
                ProposalAction::AddValidator { .. } => {
                    // Auto-approve validator additions if they meet criteria
                    true
                }
                ProposalAction::RemoveValidator { .. } => {
                    // Auto-approve validator removals for performance reasons
                    true
                }
                ProposalAction::Eject { reason, .. } => {
                    // Auto-approve ejections for performance reasons
                    matches!(reason, EjectionReason::ScoreBelowThreshold | EjectionReason::ExcessValidators)
                }
                _ => {
                    // Manual approval required for rewards and slashing
                    false
                }
            }
        }

        /// Process block authorship validation and scoring
        fn process_block_authorship(block_number: u32) {
            if let Some(expected_author) = Self::get_expected_author(block_number) {
                // In a real implementation, you would extract the actual author from the block
                // For now, we'll use a simplified approach
                let actual_author = Self::extract_block_author();
                
                match actual_author {
                    Some(actual) if actual == expected_author => {
                        // Correct author produced the block
                        let _ = Self::record_block_authorship(&actual);
                    }
                    Some(actual) => {
                        // Wrong author produced the block
                        let _ = Self::record_missed_block(&expected_author);
                        Self::deposit_event(Event::InvalidAuthor {
                            block_number,
                            author: actual,
                        });
                    }
                    None => {
                        // No author found, record as missed block
                        let _ = Self::record_missed_block(&expected_author);
                    }
                }
            }
        }

        /// Extract block author from system digest (simplified implementation)
        fn extract_block_author() -> Option<T::AccountId> {
            // This is a simplified implementation
            // In a real scenario, you would extract the author from block headers or consensus logs
            frame_system::Pallet::<T>::digest()
                .logs()
                .iter()
                .find_map(|log| {
                    if let DigestItem::Consensus(_, data) = log {
                        T::AccountId::decode(&mut &data[..]).ok()
                    } else {
                        None
                    }
                })
        }

        /// Apply score decay to inactive validators
        fn apply_validator_score_decay(current_epoch: u32) -> Weight {
            let mut weight = Weight::zero();
            let validators = Self::validator_set();
            
            for validator in validators.iter() {
                if let Err(_) = Self::apply_score_decay(validator, current_epoch) {
                    log::warn!("Failed to apply score decay for validator {:?}", validator);
                }
                weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            }
            
            weight
        }

        /// Update participation rates for all validators
        fn update_validator_participation_rates() -> Weight {
            let mut weight = Weight::zero();
            let validators = Self::validator_set();
            
            for validator in validators.iter() {
                ValidatorStates::<T>::mutate(validator, |maybe_state| {
                    if let Some(state) = maybe_state.as_mut() {
                        let total_blocks = state.current.authored_blocks + state.current.missed_blocks;
                        if total_blocks > 0 {
                            state.participation_rate = (state.current.authored_blocks * T::PercentagePrecision::get()) / total_blocks;
                        }
                    }
                });
                weight = weight.saturating_add(<T as Config>::WeightInfo::on_initialize());
            }
            
            weight
        }

        /// Calculate trust score for a validator based on uptime, inference success, and slashing history.
        pub fn calculate_trust_score(validator: &T::AccountId) -> u64 {
            let state = match ValidatorStates::<T>::get(validator) {
                Some(state) => state,
                None => return 0,
            };

            let current_epoch = CurrentEpoch::<T>::get();

            // Calculate uptime component (percentage of epochs active)
            let uptime_percentage = if current_epoch > 0 {
                (state.uptime * T::PercentagePrecision::get()) / current_epoch
            } else {
                T::PercentagePrecision::get() // 100% for genesis
            };

            // Calculate inference success rate
            let inference_success_rate = if state.inference_count > 0 {
                (state.inference_success_count as u64 * T::PercentagePrecision::get() as u64) / state.inference_count
            } else {
                T::PercentagePrecision::get() as u64 // 100% if no inferences yet
            };

            // Calculate slashing penalty (lower is better)
            let slashing_penalty = 0u64; // TODO: Implement slashing history tracking

            // Weighted trust score calculation
            let uptime_component = (uptime_percentage as u64 * T::TrustScoreUptimeWeight::get()) / T::PercentagePrecision::get() as u64;
            let inference_component = (inference_success_rate * T::TrustScoreInferenceWeight::get()) / T::PercentagePrecision::get() as u64;
            let slashing_component = slashing_penalty * T::TrustScoreSlashingWeight::get();

            let trust_score = uptime_component
                .saturating_add(inference_component)
                .saturating_sub(slashing_component);

            // Cap at maximum trust score
            trust_score.min(T::MaxTrustScore::get())
        }

        /// Update trust score for a validator.
        pub fn update_trust_score(validator: &T::AccountId) -> DispatchResult {
            let new_trust_score = Self::calculate_trust_score(validator);

            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.trust_score = new_trust_score;
                Ok::<(), Error<T>>(())
            })?;

            Ok(())
        }


    }
}

// Re-export the pallet for external use
pub use pallet::*;

// --- Benchmarking (if enabled) --- //
// (Benchmarking module is declared at the top of the file)

// --- Score/Ejection/Inference Reason Enums --- //

/// Reason for boosting a validator's score.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum ScoreBoostReason {
    ValidBlockAuthored,
    ValidInference,
    ManualBoost,
}

/// Reason for ejecting a validator.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum EjectionReason {
    ScoreBelowThreshold,
    MaxSlashingReached,
    ManualEjection,
    ExcessValidators,
    InsufficientStake,
}

/// Severity of an inference error.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum InferenceErrorSeverity {
    High,   // Major error, significant impact
    Medium, // Moderate error
    Low,    // Minor error
}

/// Reason for slashing a validator.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum SlashReason {
    Misbehavior,
    PoorPerformance,
    ManualSlash,
    ConsensusViolation,
}

/// Data structure for inference data collected off-chain
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct InferenceData<T: Config> {
    pub validator: <T as frame_system::Config>::AccountId,
    pub epoch: u32,
    pub inference_result: u32,
    pub confidence_score: u32,
    pub timestamp: u64,
    pub data_sources: Vec<Vec<u8>>,
}

/// Off-chain storage structure for PoI scores
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OffchainPoiScore<T: Config> {
    pub validator: <T as frame_system::Config>::AccountId,
    pub score: u64,
    pub block_number: u32,
    pub timestamp: u64,
}

// --- DcfInterface Implementation --- //
impl<T: Config> pallet_cbc_poi::DcfInterface<T::AccountId> for Pallet<T> {
    fn record_inference_activity(validator: &T::AccountId) -> DispatchResult {
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u32>();
        
        ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
            let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
            
            // Increment inference count
            state.inference_count = state.inference_count.saturating_add(1);
            
            // Update last active block
            state.last_active_block = current_block;
            
            // Update last active epoch
            state.last_active_epoch = Self::current_epoch();
            
            Ok::<(), Error<T>>(())
        })?;
        
        // Also update the separate ValidatorInferenceCount storage for compatibility
        ValidatorInferenceCount::<T>::mutate(validator, |count| {
            *count = count.saturating_add(1);
        });
        
        Ok(())
    }
}

// --- Tests Module --- //
#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod mock;