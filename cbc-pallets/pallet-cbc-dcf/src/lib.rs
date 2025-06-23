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
mod tests;

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
};
use sp_std::prelude::*;
use sp_std::fmt; // <-- Add this import
use pallet_cbc_pos as pos;
use pallet_cbc_poi as poi;
use serde::{Serialize, Deserialize};

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
        fn get_validator_profile(account_id: AccountId) -> Option<(u64, u32, u32, u32, u32)>;
        fn get_inference_result(account_id: AccountId) -> Option<u64>;
        fn get_epoch_history(epoch_number: u32) -> Option<RuntimeEpochHistory<AccountId>>;
        fn get_recent_epochs(n: u32) -> Vec<RuntimeEpochHistory<AccountId>>;
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
        pub history: BoundedVec<EpochStats, ConstU32<10>>,
        pub uptime: u32, // Number of epochs active
        pub inference_success_count: u32,
        pub participation_rate: u32, // Percentage (0-100)
    }

    /// Configuration for epochs (block count, min stake, max validators).
    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default, Serialize, Deserialize)]
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

    /// Non-generic struct for runtime API (AccountId = T::AccountId, all BoundedVecs use MaxValidators, history uses 24).
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct RuntimeEpochHistory<AccountId> {
        pub epoch_number: u32,
        pub active_validators: BoundedVec<AccountId, ConstU32<24>>,
        pub score_snapshot: BoundedVec<(AccountId, u64), ConstU32<24>>,
        pub inference_summary: BoundedVec<(AccountId, Option<u64>), ConstU32<24>>,
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

    // --- Pallet Configuration Trait --- //
    #[pallet::config]
    pub trait Config: frame_system::Config + pos::Config + poi::Config + TypeInfo + fmt::Debug {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        #[pallet::constant]
        type MaxEpochHistory: Get<u32>;
        #[pallet::constant]
        type DefaultPosWeight: Get<u64>;
        #[pallet::constant]
        type DefaultPoiWeight: Get<u64>;
        #[pallet::constant]
        type MinActiveValidators: Get<u32>;
        #[pallet::constant]
        type MinValidatorScore: Get<u32>;
        #[pallet::constant]
        type ValidatorScoreDecay: Get<u32>;
        #[pallet::constant]
        type MaxValidatorScore: Get<u64>;
        #[pallet::constant]
        type BlockAuthorshipBoost: Get<u64>;
        #[pallet::constant]
        type MissedBlockPenalty: Get<u64>;
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

    /// Stores recent epoch histories in a ring buffer.
    #[pallet::storage]
    #[pallet::getter(fn epoch_histories)]
    pub type EpochHistories<T: Config> = StorageValue<_, BoundedVec<EpochHistory<T>, ConstU32<24>>, ValueQuery>;

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

        /// Update consensus weights for PoS and PoI.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn update_consensus_weights(
            origin: OriginFor<T>,
            pos_weight: u64,
            poi_weight: u64,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(pos_weight + poi_weight == 100, Error::<T>::InvalidWeight);
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

                // Execute action
                match &prop.action {
                    ProposalAction::Slash { validator, amount } => {
                        // Slash logic here (call PoS or custom logic)
                        // For demo: just eject if amount > 0
                        if *amount > Zero::zero() {
                            let _ = Self::eject_validator(validator, EjectionReason::MaxSlashingReached);
                        }
                    }
                    ProposalAction::Reward { validator, amount } => {
                        // Reward logic here (call PoS or custom logic)
                        // For demo: boost score
                        let _ = Self::boost_score(validator, (*amount).saturated_into(), ScoreBoostReason::ManualBoost);
                    }
                    ProposalAction::Eject { validator, reason } => {
                        let _ = Self::eject_validator(validator, reason.clone());
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
                Error::<T>::NotAllowedInGovernanceMode // Reuse or add a new error if needed
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
                let mut final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / 100;
                if final_score > T::MaxValidatorScore::get() {
                    final_score = T::MaxValidatorScore::get();
                }
                let _old_score = state.current.final_score;
                state.current.final_score = final_score;
                state.last_active_epoch = Self::current_epoch();
                if state.history.len() == state.history.capacity() {
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
                    let decay_amount = state.current.final_score.saturating_mul(decay_rate as u64) / 100u64;
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
            let boost_amount = if confidence >= 90 {
                T::InferenceBoostHigh::get()
            } else if confidence >= 70 {
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
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let epoch_config = Self::epoch_config();
            let current_epoch = Self::current_epoch();

            // Handle epoch transition if needed
            if !Self::governance_mode_enabled() && Self::should_transition_epoch(now, current_epoch, &epoch_config) {
                Self::handle_epoch_transition()
            } else {
                // Check block author and update scores
                if let Some(expected_author) = Self::get_expected_author(now.saturated_into::<u32>()) {
                    let actual_author = frame_system::Pallet::<T>::digest()
                        .logs()
                        .iter()
                        .find_map(|log| {
                            if let DigestItem::Consensus(_, data) = log {
                                let account_id = T::AccountId::decode(&mut &data[..]).ok()?;
                                Some(account_id)
                            } else {
                                None
                            }
                        });
                    
                    if let Some(actual) = actual_author {
                        if actual != expected_author {
                            let _ = Self::record_missed_block(&expected_author);
                        } else {
                            let _ = Self::record_block_authorship(&actual);
                        }
                    }
                }

                <T as Config>::WeightInfo::on_initialize()
            }
        }

        /// Called at the end of each block.
        fn on_finalize(_n: BlockNumberFor<T>) {
            let validators = ValidatorSet::<T>::get();
            for validator in validators.iter() {
                let _ = Self::update_final_score(validator);
            }
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
                let final_score = (*score as u64 * pos_weight + *score as u64 * poi_weight) / 100;
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
                return None;
            }
            let idx = (block_number as usize) % validators.len();
            validators.get(idx).cloned()
        }

        /// Get validator profile information.
        pub fn get_validator_profile(account_id: T::AccountId) -> Option<(u64, u32, u32, u32, u32)> {
            ValidatorStates::<T>::get(&account_id).map(|state| (
                state.current.final_score,
                state.uptime,
                state.inference_success_count,
                state.participation_rate,
                state.current.missed_blocks,
            ))
        }

        /// Get the inference result for a validator.
        pub fn get_inference_result(account_id: T::AccountId) -> Option<u64> {
            ValidatorStates::<T>::get(&account_id).map(|state| state.current.inference_score)
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
    }
}

// --- Benchmarking (if enabled) --- //
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

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




