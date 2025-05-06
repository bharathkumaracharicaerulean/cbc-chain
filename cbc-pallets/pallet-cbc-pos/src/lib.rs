#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::{StorageMap, StorageValue}};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

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
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        InvalidScore,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_validator())]
        pub fn register_validator(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::ValidatorAlreadyRegistered);
            Validators::<T>::insert(&who, ());
            Self::deposit_event(Event::ValidatorRegistered { validator: who });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_score())]
        pub fn submit_score(origin: OriginFor<T>, validator: T::AccountId, score: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            ensure!(score > 0, Error::<T>::InvalidScore);
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
            SlashingCount::<T>::insert(&validator, count);
            Self::deposit_event(Event::ValidatorSlashed { validator, slashing_count: count });
            Ok(())
        }
    }
}
