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
//! - **19. `leave_validators`**: Leave the validator set with cooldown period.  
//!   - Signed required.
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
    traits::{SaturatedConversion, AtLeast32BitUnsigned},
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
    pub struct ValidatorState {
        pub last_active_epoch: u32,
        pub current: EpochStats,
        pub history: BoundedVec<EpochStats, ConstU32<10>>, // Keep as constant for now
        pub uptime: u32, // Number of epochs active
        pub inference_success_count: u32,
        pub participation_rate: u32, // Percentage (0 to T::PercentagePrecision)
        pub inference_count: u64, // Total number of inferences performed
        pub last_active_block: u32, // Last block number where validator was active
        pub name: Option<BoundedVec<u8, ConstU32<32>>>, // Optional validator name
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
    pub enum ProposalAction<T: Config + TypeInfo + fmt::Debug> { // <-- Change here
        Slash { validator: T::AccountId, amount: <T as pallet::Config>::Balance },
        Reward { validator: T::AccountId, amount: <T as pallet::Config>::Balance },
        Eject { validator: T::AccountId, reason: EjectionReason },
        AddValidator { validator: T::AccountId },
        RemoveValidator { validator: T::AccountId },
    }

    /// Governance proposal structure.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct GovernanceProposal<T: Config + TypeInfo + fmt::Debug> { // <-- Change here
        pub proposer: T::AccountId,
        pub action: ProposalAction<T>,
        pub status: ProposalStatus,
        pub votes_for: u32,
        pub votes_against: u32,
    }

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

    /// Non-generic struct for runtime API (AccountId = T::AccountId, all BoundedVecs use MaxValidators).
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct RuntimeEpochHistory<AccountId> {
        pub epoch_number: u32,
        pub active_validators: BoundedVec<AccountId, ConstU32<100>>, // Using a reasonable default for runtime API
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
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        #[pallet::constant]
        type MinActiveValidators: Get<u32>;
        #[pallet::constant]
        type MaxEpochHistory: Get<u32>;
        
        // Scoring weights and thresholds
        #[pallet::constant]
        type DefaultPosWeight: Get<u64>;
        #[pallet::constant]
        type DefaultPoiWeight: Get<u64>;
        #[pallet::constant]
        type MinValidatorScore: Get<u32>;
        #[pallet::constant]
        type MaxValidatorScore: Get<u64>;
        
        // Score decay and activity parameters
        #[pallet::constant]
        type ValidatorScoreDecay: Get<u32>;
        #[pallet::constant]
        type MaxInactiveEpochs: Get<u32>;
        #[pallet::constant]
        type ScoreDecayInterval: Get<u32>; // blocks
        #[pallet::constant]
        type ParticipationUpdateInterval: Get<u32>; // blocks
        #[pallet::constant]
        type UnderperformanceCheckInterval: Get<u32>; // blocks
        #[pallet::constant]
        type ValidatorProposalInterval: Get<u32>; // blocks
        #[pallet::constant]
        type HealthMetricsInterval: Get<u32>; // blocks
        #[pallet::constant]
        type OffchainWorkerInterval: Get<u32>; // blocks
        #[pallet::constant]
        type LeaveCooldown: Get<u32>; // blocks
        #[pallet::constant]
        type EpochLength: Get<u32>; // blocks per epoch
        
        // Block authorship rewards and penalties
        #[pallet::constant]
        type BlockAuthorshipBoost: Get<u64>;
        #[pallet::constant]
        type MissedBlockPenalty: Get<u64>;
        
        // Inference scoring parameters
        #[pallet::constant]
        type InferenceBoostLow: Get<u64>;
        #[pallet::constant]
        type InferenceBoostMedium: Get<u64>;
        #[pallet::constant]
        type InferenceBoostHigh: Get<u64>;
        #[pallet::constant]
        type InferencePenaltyLow: Get<u64>;
        #[pallet::constant]
        type InferencePenaltyMedium: Get<u64>;
        #[pallet::constant]
        type InferencePenaltyHigh: Get<u64>;
        #[pallet::constant]
        type InferenceConfidenceThresholdLow: Get<u32>;
        #[pallet::constant]
        type InferenceConfidenceThresholdHigh: Get<u32>;
        
        // Governance and slashing parameters
        #[pallet::constant]
        type MaxSlashPenalty: Get<u64>;
        #[pallet::constant]
        type MaxRewardBoost: Get<u64>;
        #[pallet::constant]
        type SlashPenaltyDivisor: Get<u64>;
        #[pallet::constant]
        type RewardBoostDivisor: Get<u64>;
        #[pallet::constant]
        type SlashPercent: Get<u32>; // Percentage of stake to slash (0-100)
        
        // Validator metadata limits
        #[pallet::constant]
        type MaxValidatorNameLength: Get<u32>;
        #[pallet::constant]
        type MaxValidatorWebsiteLength: Get<u32>;
        #[pallet::constant]
        type MaxValidatorContactLength: Get<u32>;
        #[pallet::constant]
        type MaxValidatorDescriptionLength: Get<u32>;
        #[pallet::constant]
        type MaxValidatorLocationLength: Get<u32>;
        #[pallet::constant]
        type MaxPerformanceHistoryLength: Get<u32>;
        #[pallet::constant]
        type MaxValidatorHistoryLength: Get<u32>;
        #[pallet::constant]
        type MaxCommissionRate: Get<u32>; // basis points (10000 = 100%)
        
        // Percentage calculation precision
        #[pallet::constant]
        type PercentagePrecision: Get<u32>; // 10000 for basis points (0.01% precision)
        
        // Misbehavior reporting
        #[pallet::constant]
        type MaxEvidenceLength: Get<u32>;
        #[pallet::constant]
        type MisbehaviorSlashThreshold: Get<u32>; // Number of reports needed to trigger slash
        
        // Off-chain worker configuration
        #[pallet::constant]
        type OffchainWorkerTimeout: Get<u64>; // milliseconds
        #[pallet::constant]
        type EstimatedBlockTime: Get<u64>; // milliseconds
        
        // Stake and balance configuration
        #[pallet::constant]
        type MinStake: Get<<Self as Config>::Balance>;
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        type Currency: Currency<Self::AccountId, Balance = <Self as pallet::Config>::Balance> + ReservableCurrency<Self::AccountId>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Stores state for each validator.
    #[pallet::storage]
    #[pallet::getter(fn validator_states)]
    pub type ValidatorStates<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorState,
        OptionQuery,
    >;

    /// Current PoS weight for scoring.
    #[pallet::storage]
    #[pallet::getter(fn pos_weight)]
    pub type PosWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Current PoI weight for scoring.
    #[pallet::storage]
    #[pallet::getter(fn poi_weight)]
    pub type PoiWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Set of all validators.
    #[pallet::storage]
    #[pallet::getter(fn validator_set)]
    pub type ValidatorSet<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    /// Epoch configuration.
    #[pallet::storage]
    #[pallet::getter(fn epoch_config)]
    pub type EpochConfigStorage<T: Config> = StorageValue<_, EpochConfig, ValueQuery>;

    /// Current epoch number.
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Currently active validators.
    #[pallet::storage]
    #[pallet::getter(fn active_validators)]
    pub type ActiveValidators<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    /// Whether governance mode is enabled (sudo-like).
    #[pallet::storage]
    #[pallet::getter(fn governance_mode_enabled)]
    pub type GovernanceModeEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Governance proposals by ID.
    #[pallet::storage]
    pub type Proposals<T: Config> = StorageMap<
        _, Blake2_128Concat, u32, GovernanceProposal<T>, OptionQuery
    >;

    /// Next proposal ID counter.
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Track who has voted on which proposal.
    #[pallet::storage]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, bool, OptionQuery
    >;

    /// Pending validator join/leave requests, applied at next epoch.
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

    /// Stores recent epoch histories in a ring buffer.
    #[pallet::storage]
    #[pallet::getter(fn epoch_histories)]
    pub type EpochHistories<T: Config> = StorageValue<_, BoundedVec<EpochHistory<T>, T::MaxEpochHistory>, ValueQuery>;

    /// Validator names for display purposes
    #[pallet::storage]
    #[pallet::getter(fn validator_names)]
    pub type ValidatorNames<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u8, ConstU32<32>>,
        OptionQuery,
    >;

    /// Validator uptime tracking (number of epochs active)
    #[pallet::storage]
    #[pallet::getter(fn validator_uptime)]
    pub type ValidatorUptime<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Validator inference success count
    #[pallet::storage]
    #[pallet::getter(fn validator_inference_count)]
    pub type ValidatorInferenceCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Comprehensive validator metadata including contact info, website, etc.
    #[pallet::storage]
    #[pallet::getter(fn validator_metadata)]
    pub type ValidatorMetadata<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorMetadataInfo,
        OptionQuery,
    >;

    /// Validator performance metrics over time
    #[pallet::storage]
    #[pallet::getter(fn validator_performance_history)]
    pub type ValidatorPerformanceHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<PerformanceRecord, ConstU32<100>>,
        ValueQuery,
    >;

    /// Validator last seen block number (for activity tracking)
    #[pallet::storage]
    #[pallet::getter(fn validator_last_seen)]
    pub type ValidatorLastSeen<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Block number
        ValueQuery,
    >;

    /// Validator total blocks authored
    #[pallet::storage]
    #[pallet::getter(fn validator_blocks_authored)]
    pub type ValidatorBlocksAuthored<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Validator total blocks missed
    #[pallet::storage]
    #[pallet::getter(fn validator_blocks_missed)]
    pub type ValidatorBlocksMissed<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Validator join timestamp
    #[pallet::storage]
    #[pallet::getter(fn validator_join_time)]
    pub type ValidatorJoinTime<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u64, // Timestamp in milliseconds
        OptionQuery,
    >;

    /// Validator leave requests with block number when request was made
    #[pallet::storage]
    #[pallet::getter(fn validator_leave_requests)]
    pub type ValidatorLeaveRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32, // Block number when leave request was made
        OptionQuery,
    >;

    /// Last finalized block number (finalized at end of each epoch)
    #[pallet::storage]
    #[pallet::getter(fn last_finalized_block)]
    pub type LastFinalizedBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Misbehavior reports: (reported_validator, reporter) -> evidence
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
        InsufficientStake,
        ValidatorNotInSet,
        LeaveCooldownActive,
    }

    // --- Dispatchable Calls --- //
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Update a validator's stake score (PoS).
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_governance_mode(origin: OriginFor<T>, enabled: bool) -> DispatchResult {
            ensure_root(origin)?;
            GovernanceModeEnabled::<T>::put(enabled);
            Self::deposit_event(Event::GovernanceModeToggled { enabled });
            Ok(())
        }

        /// Sudo: advance epoch manually (governance mode only).
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn sudo_advance_epoch(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(GovernanceModeEnabled::<T>::get(), Error::<T>::NotAllowedInGovernanceMode);
            let _ = Self::handle_epoch_transition();
            Ok(())
        }

        /// Submit a governance proposal (slash, reward, eject).
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn vote_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotApproved)?;
                ensure!(prop.status == ProposalStatus::Pending, Error::<T>::ProposalAlreadyExecuted);
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotApproved)?;
                ensure!(prop.status == ProposalStatus::Approved, Error::<T>::ProposalNotApproved);

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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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

            // Check if validator is already in the validator set
            let mut validator_set = ValidatorSet::<T>::get();
            ensure!(
                !validator_set.contains(&who),
                Error::<T>::ValidatorAlreadyExists
            );

            // Check minimum stake requirement
            let stake = pos::Pallet::<T>::stake(&who);
            ensure!(
                stake >= <T as pallet_cbc_pos::Config>::MinStake::get(),
                Error::<T>::InsufficientStake
            );

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
                let stake_score = stake.saturated_into::<u64>();
                
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
                    name: name.clone(), // Optional validator name
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
        /// This removes the validator from the ValidatorSet after cooldown period expires.
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn leave_validators(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if validator is in the validator set
            let mut validator_set = ValidatorSet::<T>::get();
            ensure!(
                validator_set.contains(&who),
                Error::<T>::ValidatorNotInSet
            );

            let current_block = frame_system::Pallet::<T>::block_number().saturated_into::<u32>();

            // Check if there's already a pending leave request
            if let Some(request_block) = ValidatorLeaveRequests::<T>::get(&who) {
                let cooldown_period = T::LeaveCooldown::get();
                let blocks_passed = current_block.saturating_sub(request_block);
                
                // If cooldown period has passed, execute the leave
                if blocks_passed >= cooldown_period {
                    // Remove from validator set
                    if let Some(pos) = validator_set.iter().position(|v| v == &who) {
                        validator_set.remove(pos);
                        ValidatorSet::<T>::put(validator_set);
                    }

                    // Remove from active validators if present
                    let mut active_validators = ActiveValidators::<T>::get();
                    if let Some(pos) = active_validators.iter().position(|v| v == &who) {
                        active_validators.remove(pos);
                        ActiveValidators::<T>::put(active_validators);
                    }

                    // Clean up validator state and related data
                    ValidatorStates::<T>::remove(&who);
                    ValidatorNames::<T>::remove(&who);
                    ValidatorMetadata::<T>::remove(&who);
                    ValidatorPerformanceHistory::<T>::remove(&who);
                    ValidatorLastSeen::<T>::remove(&who);
                    ValidatorBlocksAuthored::<T>::remove(&who);
                    ValidatorBlocksMissed::<T>::remove(&who);
                    ValidatorJoinTime::<T>::remove(&who);
                    ValidatorUptime::<T>::remove(&who);
                    ValidatorInferenceCount::<T>::remove(&who);

                    // Remove the leave request
                    ValidatorLeaveRequests::<T>::remove(&who);

                    // Remove any pending validator actions
                    PendingValidatorActions::<T>::remove(&who);

                    // Emit event
                    Self::deposit_event(Event::ValidatorLeft { validator: who });

                    Ok(())
                } else {
                    // Cooldown period is still active
                    Err(Error::<T>::LeaveCooldownActive.into())
                }
            } else {
                // First time requesting to leave - start cooldown period
                ValidatorLeaveRequests::<T>::insert(&who, current_block);

                // Emit event to indicate leave request has been made
                Self::deposit_event(Event::ValidatorLeft { validator: who.clone() });



                Ok(())
            }
        }

        /// Request to leave the validator set (opt-out, effective next epoch).
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
        #[pallet::weight(Weight::from_parts(10_000, 0))]
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
                
                // Trigger resort if score changed by more than 10% or 1000 points
                let significant_change = score_change > (old_final_score / 10).max(1000);
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
            
            // Step 4: Handle epoch transition (existing logic)
            if !Self::governance_mode_enabled() {
                weight = weight.saturating_add(Self::handle_epoch_transition());
            }
            
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
                Self::generate_validator_proposals();
            }
            
            // 6. Process expired leave requests
            if block_number % 100 == 0 {
                Self::process_expired_leave_requests(block_number);
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
            if GovernanceModeEnabled::<T>::get() {
                return <T as Config>::WeightInfo::on_initialize();
            }
            let current_epoch = Self::current_epoch();
            let next_epoch = current_epoch.saturating_add(1);
            CurrentEpoch::<T>::put(next_epoch);

            // Apply pending join/leave requests
            Self::apply_pending_validator_actions();

            // Ensure validators are sorted by final score for the new epoch
            let mut active_validators = ActiveValidators::<T>::get();
            Self::sort_validators_by_score(&mut active_validators);
            ActiveValidators::<T>::put(active_validators.clone());
            Self::deposit_event(Event::EpochStarted {
                epoch: next_epoch,
                validators: active_validators.clone().into_inner(),
            });
            
            // Emit comprehensive epoch transition event
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
                return T::PercentagePrecision::get(); // 100% if just joined
            }
            
            let total_blocks_since_join = current_block.saturating_sub(join_time_block);
            let blocks_since_last_seen = current_block.saturating_sub(last_seen);
            
            if total_blocks_since_join == 0 {
                return T::PercentagePrecision::get(); // 100%
            }
            
            let active_blocks = total_blocks_since_join.saturating_sub(blocks_since_last_seen);
            let uptime_percentage = (active_blocks as u64 * T::PercentagePrecision::get() as u64) / total_blocks_since_join as u64;
            
            uptime_percentage.min(T::PercentagePrecision::get() as u64) as u32 // Cap at 100%
        }
    }

    // --- Genesis Configuration --- //
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub validators: Vec<T::AccountId>,
        pub validator_scores: Vec<u32>,
        pub current_epoch: u32,
        pub epoch_config: EpochConfig,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            ValidatorSet::<T>::put(
                BoundedVec::try_from(self.validators.clone())
                    .expect("Initial validators exceed MaxValidators"),
            );

            for (validator, score) in self.validators.iter().zip(self.validator_scores.iter()) {
                let pos_weight = T::DefaultPosWeight::get();
                let poi_weight = T::DefaultPoiWeight::get();
                let final_score = (*score as u64 * pos_weight + *score as u64 * poi_weight) / T::PercentagePrecision::get() as u64;
                let mut history = BoundedVec::<EpochStats, ConstU32<10>>::default();
                let _ = history.try_push(EpochStats {
                    epoch: 0,
                    stake_score: *score as u64,
                    inference_score: *score as u64,
                    final_score,
                    authored_blocks: 0,
                    missed_blocks: 0,
                });
                ValidatorStates::<T>::insert(
                    validator,
                    ValidatorState {
                        last_active_epoch: 0,
                        current: EpochStats {
                            epoch: 0,
                            stake_score: *score as u64,
                            inference_score: *score as u64,
                            final_score,
                            authored_blocks: 0,
                            missed_blocks: 0,
                        },
                        history,
                        uptime: 0,
                        inference_success_count: 0,
                        participation_rate: 0,
                        inference_count: 0, // Starting with 0 inferences
                        last_active_block: 0, // Genesis block
                        name: None, // No name set at genesis
                    },
                );
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

            // Calculate total weighted score
            let mut total_weight = 0u64;
            let validator_weights: Vec<(T::AccountId, u64)> = validators.iter()
                .map(|validator| {
                    let score = ValidatorStates::<T>::get(validator)
                        .map(|state| state.current.final_score)
                        .unwrap_or(1); // Minimum weight of 1 to ensure all validators can be selected
                    total_weight = total_weight.saturating_add(score);
                    (validator.clone(), score)
                })
                .collect();

            if total_weight == 0 {
                // Fallback to round-robin if all scores are zero
                let idx = (block_number as usize) % validators.len();
                return validators.get(idx).cloned();
            }

            // Use block number as seed for deterministic selection
            let target = (block_number as u64 * 2654435761u64) % total_weight; // Using a large prime for better distribution
            let mut cumulative_weight = 0u64;

            for (validator, weight) in validator_weights {
                cumulative_weight = cumulative_weight.saturating_add(weight);
                if target < cumulative_weight {
                    log::debug!("DCF: Selected author {:?} for block {} (score: {}, target: {}/{})", 
                               validator, block_number, weight, target, total_weight);
                    return Some(validator);
                }
            }

            // Fallback to first validator if something goes wrong
            validators.get(0).cloned()
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
                // Use a default reward amount based on validator's current balance
                let validator_balance = T::Currency::free_balance(validator);
                validator_balance / <T as pallet::Config>::Balance::from(100u32) // 1% of current balance as default reward
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
                
                log::info!("Validator {:?} rewarded: amount {:?}, score {} -> {}, boost: {}", 
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
            // 1. Remove from active validator set
            let mut active_validators = ActiveValidators::<T>::get();
            let was_active = if let Some(pos) = active_validators.iter().position(|v| v == validator) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
                true
            } else {
                false
            };
            
            // 2. Update validator state to reflect ejection
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
            
            // 3. Remove any pending validator actions
            PendingValidatorActions::<T>::remove(validator);
            
            // 4. Emit ejection event
            Self::deposit_event(Event::ValidatorEjected {
                validator: validator.clone(),
                reason: reason.clone(),
            });
            
            // 5. Log the ejection with details
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
                    log::info!("DCF: Validator {:?} left the active set", who);
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
                            log::info!("DCF: Validator {:?} joined the active set (score: {})", who, score);
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
        fn generate_validator_proposals() {
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
                    let min_improvement_threshold = (*lowest_active_score / 10).max(1000); // 10% or 1000 points minimum improvement
                    
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

        /// Check if a validator was recently ejected (to avoid immediate re-addition)
        fn was_recently_ejected(validator: &T::AccountId) -> bool {
            // Check if validator was ejected in the last few epochs
            let current_epoch = Self::current_epoch();
            let grace_period = 3; // Don't re-add validators ejected in the last 3 epochs
            
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
                // Remove from validator set
                let mut validator_set = ValidatorSet::<T>::get();
                if let Some(pos) = validator_set.iter().position(|v| v == &validator) {
                    validator_set.remove(pos);
                    ValidatorSet::<T>::put(validator_set);
                }

                // Remove from active validators if present
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

                // Remove the leave request
                ValidatorLeaveRequests::<T>::remove(&validator);

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
            
            log::info!("DCF Health Metrics at block {}: epoch={}, total_validators={}, active_validators={}, avg_score={}, governance_mode={}", 
                      block_number, current_epoch, total_validators, active_validators, avg_score, governance_mode);
            
            // Emit telemetry metrics
            log::info!("[cerulea::dcf][prometheus] dcf_health_check{{block={}}} 1", block_number);
            log::info!("[cerulea::dcf][prometheus] total_validators{{}} {}", total_validators);
            log::info!("[cerulea::dcf][prometheus] active_validators{{}} {}", active_validators);
            log::info!("[cerulea::dcf][prometheus] average_validator_score{{}} {}", avg_score);
            log::info!("[cerulea::dcf][prometheus] current_epoch{{}} {}", current_epoch);
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
            Self::generate_validator_proposals();
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
                    let max_score = T::MaxValidatorScore::get();
                    let min_score = <T as pallet::Config>::MinValidatorScore::get() as u64;
                    
                    // High performer: score > 80% of max
                    if score > (max_score * 80) / 100 {
                        high_performers.push(validator.clone());
                    }
                    // Low performer: score < 120% of min
                    else if score < (min_score * 120) / 100 {
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
}

/// Severity of an inference error.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum InferenceErrorSeverity {
    High,   // Major error, significant impact
    Medium, // Moderate error
    Low,    // Minor error
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