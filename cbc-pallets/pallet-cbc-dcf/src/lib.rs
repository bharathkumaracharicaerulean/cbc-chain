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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#[warn(unused_comparisons)]

// Unit tests module
#[cfg(test)]
pub mod tests;

// Integration tests module
#[cfg(test)]
pub mod integration_tests;

// Benchmarking module
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

// Performance optimization module
pub mod performance;

// Performance tests module
#[cfg(test)]
mod performance_tests;

// Mock module for testing
#[cfg(test)]
pub mod mock;

// --- Imports --- //
use frame_support::{
    pallet_prelude::*,
    traits::Get,
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
    }
}

// --- Weights Module --- //
pub mod weights;
pub use weights::*;

// Re-export the pallet for external use
pub use self::pallet::*;

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
        type HealthMetricsInterval: Get<u32>; // blocks
        #[pallet::constant]
        type OffchainWorkerInterval: Get<u32>; // blocks
        
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
        
        // Off-chain worker configuration
        #[pallet::constant]
        type OffchainWorkerTimeout: Get<u64>; // milliseconds
        #[pallet::constant]
        type EstimatedBlockTime: Get<u64>; // milliseconds
        
        // Stake and balance configuration
        #[pallet::constant]
        type MinStake: Get<<Self as Config>::Balance>;
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
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
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
        ValidatorsDebug(Vec<T::AccountId>),
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
            let mut updated_count = 0u32;
            
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
                            updated_count += 1;
                            
                            // Emit event
                            Self::deposit_event(Event::ValidatorPoiScoreUpdated {
                                validator: validator.clone(),
                                poi_score,
                            });
                        }
                    }
                }
            }
            
            log::info!("DCF: Applied {} PoI score updates from off-chain computation", updated_count);
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
                proposer,
                action,
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
            
            ValidatorNames::<T>::insert(&who, bounded_name);
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
            ValidatorNames::<T>::insert(&who, bounded_name.clone());
            
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
                let _old_score = state.current.final_score;
                state.current.final_score = final_score;
                state.last_active_epoch = Self::current_epoch();
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

        /// Record a missed block for a validator.
        pub fn record_missed_block(validator: &T::AccountId) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.missed_blocks = state.current.missed_blocks.saturating_add(1);
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;
            // --- Telemetry: Block author failure ---
            ::log::info!("[cerulea::dcf][prometheus] block_author_failure{{validator={:?}}} 1", validator);
            Ok(())
        }

        /// Record block authorship for a validator and boost score.
        pub fn record_block_authorship(validator: &T::AccountId) -> DispatchResult {
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.authored_blocks = state.current.authored_blocks.saturating_add(1);
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;
            // --- Telemetry: Block author success ---
            ::log::info!("[cerulea::dcf][prometheus] block_author_success{{validator={:?}}} 1", validator);
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

            let active_validators = ActiveValidators::<T>::get();
            Self::deposit_event(Event::EpochStarted {
                epoch: next_epoch,
                validators: active_validators.clone().into_inner(),
            });

            // --- Telemetry: Active validator count ---
            ::log::info!("[cerulea::dcf][prometheus] active_validators_count{{}} {}", active_validators.len());

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
            // --- Telemetry: Inference rate per epoch ---
            let inference_count = inference_summary.iter().filter(|(_, inf)| inf.is_some()).count();
            ::log::info!("[cerulea::dcf][prometheus] inference_rate_per_epoch{{epoch={}}} {}", next_epoch, inference_count);
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
            let epoch_config = Self::epoch_config();
            let current_epoch = Self::current_epoch();
            let block_number = now.saturated_into::<u32>();

            // 1. Handle automatic epoch transitions (core DCF functionality)
            if !Self::governance_mode_enabled() && Self::should_transition_epoch(now, current_epoch, &epoch_config) {
                log::info!("DCF: Epoch transition triggered at block {}, epoch {} -> {}", 
                          block_number, current_epoch, current_epoch + 1);
                
                weight = weight.saturating_add(Self::handle_epoch_transition());
                
                // Log epoch transition completion
                log::info!("DCF: Epoch transition completed. New epoch: {}, active validators: {}", 
                          Self::current_epoch(), Self::active_validators().len());
            }

            // 2. Validate block authorship and update validator metrics (every block)
            Self::process_block_authorship(block_number);

            // 3. Apply score decay for inactive validators
            if block_number % T::ScoreDecayInterval::get() == 0 {
                let decay_weight = Self::apply_validator_score_decay(current_epoch);
                weight = weight.saturating_add(decay_weight);
                
                if block_number % T::ParticipationUpdateInterval::get() == 0 {
                    log::debug!("DCF: Applied score decay at block {}", block_number);
                }
            }

            // 4. Update validator participation rates
            if block_number % T::ParticipationUpdateInterval::get() == 0 {
                let participation_weight = Self::update_validator_participation_rates();
                weight = weight.saturating_add(participation_weight);
                
                // Log validator statistics
                let total_validators = Self::validator_set().len();
                let active_validators = Self::active_validators().len();
                log::info!("DCF: Block {} - Total validators: {}, Active: {}", 
                          block_number, total_validators, active_validators);
            }

            // 5. Check for low-performing validators
            if block_number % T::UnderperformanceCheckInterval::get() == 0 {
                Self::check_and_handle_underperforming_validators();
            }

            // 6. Emit periodic health metrics
            if block_number % T::HealthMetricsInterval::get() == 0 {
                Self::emit_dcf_health_metrics(block_number);
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
            log::info!("DCF off-chain worker starting at block {:?}", block_number);
            
            // Run off-chain worker at configured intervals to reduce overhead
            if (block_number.saturated_into::<u32>()) % T::OffchainWorkerInterval::get() != 0 {
                return;
            }

            let result = Self::run_offchain_computation(block_number);
            if let Err(e) = result {
                log::error!("Off-chain worker error: {:?}", e);
            }
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

            log::info!("DCF off-chain worker acquired lock, starting computation");

            // Get current validators
            let validators = Self::validator_set();
            let current_epoch = Self::current_epoch();

            for validator in validators.iter() {
                // Collect inference data for this validator
                if let Ok(inference_data) = Self::collect_inference_data(validator, current_epoch) {
                    // Compute PoI score based on collected data
                    let poi_score = Self::compute_poi_score(&inference_data);
                    
                    // Submit unsigned transaction to update the score
                    if let Err(e) = Self::submit_poi_score_update(validator.clone(), poi_score, block_number) {
                        log::error!("Failed to submit PoI score update for {:?}: {:?}", validator, e);
                    }
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

        /// Get the expected author for a given block number.
        pub fn get_expected_author(block_number: u32) -> Option<T::AccountId> {
            let validators = Self::active_validators();
            if validators.is_empty() {
                log::warn!("DCF: No active validators available for block authorship at block {}", block_number);
                return None;
            }
            let idx = (block_number as usize) % validators.len();
            let author = validators.get(idx).cloned();
            if let Some(ref author) = author {
                log::debug!("DCF: Selected author {:?} for block {} (index {} of {} validators)", 
                           author, block_number, idx, validators.len());
            }
            author
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

        /// Get the inference result for a validator.
        pub fn get_inference_result(account_id: T::AccountId) -> Option<u64> {
            ValidatorStates::<T>::get(&account_id).map(|state| state.current.inference_score)
        }

        /// Execute slashing action on a validator
        fn execute_slash_validator(validator: &T::AccountId, amount: <T as pallet::Config>::Balance) -> DispatchResult {
            // 1. Reduce validator's DCF score based on slash amount
            let score_penalty = (amount.saturated_into::<u64>() / T::SlashPenaltyDivisor::get()).min(T::MaxSlashPenalty::get());
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
                
                log::info!("Slashed validator {:?}: score {} -> {}, penalty: {}", 
                          validator, old_score, state.current.final_score, score_penalty);
                
                Ok::<(), Error<T>>(())
            })?;
            
            // 2. Check if validator should be ejected due to low score
            let current_score = ValidatorStates::<T>::get(validator)
                .map(|s| s.current.final_score)
                .unwrap_or(0);
                
            if current_score < <T as pallet::Config>::MinValidatorScore::get() as u64 {
                let _ = Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold);
                log::info!("Validator {:?} ejected due to low score after slashing", validator);
            }
            
            // 3. Emit event for slashing
            Self::deposit_event(Event::ValidatorScoreUpdated {
                validator: validator.clone(),
                stake_score: ValidatorStates::<T>::get(validator).map(|s| s.current.stake_score).unwrap_or(0),
                inference_score: ValidatorStates::<T>::get(validator).map(|s| s.current.inference_score).unwrap_or(0),
                final_score: current_score,
            });
            
            log::info!("Successfully executed slash on validator {:?}, amount: {:?}", validator, amount);
            Ok(())
        }

        /// Execute reward action on a validator
        fn execute_reward_validator(validator: &T::AccountId, amount: <T as pallet::Config>::Balance) -> DispatchResult {
            // 1. Boost validator's DCF score based on reward amount
            let score_boost = (amount.saturated_into::<u64>() / T::RewardBoostDivisor::get()).min(T::MaxRewardBoost::get());
            
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
                
                log::info!("Rewarded validator {:?}: score {} -> {}, boost: {}", 
                          validator, old_score, state.current.final_score, score_boost);
                
                Ok::<(), Error<T>>(())
            })?;
            
            // 2. Emit event for reward
            let current_score = ValidatorStates::<T>::get(validator)
                .map(|s| s.current.final_score)
                .unwrap_or(0);
                
            Self::deposit_event(Event::ValidatorScoreBoosted {
                validator: validator.clone(),
                old_score: current_score.saturating_sub(score_boost),
                new_score: current_score,
                reason: ScoreBoostReason::ManualBoost,
            });
            
            log::info!("Successfully executed reward on validator {:?}, amount: {:?}", validator, amount);
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

        /// Apply pending join/leave actions at epoch transition.
        fn apply_pending_validator_actions() {
            let mut active = ActiveValidators::<T>::get();
            let mut changed = false;

            // Collect all actions to avoid double borrow
            let actions: Vec<(T::AccountId, ValidatorAction)> =
                PendingValidatorActions::<T>::iter().collect();

            for (who, action) in actions {
                match action {
                    ValidatorAction::Join => {
                        if !active.contains(&who) && active.len() < active.capacity() {
                            active.try_push(who.clone()).ok();
                            changed = true;
                        }
                    }
                    ValidatorAction::Leave => {
                        if let Some(pos) = active.iter().position(|v| v == &who) {
                            active.remove(pos);
                            changed = true;
                        }
                    }
                }
                PendingValidatorActions::<T>::remove(&who);
            }

            if changed {
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
    pub validator: T::AccountId,
    pub epoch: u32,
    pub inference_result: u32,
    pub confidence_score: u32,
    pub timestamp: u64,
    pub data_sources: Vec<Vec<u8>>,
}

/// Off-chain storage structure for PoI scores
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OffchainPoiScore<T: Config> {
    pub validator: T::AccountId,
    pub score: u64,
    pub block_number: u32,
    pub timestamp: u64,
}