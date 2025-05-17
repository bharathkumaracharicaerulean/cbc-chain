#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::Get,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

/// The DCF (Decentralized Consensus Framework) pallet provides functionality for
/// validator scoring and selection based on stake and inference performance.
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Represents the score components for a validator
    #[derive(Clone, Encode, Decode, Default, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct ValidatorScore {
        /// Score based on the validator's stake
        pub stake_weight: u128,
        /// Score based on the validator's inference performance
        pub inference_weight: u128,
        /// Final combined score
        pub final_score: u128,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The maximum number of validators allowed in the set
        #[pallet::constant]
        type MaxValidators: Get<u32>;
        
        /// The minimum stake required to become a validator
        #[pallet::constant]
        type MinStake: Get<u128>;
        
        /// The maximum stake that can be used for scoring
        #[pallet::constant]
        type MaxStake: Get<u128>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // --- Storage ---

    /// Maps validator accounts to their stake-based scores
    #[pallet::storage]
    #[pallet::getter(fn validator_stake_scores)]
    pub type ValidatorStakeScores<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128>;

    /// Maps validator accounts to their inference-based scores
    #[pallet::storage]
    #[pallet::getter(fn validator_inference_scores)]
    pub type ValidatorInferenceScores<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128>;

    /// Maps validator accounts to their final combined scores
    #[pallet::storage]
    #[pallet::getter(fn validator_final_scores)]
    pub type ValidatorFinalScores<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorScore>;

    /// Weight given to proof-of-stake in final score calculation (percentage)
    #[pallet::storage]
    #[pallet::getter(fn pos_weight)]
    pub type PosWeight<T> = StorageValue<_, u128, ValueQuery>;

    /// Weight given to proof-of-inference in final score calculation (percentage)
    #[pallet::storage]
    #[pallet::getter(fn poi_weight)]
    pub type PoiWeight<T> = StorageValue<_, u128, ValueQuery>;

    /// Current set of active validators
    #[pallet::storage]
    #[pallet::getter(fn validator_set)]
    pub type ValidatorSet<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    // --- Events ---

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A validator's score has been updated
        ValidatorScoreUpdated {
            validator: T::AccountId,
            stake_score: u128,
            inference_score: u128,
            final_score: u128,
        },
        /// Weights have been updated
        WeightsUpdated {
            pos_weight: u128,
            poi_weight: u128,
        },
    }

    // --- Errors ---

    #[pallet::error]
    pub enum Error<T> {
        /// Validator set is full
        ValidatorSetFull,
        /// Validator not found
        ValidatorNotFound,
        /// Invalid weight values
        InvalidWeights,
    }

    // --- Hooks ---

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: T::BlockNumber) {
            let pos_weight = Self::pos_weight();
            let poi_weight = Self::poi_weight();
            
            // Ensure weights sum to 100
            if pos_weight + poi_weight != 100 {
                return;
            }

            for validator in Self::validator_set() {
                let stake_score = Self::validator_stake_scores(&validator).unwrap_or(0);
                let inference_score = Self::validator_inference_scores(&validator).unwrap_or(0);
                
                // Calculate final score with proper weighting
                let final_score = (stake_score * pos_weight + inference_score * poi_weight) / 100;
                
                let score = ValidatorScore {
                    stake_weight: stake_score,
                    inference_weight: inference_score,
                    final_score,
                };
                
                ValidatorFinalScores::<T>::insert(&validator, score);
                
                // Emit event for score update
                Self::deposit_event(Event::ValidatorScoreUpdated {
                    validator: validator.clone(),
                    stake_score,
                    inference_score,
                    final_score,
                });
            }
        }
    }

    // --- Helper Functions ---

    impl<T: Config> Pallet<T> {
        /// Updates the weights for PoS and PoI scoring
        pub fn update_weights(pos_weight: u128, poi_weight: u128) -> DispatchResult {
            ensure!(pos_weight + poi_weight == 100, Error::<T>::InvalidWeights);
            
            PosWeight::<T>::put(pos_weight);
            PoiWeight::<T>::put(poi_weight);
            
            Self::deposit_event(Event::WeightsUpdated {
                pos_weight,
                poi_weight,
            });
            
            Ok(().into())
        }

        /// Adds a validator to the set if there's space
        pub fn add_validator(validator: T::AccountId) -> DispatchResult {
            let mut validators = Self::validator_set();
            ensure!(
                validators.len() < T::MaxValidators::get() as usize,
                Error::<T>::ValidatorSetFull
            );
            
            validators.push(validator.clone());
            ValidatorSet::<T>::put(validators);
            
            Ok(().into())
        }
    }
}
