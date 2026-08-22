#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::traits::AtLeast32BitUnsigned;
use scale_info::TypeInfo;
use serde::{Serialize, Deserialize};



pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Reasons for ejecting validators from the network.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Serialize, Deserialize, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum EjectionReason {
    ScoreBelowThreshold,
    MaxSlashingReached,
    ManualEjection,
    ExcessValidators,
    RepeatedMisbehavior,
    InsufficientStake,
    GovernanceDecision,
    ConsecutiveMissedBlocks,
    ConsensusRuleViolation,
}

/// Actions that can be proposed via governance.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Serialize, Deserialize, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum ProposalAction<AccountId, Balance> {
    Slash { validator: AccountId, amount: Balance },
    Reward { validator: AccountId, amount: Balance },
    Eject { validator: AccountId, reason: EjectionReason },
    AddValidator { validator: AccountId },
    RemoveValidator { validator: AccountId },
    RewardMultiple { validators: Vec<AccountId>, amount: Balance },
}

pub type ApiProposalAction<AccountId, Balance> = ProposalAction<AccountId, Balance>;

/// Status of a governance proposal.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Serialize, Deserialize, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
}

/// Governance proposal structure.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Serialize, Deserialize, frame_support::__private::codec::DecodeWithMemTracking)]
pub struct GovernanceProposal<AccountId, ProposalAction> {
    pub proposer: AccountId,
    pub action: ProposalAction,
    pub status: ProposalStatus,
    pub votes_for: u32,
    pub votes_against: u32,
}

/// Executor trait for applying approved governance proposal actions.
pub trait ProposalExecutor<AccountId, Balance> {
    fn slash_validator(validator: &AccountId, amount: Balance) -> DispatchResult;
    fn reward_validator(validator: &AccountId, amount: Balance) -> DispatchResult;
    fn eject_validator(validator: &AccountId, reason: EjectionReason) -> DispatchResult;
    fn add_validator(validator: &AccountId) -> DispatchResult;
    fn remove_validator(validator: &AccountId) -> DispatchResult;
}

