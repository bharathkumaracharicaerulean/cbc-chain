#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#[warn(unused_comparisons)]
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
};
use sp_std::prelude::*;
use pallet_cbc_pos as pos;
use pallet_cbc_poi as poi;
use serde::{Serialize, Deserialize};

// Runtime API declaration
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
    }
}



pub mod weights;
pub use weights::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    #[scale_info(skip_type_params(T))]
    pub struct ValidatorScore {
        pub stake_weight: u64,
        pub inference_weight: u64,
        pub final_score: u64,
        pub last_epoch_active: u32,
        pub participation_count: u32,
        pub missed_blocks: u32,
        pub authored_blocks: u32,
    }

    /// Configuration for epoch transitions
    #[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default, Serialize, Deserialize)]
    #[scale_info(skip_type_params(T))]
    pub struct EpochConfig {
        /// Number of blocks per epoch
        pub blocks_per_epoch: u32,
        /// Minimum stake required to be a validator
        pub min_stake: u128,
        /// Maximum number of validators per epoch
        pub max_validators: u32,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pos::Config + poi::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// The maximum number of validators that can be active at once.
        #[pallet::constant]
        type MaxValidators: Get<u32>;

        /// Default weight for POS score in final calculation (0-100)
        #[pallet::constant]
        type DefaultPosWeight: Get<u64>;

        /// Default weight for POI score in final calculation (0-100)
        #[pallet::constant]
        type DefaultPoiWeight: Get<u64>;

        /// Minimum number of active validators required for a new epoch
        #[pallet::constant]
        type MinActiveValidators: Get<u32>;

        /// Minimum validator score required
        #[pallet::constant]
        type MinValidatorScore: Get<u32>;

        /// Score decay per epoch (percentage, 0-100)
        #[pallet::constant]
        type ValidatorScoreDecay: Get<u32>;

        /// Maximum validator score
        #[pallet::constant]
        type MaxValidatorScore: Get<u64>;

        /// Score boost for valid block authored
        #[pallet::constant]
        type BlockAuthorshipBoost: Get<u64>;

        /// Score penalty for missed block
        #[pallet::constant]
        type MissedBlockPenalty: Get<u64>;

        /// Score boost for valid inference (low/medium/high)
        #[pallet::constant]
        type InferenceBoostLow: Get<u64>;
        #[pallet::constant]
        type InferenceBoostMedium: Get<u64>;
        #[pallet::constant]
        type InferenceBoostHigh: Get<u64>;

        /// Score penalty for inference error (low/medium/high)
        #[pallet::constant]
        type InferencePenaltyLow: Get<u64>;
        #[pallet::constant]
        type InferencePenaltyMedium: Get<u64>;
        #[pallet::constant]
        type InferencePenaltyHigh: Get<u64>;

        /// Minimum stake amount required for validators
        #[pallet::constant]
        type MinStake: Get<<Self as Config>::Balance>;

        /// The balance type
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

        /// Weight information for the pallet
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn validator_stake_scores)]
    pub type ValidatorStakeScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u64,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_inference_scores)]
    pub type ValidatorInferenceScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u64,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_final_scores)]
    pub type ValidatorFinalScores<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorScore,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pos_weight)]
    pub type PosWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn poi_weight)]
    pub type PoiWeight<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_set)]
    pub type ValidatorSet<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    /// Storage for epoch configuration
    #[pallet::storage]
    #[pallet::getter(fn epoch_config)]
    pub type EpochConfigStorage<T: Config> = StorageValue<_, EpochConfig, ValueQuery>;

    /// Storage for current epoch number
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Storage for active validators in current epoch
    #[pallet::storage]
    #[pallet::getter(fn active_validators)]
    pub type ActiveValidators<T: Config> = StorageValue<_, BoundedVec<T::AccountId, <T as Config>::MaxValidators>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_score_history)]
    pub type ValidatorScoreHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<10>>, // Store last 10 epochs of scores
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_participation)]
    pub type ValidatorParticipation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (u32, u32), // (authored_blocks, missed_blocks)
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_last_active)]
    pub type ValidatorLastActive<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Validator score updated
        ValidatorScoreUpdated {
            validator: T::AccountId,
            stake_score: u64,
            inference_score: u64,
            final_score: u64,
        },
        /// Consensus weights updated
        ConsensusWeightsUpdated {
            pos_weight: u64,
            poi_weight: u64,
        },
        /// A new epoch has started
        EpochStarted {
            epoch: u32,
            validators: Vec<T::AccountId>,
        },
        /// Validator score decayed
        ValidatorScoreDecayed {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
        },
        /// Validator score boosted
        ValidatorScoreBoosted {
            validator: T::AccountId,
            old_score: u64,
            new_score: u64,
            reason: ScoreBoostReason,
        },
        /// Validator ejected from active set
        ValidatorEjected {
            validator: T::AccountId,
            reason: EjectionReason,
        },
        /// Validator re-entered active set
        ValidatorReEntered {
            validator: T::AccountId,
            score: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Validator not found
        ValidatorNotFound,
        /// Invalid weight value
        InvalidWeight,
        /// Invalid epoch configuration
        InvalidEpochConfig,
        /// Not enough validators for epoch
        NotEnoughValidators,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn update_validator_stake_score(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            // Get stake score from POS pallet
            let stake = pos::Pallet::<T>::stake(&validator);
            let stake_score = stake.saturated_into::<u64>();
            
            ValidatorStakeScores::<T>::insert(&validator, stake_score);
            Self::update_final_score(&validator)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn update_validator_inference_score(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            // Get inference score from POI pallet
            if let Some((result, _)) = poi::Pallet::<T>::inference_results(&validator) {
                let inference_score = result as u64;
                ValidatorInferenceScores::<T>::insert(&validator, inference_score);
                Self::update_final_score(&validator)?;
            }
            Ok(())
        }

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
    }

    impl<T: Config> Pallet<T> {
        /// Update the final score for a validator based on weighted PoS and PoI scores.
        /// All weights and thresholds are runtime-configurable.
        fn update_final_score(validator: &T::AccountId) -> DispatchResult {
            let stake_score = ValidatorStakeScores::<T>::get(validator);
            let inference_score = ValidatorInferenceScores::<T>::get(validator);

            // Use runtime-configured weights, fallback to defaults if not set
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

            // Weighted sum, normalized to 100
            let mut final_score = (stake_score.saturating_mul(pos_weight) + inference_score.saturating_mul(poi_weight)) / 100;

            // Clamp to max validator score
            if final_score > T::MaxValidatorScore::get() {
                final_score = T::MaxValidatorScore::get();
            }

            ValidatorFinalScores::<T>::insert(
                validator,
                ValidatorScore {
                    stake_weight: stake_score,
                    inference_weight: inference_score,
                    final_score,
                    last_epoch_active: 0,
                    participation_count: 0,
                    missed_blocks: 0,
                    authored_blocks: 0,
                },
            );

            Self::deposit_event(Event::ValidatorScoreUpdated {
                validator: validator.clone(),
                stake_score,
                inference_score,
                final_score,
            });

            Ok(())
        }

        /// Apply score decay to a validator's score if inactive for one or more epochs.
        /// Decay rate and min/max scores are runtime-configurable.
        fn apply_score_decay(validator: &T::AccountId, current_epoch: u32) -> DispatchResult {
            let mut score = ValidatorFinalScores::<T>::get(validator);
            let last_active = ValidatorLastActive::<T>::get(validator);

            // Calculate epochs since last activity
            let inactive_epochs = current_epoch.saturating_sub(last_active);

            if inactive_epochs > 0 {
                let decay_rate = <T as pallet::Config>::ValidatorScoreDecay::get();
                let decay_amount = score.final_score.saturating_mul(decay_rate as u64) / 100u64;

                let old_score = score.final_score;
                score.final_score = score.final_score.saturating_sub(decay_amount);

                // Clamp to MaxValidatorScore only (no need to clamp to zero, saturating_sub already does it)
                if score.final_score > T::MaxValidatorScore::get() {
                    score.final_score = T::MaxValidatorScore::get();
                }

                ValidatorFinalScores::<T>::insert(validator, score.clone());

                Self::deposit_event(Event::ValidatorScoreDecayed {
                    validator: validator.clone(),
                    old_score,
                    new_score: score.final_score,
                });

                // Eject if below minimum threshold
                if score.final_score < <T as Config>::MinValidatorScore::get() as u64 {
                    Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold)?;
                }
            }

            Ok(())
        }

        /// Boost a validator's score for positive actions (block authored, valid inference, etc).
        /// Boost amounts are runtime-configurable.
        fn boost_score(
            validator: &T::AccountId,
            amount: u64,
            reason: ScoreBoostReason,
        ) -> DispatchResult {
            let mut score = ValidatorFinalScores::<T>::get(validator);
            let old_score = score.final_score;

            score.final_score = score.final_score.saturating_add(amount);
            // Clamp to max
            if score.final_score > T::MaxValidatorScore::get() {
                score.final_score = T::MaxValidatorScore::get();
            }
            score.last_epoch_active = Self::current_epoch();

            // Update storage
            ValidatorFinalScores::<T>::insert(validator, score.clone());
            ValidatorLastActive::<T>::insert(validator, Self::current_epoch());

            // Update score history (last 10)
            let history = ValidatorScoreHistory::<T>::get(validator);
            let mut new_history = Vec::new();
            if history.len() >= 10 {
                new_history.extend_from_slice(&history[1..]);
            } else {
                new_history.extend_from_slice(&history);
            }
            new_history.push(score.final_score);
            let bounded_history: BoundedVec<u64, ConstU32<10>> = new_history.try_into().expect("We know this is bounded by 10");
            ValidatorScoreHistory::<T>::insert(validator, bounded_history);

            // Emit event
            Self::deposit_event(Event::ValidatorScoreBoosted {
                validator: validator.clone(),
                old_score,
                new_score: score.final_score,
                reason,
            });

            Ok(())
        }

        /// Penalize a validator for missed blocks.
        /// Penalty amount is runtime-configurable.
        fn record_missed_block(validator: &T::AccountId) -> DispatchResult {
            let mut participation = ValidatorParticipation::<T>::get(validator);
            participation.1 = participation.1.saturating_add(1);
            ValidatorParticipation::<T>::insert(validator, participation);

            let mut score = ValidatorFinalScores::<T>::get(validator);
            let penalty = T::MissedBlockPenalty::get();
            score.final_score = score.final_score.saturating_sub(penalty);
            ValidatorFinalScores::<T>::insert(validator, score);

            Ok(())
        }

        /// Reward a validator for block authorship.
        /// Boost amount is runtime-configurable.
        fn record_block_authorship(validator: &T::AccountId) -> DispatchResult {
            let mut participation = ValidatorParticipation::<T>::get(validator);
            participation.0 = participation.0.saturating_add(1);
            ValidatorParticipation::<T>::insert(validator, participation);

            Self::boost_score(
                validator,
                T::BlockAuthorshipBoost::get(),
                ScoreBoostReason::ValidBlockAuthored,
            )
        }

        /// Reward a validator for valid inference, boost depends on confidence.
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

        /// Penalize a validator for invalid/challenged inference.
        /// Penalty depends on severity and is runtime-configurable.
        fn handle_invalid_inference(
            validator: &T::AccountId,
            severity: InferenceErrorSeverity,
        ) -> DispatchResult {
            let penalty = match severity {
                InferenceErrorSeverity::High => T::InferencePenaltyHigh::get(),
                InferenceErrorSeverity::Medium => T::InferencePenaltyMedium::get(),
                InferenceErrorSeverity::Low => T::InferencePenaltyLow::get(),
            };

            let mut score = ValidatorFinalScores::<T>::get(validator);
            let new_score = score.final_score.saturating_sub(penalty);
            score.final_score = new_score;
            ValidatorFinalScores::<T>::insert(validator, score.clone());

            if new_score < <T as Config>::MinValidatorScore::get() as u64 {
                Self::eject_validator(validator, EjectionReason::ScoreBelowThreshold)?;
            }

            Ok(())
        }

        // Eject a validator from the active set for a given reason
        fn eject_validator(validator: &T::AccountId, reason: EjectionReason) -> DispatchResult {
            // Remove from active set
            let mut active_validators = ActiveValidators::<T>::get();
            if let Some(pos) = active_validators.iter().position(|v| v == validator) {
                active_validators.remove(pos);
                ActiveValidators::<T>::put(active_validators);
            }
            // Emit event
            Self::deposit_event(Event::ValidatorEjected {
                validator: validator.clone(),
                reason,
            });
            Ok(())
        }

        // Determine if an epoch transition should occur
        fn should_transition_epoch(
            now: BlockNumberFor<T>,
            current_epoch: u32,
            epoch_config: &EpochConfig,
        ) -> bool {
            // Transition if enough blocks have passed since last epoch
            let blocks_per_epoch = epoch_config.blocks_per_epoch;
            let epoch_start_block = current_epoch.saturating_mul(blocks_per_epoch);
            let now_u32: u32 = now.saturated_into();
            now_u32 >= epoch_start_block + blocks_per_epoch
        }

        // Handle epoch transition logic
        fn handle_epoch_transition() -> Weight {
            let current_epoch = Self::current_epoch();
            let next_epoch = current_epoch.saturating_add(1);
            CurrentEpoch::<T>::put(next_epoch);

            // Select new validator set (for now, just keep the same set)
            let active_validators = ActiveValidators::<T>::get();
            Self::deposit_event(Event::EpochStarted {
                epoch: next_epoch,
                validators: active_validators.clone().into_inner(),
            });

            // Optionally, update scores or perform other epoch tasks here

            <T as Config>::WeightInfo::on_initialize()
        }

        // Get the expected block author for a given block number
        fn get_expected_author(now: BlockNumberFor<T>) -> Option<T::AccountId> {
            let validators = ActiveValidators::<T>::get();
            if validators.is_empty() {
                return None;
            }
            let now_u32: u32 = now.saturated_into();
            let idx = (now_u32 as usize) % validators.len();
            validators.get(idx).cloned()
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let epoch_config = Self::epoch_config();
            let current_epoch = Self::current_epoch();
            
            // Check if we should transition to a new epoch
            if Self::should_transition_epoch(now, current_epoch, &epoch_config) {
                Self::handle_epoch_transition()
            } else {
                // Validate block author
                if let Some(expected_author) = Self::get_expected_author(now) {
                    let actual_author = frame_system::Pallet::<T>::digest()
                        .logs()
                        .iter()
                        .find_map(|log| {
                            if let DigestItem::Consensus(_, data) = log {
                                // Convert Vec<u8> to AccountId
                                let account_id = T::AccountId::decode(&mut &data[..]).ok()?;
                                Some(account_id)
                            } else {
                                None
                            }
                        });
                    
                    if let Some(actual) = actual_author {
                        if actual != expected_author {
                            // Record missed block for expected author
                            let _ = Self::record_missed_block(&expected_author);
                        } else {
                            // Record successful authorship
                            let _ = Self::record_block_authorship(&actual);
                        }
                    }
                }
                
                <T as Config>::WeightInfo::on_initialize()
            }
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            // Update all validator scores at the end of each block
            let validators = ValidatorSet::<T>::get();
            for validator in validators.iter() {
                let _ = Self::update_final_score(validator);
            }
        }
    }

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
            // Set initial validators
            ValidatorSet::<T>::put(
                BoundedVec::try_from(self.validators.clone())
                    .expect("Initial validators exceed MaxValidators"),
            );

            // Set initial validator scores
            for (validator, score) in self.validators.iter().zip(self.validator_scores.iter()) {
                ValidatorStakeScores::<T>::insert(validator, *score as u64);
                ValidatorInferenceScores::<T>::insert(validator, *score as u64);
                
                // Calculate final score directly instead of calling update_final_score
                let pos_weight = T::DefaultPosWeight::get();
                let poi_weight = T::DefaultPoiWeight::get();
                let final_score = (*score as u64 * pos_weight + *score as u64 * poi_weight) / 100;
                
                ValidatorFinalScores::<T>::insert(
                    validator,
                    ValidatorScore {
                        stake_weight: *score as u64,
                        inference_weight: *score as u64,
                        final_score,
                        last_epoch_active: 0,
                        participation_count: 0,
                        missed_blocks: 0,
                        authored_blocks: 0,
                    },
                );
            }

            // Set initial epoch
            CurrentEpoch::<T>::put(self.current_epoch);

            // Set epoch config
            EpochConfigStorage::<T>::put(self.epoch_config.clone());

            // Set initial weights
            PosWeight::<T>::put(T::DefaultPosWeight::get());
            PoiWeight::<T>::put(T::DefaultPoiWeight::get());
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum ScoreBoostReason {
    ValidBlockAuthored,
    ValidInference,
    ManualBoost,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum EjectionReason {
    ScoreBelowThreshold,
    MaxSlashingReached,
    ManualEjection,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, frame_support::__private::codec::DecodeWithMemTracking)]
pub enum InferenceErrorSeverity {
    High,   // Major error, significant impact
    Medium, // Moderate error
    Low,    // Minor error
}

