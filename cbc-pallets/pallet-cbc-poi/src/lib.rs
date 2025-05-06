#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::StorageMap}; // Corrected import
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo; // Ensure WeightInfo is used
    }

    /// Storage for inference result submissions from validators.
    #[pallet::storage]
    #[pallet::getter(fn inference_results)]
    pub type InferenceResults<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        u32, // Example: u32 represents the inference result.
        OptionQuery
    >;

    /// Storage for challenge submissions (if a validator disputes an inference).
    #[pallet::storage]
    #[pallet::getter(fn challenges)]
    pub type Challenges<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        (T::AccountId, u32), // Challenger, disputed inference result.
        OptionQuery
    >;

    /// Storage for the current inference epoch or round.
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An inference result was submitted. [who, result]
        InferenceSubmitted { who: T::AccountId, result: u32 },
        /// An inference result was challenged. [challenger, challenged, result]
        InferenceChallenged { challenger: T::AccountId, challenged: T::AccountId, result: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The inference result already exists.
        InferenceAlreadySubmitted,
        /// The inference result does not exist.
        InferenceNotFound,
        /// The challenge is invalid.
        InvalidChallenge,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit an inference result.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_inference())] // Updated weight function
        pub fn submit_inference(
            origin: OriginFor<T>,
            result: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure the inference result is not already submitted.
            ensure!(
                !InferenceResults::<T>::contains_key(&who),
                Error::<T>::InferenceAlreadySubmitted
            );

            // Store the inference result.
            InferenceResults::<T>::insert(&who, result);

            // Emit an event.
            Self::deposit_event(Event::InferenceSubmitted { who, result });

            Ok(())
        }

        /// Submit a challenge against an inference result.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::challenge_inference())] // Updated weight function
        pub fn challenge_inference(
            origin: OriginFor<T>,
            challenged: T::AccountId,
            result: u32,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;

            // Ensure the inference result exists.
            ensure!(
                InferenceResults::<T>::get(&challenged) == Some(result),
                Error::<T>::InferenceNotFound
            );

            // Store the challenge.
            Challenges::<T>::insert(&challenger, (&challenged, result));

            // Emit an event.
            Self::deposit_event(Event::InferenceChallenged {
                challenger,
                challenged,
                result,
            });

            Ok(())
        }
    }
}