/// Validator provider trait for reading active set, verifying private chain mode, and rate-limiting.
pub trait ValidatorProvider<AccountId, Weight> {
    fn active_validators() -> Vec<AccountId>;
    fn is_private_chain_mode() -> bool;
    fn validate_governance_in_private_mode(proposer: &AccountId) -> DispatchResult;
    fn validate_proposal_in_private_mode(proposer: &AccountId, target: &AccountId) -> DispatchResult;
    fn check_rate_limits(proposer: &AccountId, op_type: u8, weight: Weight) -> Result<(), (DispatchError, Option<(u8, u8, u32, u32)>)>;
    fn record_operation(proposer: &AccountId, op_type: u8);
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen + Serialize + for<'de> Deserialize<'de>;
        type ProposalExecutor: ProposalExecutor<Self::AccountId, Self::Balance>;
        type ValidatorProvider: ValidatorProvider<Self::AccountId, Weight>;
        /// Information on runtime weights.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn governance_mode_enabled)]
    pub type GovernanceModeEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        GovernanceProposal<T::AccountId, ProposalAction<T::AccountId, T::Balance>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_votes)]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        GovernanceModeSet { enabled: bool },
        ProposalSubmitted {
            proposal_id: u32,
            proposer: T::AccountId,
            action: ProposalAction<T::AccountId, T::Balance>,
        },
        ProposalCreated {
            proposal_id: u32,
            proposer: T::AccountId,
            action: ProposalAction<T::AccountId, T::Balance>,
            description: BoundedVec<u8, ConstU32<128>>,
        },
        ProposalVoted {
            proposal_id: u32,
            voter: T::AccountId,
            approve: bool,
        },
        ProposalPassed { proposal_id: u32 },
        ProposalRejected { proposal_id: u32 },
        ProposalExecuted {
            proposal_id: u32,
            action: ProposalAction<T::AccountId, T::Balance>,
        },
        RateLimitViolation {
            account: T::AccountId,
            operation: u8,
            violation_type: u8,
            current_count: u32,
            limit: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NotAllowedInGovernanceMode,
        ProposalNotApproved,
        ProposalAlreadyExecuted,
        AlreadyVoted,
        ProposalNotFound,
        ValidatorNotInSet,
        ValidatorNotFound,
        RateLimitExceeded,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Enable or disable governance mode (Root only).
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_governance_mode())]
        pub fn set_governance_mode(
            origin: OriginFor<T>,
            enabled: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            GovernanceModeEnabled::<T>::put(enabled);
            Self::deposit_event(Event::GovernanceModeSet { enabled });
            Ok(())
        }

        /// Submit a governance proposal (slash, reward, eject, add/remove validator).
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_proposal())]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            action: ProposalAction<T::AccountId, T::Balance>,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            Self::submit_proposal_internal(proposer, action, description)
        }

        /// Vote on a governance proposal.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::vote_proposal())]
        pub fn vote_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check rate limits
            let weight = T::WeightInfo::vote_proposal();
            if let Err((e, violation_opt)) = T::ValidatorProvider::check_rate_limits(&who, 6u8, weight) {
                if let Some((operation_code, violation_type, current_count, limit)) = violation_opt {
                    Self::deposit_event(Event::RateLimitViolation {
                        account: who,
                        operation: operation_code,
                        violation_type,
                        current_count,
                        limit,
                    });
                }
                return Err(e);
            }
            
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
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
                    voter: who.clone(),
                    approve,
                });

                // Quorum logic
                let active_validators = T::ValidatorProvider::active_validators();
                let quorum = (active_validators.len() as u32 + 1) / 2;
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
            
            T::ValidatorProvider::record_operation(&who, 6u8);
            
            Ok(())
        }

        /// Execute an approved governance proposal (sudo only).
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::execute_proposal())]
        pub fn execute_proposal(
            origin: OriginFor<T>,
            proposal_id: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Proposals::<T>::try_mutate_exists(proposal_id, |maybe_prop| {
                let prop = maybe_prop.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                ensure!(matches!(prop.status, ProposalStatus::Approved), Error::<T>::ProposalNotApproved);

                match &prop.action {
                    ProposalAction::Slash { validator, amount } => {
                        T::ProposalExecutor::slash_validator(validator, *amount)?;
                    }
                    ProposalAction::Reward { validator, amount } => {
                        T::ProposalExecutor::reward_validator(validator, *amount)?;
                    }
                    ProposalAction::RewardMultiple { validators, amount } => {
                        for validator in validators.iter() {
                            T::ProposalExecutor::reward_validator(validator, *amount)?;
                        }
                    }
                    ProposalAction::Eject { validator, reason } => {
                        T::ProposalExecutor::eject_validator(validator, reason.clone())?;
                    }
                    ProposalAction::AddValidator { validator } => {
                        T::ProposalExecutor::add_validator(validator)?;
                    }
                    ProposalAction::RemoveValidator { validator } => {
                        T::ProposalExecutor::remove_validator(validator)?;
                    }
                }
                
                prop.status = ProposalStatus::Executed;
                Self::deposit_event(Event::ProposalExecuted {
                    proposal_id,
                    action: prop.action.clone(),
                });
                
                Ok::<(), DispatchError>(())
            })?;
            
            Ok(())
        }

        /// Propose validator slashing.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::propose_slash_validator())]
        pub fn propose_slash_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::Slash { validator, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose validator reward.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::propose_reward_validator())]
        pub fn propose_reward_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::Reward { validator, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose default validator reward.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::propose_default_reward_validator())]
        pub fn propose_default_reward_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::Reward { validator, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose rewards for multiple validators.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::propose_reward_multiple_validators())]
        pub fn propose_reward_multiple_validators(
            origin: OriginFor<T>,
            validators: Vec<T::AccountId>,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::RewardMultiple { validators, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose default rewards for multiple validators.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::propose_default_reward_multiple_validators())]
        pub fn propose_default_reward_multiple_validators(
            origin: OriginFor<T>,
            validators: Vec<T::AccountId>,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::RewardMultiple { validators, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose rewards for all active validators.
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::propose_reward_all_active_validators())]
        pub fn propose_reward_all_active_validators(
            origin: OriginFor<T>,
            amount: T::Balance,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let active_set = T::ValidatorProvider::active_validators();
            let mut validators = Vec::new();
            for val in active_set {
                validators.push(val);
            }
            let action = ProposalAction::RewardMultiple { validators, amount };
            Self::submit_proposal(origin, action, description)
        }

        /// Propose validator ejection.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::propose_eject_validator())]
        pub fn propose_eject_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
            reason: EjectionReason,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let action = ProposalAction::Eject { validator, reason };
            Self::submit_proposal(origin, action, description)
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn submit_proposal_internal(
            proposer: T::AccountId,
            action: ProposalAction<T::AccountId, T::Balance>,
            description: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            // Check rate limits using ValidatorProvider
            let weight = T::DbWeight::get().reads_writes(4, 2);
            if let Err((e, violation_opt)) = T::ValidatorProvider::check_rate_limits(&proposer, 5u8, weight) {
                if let Some((operation_code, violation_type, current_count, limit)) = violation_opt {
                    Self::deposit_event(Event::RateLimitViolation {
                        account: proposer.clone(),
                        operation: operation_code,
                        violation_type,
                        current_count,
                        limit,
                    });
                }
                return Err(e);
            }
            
            // Check private chain governance restrictions
            if T::ValidatorProvider::is_private_chain_mode() {
                T::ValidatorProvider::validate_governance_in_private_mode(&proposer)?;
                
                match &action {
                    ProposalAction::Slash { validator, .. } |
                    ProposalAction::Reward { validator, .. } => {
                        T::ValidatorProvider::validate_proposal_in_private_mode(&proposer, validator)?;
                    },
                    ProposalAction::Eject { validator, .. } => {
                        T::ValidatorProvider::validate_proposal_in_private_mode(&proposer, validator)?;
                    },
                    _ => {}
                }
            }
            
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
            
            Self::deposit_event(Event::ProposalCreated {
                proposal_id,
                proposer: proposer.clone(),
                action,
                description: description.unwrap_or_else(|| BoundedVec::truncate_from(b"No description provided".to_vec())),
            });
            
            T::ValidatorProvider::record_operation(&proposer, 5u8);
            
            Ok(())
        }

        pub fn get_proposal_details(proposal_id: u32) -> Option<GovernanceProposal<T::AccountId, ProposalAction<T::AccountId, T::Balance>>> {
            Proposals::<T>::get(proposal_id)
        }
        pub fn get_active_proposals() -> Vec<(u32, GovernanceProposal<T::AccountId, ProposalAction<T::AccountId, T::Balance>>)> {
            Proposals::<T>::iter()
                .filter(|(_, proposal)| proposal.status == ProposalStatus::Pending)
                .collect()
        }
    }
}
