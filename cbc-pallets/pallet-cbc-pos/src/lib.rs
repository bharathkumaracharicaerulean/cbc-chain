#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{pallet_prelude::*, storage::types::StorageMap};
use frame_system::pallet_prelude::*;
use scale_info::prelude::vec::Vec;
use pallet_cbc_dcf::ValidatorStakeScoreProvider;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_cbc_dcf::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;

        /// Minimum score required for a validator to be considered active
        type MinValidatorScore: Get<u32>;
        /// Minimum number of active validators required
        type MinActiveValidators: Get<u32>;
        /// Maximum number of validators allowed
        type MaxValidators: Get<u32>;
        /// Score decay per epoch (in percentage, e.g., 10 means 10% decay)
        type ValidatorScoreDecay: Get<u32>;
        /// Maximum slashing count before removal
        type MaxSlashingCount: Get<u32>;
        /// Maximum score a validator can have
        type MaxValidatorScore: Get<u32>;
        /// Number of epochs to keep score history
        type ScoreHistoryLength: Get<u32>;
        /// Score boost for valid block authorship
        type BlockAuthorshipBoost: Get<u32>;
        /// Score boost for accurate inference
        type InferenceAccuracyBoost: Get<u32>;
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub validators: Vec<T::AccountId>,
        pub validator_scores: Vec<u32>,
        pub slashing_count: Vec<(T::AccountId, u32)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Initialize validators
            for validator in &self.validators {
                Validators::<T>::insert(validator, true);
            }

            // Initialize validator scores
            for (validator, score) in self.validators.iter().zip(self.validator_scores.iter()) {
                ValidatorScores::<T>::insert(validator, score);
            }

            // Initialize slashing counts
            for (validator, count) in &self.slashing_count {
                SlashingCount::<T>::insert(validator, count);
            }
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

    #[pallet::storage]
    #[pallet::getter(fn validator_scores)]
    pub type ValidatorScores<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

    #[pallet::storage]
    #[pallet::getter(fn slashing_count)]
    pub type SlashingCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

    #[pallet::storage]
    #[pallet::getter(fn validator_score_history)]
    pub type ValidatorScoreHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<(u32, u32), ConstU32<10>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorRegistered { validator: T::AccountId },
        ScoreSubmitted { validator: T::AccountId, score: u32 },
        ValidatorSlashed { validator: T::AccountId, slashing_count: u32 },
        ValidatorRemoved { validator: T::AccountId, reason: Vec<u8> },
        ScoreBoosted { validator: T::AccountId, boost_type: Vec<u8>, amount: u32 },
        ScoreDecayed { validator: T::AccountId, new_score: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        InvalidScore,
        TooManyValidators,
        ScoreTooLow,
        MaxSlashingCountReached,
        InvalidBoostAmount,
        ScoreHistoryFull,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_validator())]
        pub fn register_validator(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            
            // Check if we've reached max validators
            ensure!(
                Validators::<T>::iter().count() < <T as pallet::Config>::MaxValidators::get() as usize,
                Error::<T>::TooManyValidators
            );

            ensure!(!Validators::<T>::contains_key(&_who), Error::<T>::ValidatorAlreadyRegistered);
            Validators::<T>::insert(&_who, true);
            Self::deposit_event(Event::ValidatorRegistered { validator: _who });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_score())]
        pub fn submit_score(origin: OriginFor<T>, validator: T::AccountId, score: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            ensure!(score >= <T as pallet::Config>::MinValidatorScore::get(), Error::<T>::ScoreTooLow);
            ValidatorScores::<T>::insert(&validator, score);
            Self::deposit_event(Event::ScoreSubmitted { validator, score });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::slash_validator())]
        pub fn slash_validator(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            
            let count = SlashingCount::<T>::get(&validator).unwrap_or(0) + 1;
            
            if count >= <T as pallet::Config>::MaxSlashingCount::get() {
                // Remove validator if max slashing count reached
                Validators::<T>::remove(&validator);
                ValidatorScores::<T>::remove(&validator);
                SlashingCount::<T>::remove(&validator);
                Self::deposit_event(Event::ValidatorRemoved { 
                    validator: validator.clone(),
                    reason: b"Max slashing count reached".to_vec(),
                });
            } else {
                SlashingCount::<T>::insert(&validator, count);
            }
            
            Self::deposit_event(Event::ValidatorSlashed { validator, slashing_count: count });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::boost_score())]
        pub fn boost_score(
            origin: OriginFor<T>,
            validator: T::AccountId,
            boost_type: Vec<u8>,
            _amount: u32,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            
            let current_score = ValidatorScores::<T>::get(&validator).unwrap_or(0);
            let max_score = <T as pallet::Config>::MaxValidatorScore::get();
            
            let boost_amount = match boost_type.as_slice() {
                b"block" => <T as pallet::Config>::BlockAuthorshipBoost::get(),
                b"inference" => <T as pallet::Config>::InferenceAccuracyBoost::get(),
                _ => return Err(Error::<T>::InvalidBoostAmount.into()),
            };
            
            let new_score = current_score.saturating_add(boost_amount).min(max_score);
            ValidatorScores::<T>::insert(&validator, new_score);
            
            // Update score history
            let history = ValidatorScoreHistory::<T>::get(&validator);
            let mut new_history = history.clone();
            new_history.try_push((pallet_cbc_dcf::Pallet::<T>::current_epoch(), new_score))
                .map_err(|_| Error::<T>::ScoreHistoryFull)?;
            ValidatorScoreHistory::<T>::insert(&validator, new_history);
            
            Self::deposit_event(Event::ScoreBoosted { 
                validator,
                boost_type,
                amount: boost_amount,
            });
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the top validators based on their scores
        pub fn get_top_validators(count: u32) -> Vec<T::AccountId> {
            let mut validators: Vec<(T::AccountId, u32)> = ValidatorScores::<T>::iter()
                .filter(|(_, score)| *score >= <T as pallet::Config>::MinValidatorScore::get())
                .collect();
                
            // Sort by score in descending order first
            validators.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Then sort by account ID for tiebreaking
            validators.sort_by(|a, b| {
                if a.1 == b.1 {
                    // For tiebreaking, use the raw account ID comparison
                    a.0.cmp(&b.0)
                } else {
                    b.1.cmp(&a.1)
                }
            });
            
            validators
                .into_iter()
                .take(count as usize)
                .map(|(validator, _)| validator)
                .collect()
        }
        
        /// Get the current active validators
        pub fn get_active_validators() -> Vec<T::AccountId> {
            Self::get_top_validators(<T as pallet::Config>::MaxValidators::get())
        }
    }

    impl<T: Config> ValidatorStakeScoreProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn get_validator_stake_score(validator: &Self::AccountId) -> Option<u64> {
            Self::validator_scores(validator).map(|score| score as u64)
        }
    }
}
