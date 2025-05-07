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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::{StorageMap, StorageValue}};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;

        /// Minimum score required for a validator to be considered active
        type MinValidatorScore: Get<u32>;
        /// Minimum number of active validators required
        type MinActiveValidators: Get<u32>;
        /// Maximum number of validators allowed
        type MaxValidators: Get<u32>;
        /// Score decay per epoch
        type ValidatorScoreDecay: Get<u32>;
        /// Maximum slashing count before removal
        type MaxSlashingCount: Get<u32>;
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub validators: Vec<T::AccountId>,
        pub validator_scores: Vec<u32>,
        pub current_epoch: u32,
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

            // Initialize current epoch
            CurrentEpoch::<T>::put(self.current_epoch);

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
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn slashing_count)]
    pub type SlashingCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorRegistered { validator: T::AccountId },
        ScoreSubmitted { validator: T::AccountId, score: u32 },
        ValidatorSlashed { validator: T::AccountId, slashing_count: u32 },
        ValidatorRemoved { validator: T::AccountId, reason: Vec<u8> },
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        InvalidScore,
        TooManyValidators,
        ScoreTooLow,
        MaxSlashingCountReached,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_validator())]
        pub fn register_validator(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if we've reached max validators
            ensure!(
                Validators::<T>::iter().count() < T::MaxValidators::get() as usize,
                Error::<T>::TooManyValidators
            );

            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyRegistered);
            Validators::<T>::insert(&who, true);
            Self::deposit_event(Event::ValidatorRegistered { validator: who });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_score())]
        pub fn submit_score(origin: OriginFor<T>, validator: T::AccountId, score: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            ensure!(score >= T::MinValidatorScore::get(), Error::<T>::ScoreTooLow);
            ValidatorScores::<T>::insert(&validator, score);
            Self::deposit_event(Event::ScoreSubmitted { validator, score });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::slash_validator())]
        pub fn slash_validator(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            
            let count = SlashingCount::<T>::get(&validator).unwrap_or(0) + 1;
            
            if count >= T::MaxSlashingCount::get() {
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
    }
}
