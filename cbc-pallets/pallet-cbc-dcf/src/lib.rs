#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::Get,
    BoundedVec,
    Parameter,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
    traits::{Saturating, SaturatedConversion, AtLeast32BitUnsigned},
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
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

        /// Score decay per epoch
        #[pallet::constant]
        type ValidatorScoreDecay: Get<u32>;

        /// Maximum slashing count before removal
        #[pallet::constant]
        type MaxSlashingCount: Get<u32>;

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
            
            // Get validator score from POS pallet
            let stake_score = pos::Pallet::<T>::validator_scores(&validator)
                .ok_or(Error::<T>::ValidatorNotFound)? as u64;
            
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
            
            // Get inference score from POI pallet using the new calculate_validator_score function
            let inference_score = poi::Pallet::<T>::calculate_validator_score(&validator)
                .ok_or(Error::<T>::ValidatorNotFound)? as u64;
            
            ValidatorInferenceScores::<T>::insert(&validator, inference_score);
            Self::update_final_score(&validator)?;
            
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
        fn update_final_score(validator: &T::AccountId) -> DispatchResult {
            let stake_score = ValidatorStakeScores::<T>::get(validator);
            let inference_score = ValidatorInferenceScores::<T>::get(validator);
            
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
            
            // Calculate final score with proper order of operations
            let final_score = ((stake_score as u128 * pos_weight as u128) + 
                              (inference_score as u128 * poi_weight as u128)) / 100;
            
            ValidatorFinalScores::<T>::insert(
                validator,
                ValidatorScore {
                    stake_weight: stake_score,
                    inference_weight: inference_score,
                    final_score: final_score as u64,
                },
            );
            
            Self::deposit_event(Event::ValidatorScoreUpdated {
                validator: validator.clone(),
                stake_score,
                inference_score,
                final_score: final_score as u64,
            });
            
            Ok(())
        }

        /// Check if we should transition to a new epoch
        fn should_transition_epoch(now: BlockNumberFor<T>, current_epoch: u32, epoch_config: &EpochConfig) -> bool {
            let blocks_per_epoch: BlockNumberFor<T> = epoch_config.blocks_per_epoch.into();
            let current_epoch_blocks: BlockNumberFor<T> = (current_epoch as u64).saturated_into();
            let blocks_in_current_epoch = now.saturating_sub(current_epoch_blocks);
            
            blocks_in_current_epoch >= blocks_per_epoch
        }

        /// Transition to a new epoch
        fn transition_epoch() -> Weight {
            let validators = Self::active_validators();
            if validators.len() < <T as Config>::MinActiveValidators::get() as usize {
                return <T as Config>::WeightInfo::on_initialize(1);
            }

            let weight = Weight::zero();
            let epoch_config = Self::epoch_config();
            let current_epoch = Self::current_epoch();
            let new_epoch = current_epoch.saturating_add(1);

            // Select new validators based on scores
            let mut new_validators = BoundedVec::new();
            for validator in validators.iter() {
                if new_validators.len() >= epoch_config.max_validators as usize {
                    break;
                }
                new_validators.try_push(validator.clone()).expect("MaxValidators bound ensures this won't fail");
            }

            // Update storage
            CurrentEpoch::<T>::put(new_epoch);
            ActiveValidators::<T>::put(new_validators.clone());

            // Emit event
            Self::deposit_event(Event::EpochStarted {
                epoch: new_epoch,
                validators: new_validators.to_vec(),
            });

            weight
        }

        /// Check if an account is an active validator in the current epoch
        pub fn is_active_validator(account: &T::AccountId) -> bool {
            Self::active_validators().contains(account)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();

            // Check if we need to transition to a new epoch
            if Self::should_transition_epoch(now, Self::current_epoch(), &Self::epoch_config()) {
                weight = weight.saturating_add(Self::transition_epoch());
            }

            weight
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
