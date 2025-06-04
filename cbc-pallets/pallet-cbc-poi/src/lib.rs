#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod weights;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::StorageMap};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;
    use pallet_cbc_dcf::ValidatorScoreProvider;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_cbc_dcf::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;

        /// Minimum confidence threshold for inference (0-100)
        type MinInferenceConfidence: Get<u32>;
        /// Maximum age of inference in epochs
        type MaxInferenceAge: Get<u32>;
        /// Number of epochs to challenge an inference
        type ChallengeWindow: Get<u32>;
        /// Reward for correct inference
        type InferenceReward: Get<u128>;
        /// Reward for successful challenge
        type ChallengeReward: Get<u128>;
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub inference_results: Vec<(T::AccountId, u32)>,
        pub challenges: Vec<(T::AccountId, T::AccountId, u32)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Initialize inference results
            for (account, result) in &self.inference_results {
                InferenceResults::<T>::insert(account, (*result, 0)); // Start with epoch 0
            }

            // Initialize challenges
            for (challenger, challenged, result) in &self.challenges {
                Challenges::<T>::insert(challenger, (challenged, *result, 0)); // Start with epoch 0
            }
        }
    }

    /// Storage for inference result submissions from validators.
    #[pallet::storage]
    #[pallet::getter(fn inference_results)]
    pub type InferenceResults<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        (u32, u32), // (result, epoch)
        OptionQuery
    >;

    /// Storage for challenge submissions (if a validator disputes an inference).
    #[pallet::storage]
    #[pallet::getter(fn challenges)]
    pub type Challenges<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
        (T::AccountId, u32, u32), // (challenged, result, epoch)
        OptionQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An inference result was submitted. [who, result, confidence]
        InferenceSubmitted { who: T::AccountId, result: u32, confidence: u32 },
        /// An inference result was challenged. [challenger, challenged, result]
        InferenceChallenged { challenger: T::AccountId, challenged: T::AccountId, result: u32 },
        /// A challenge was resolved. [challenger, challenged, result, success]
        ChallengeResolved { challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The inference result already exists.
        InferenceAlreadySubmitted,
        /// The inference result does not exist.
        InferenceNotFound,
        /// The challenge is invalid.
        InvalidChallenge,
        /// The confidence level is too low.
        ConfidenceTooLow,
        /// The challenge window has expired.
        ChallengeWindowExpired,
        /// The inference is too old.
        InferenceTooOld,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit an inference result.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_inference())]
        pub fn submit_inference(
            origin: OriginFor<T>,
            result: u32,
            confidence: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure the inference result is not already submitted.
            ensure!(
                !InferenceResults::<T>::contains_key(&who),
                Error::<T>::InferenceAlreadySubmitted
            );

            // Ensure confidence meets minimum threshold
            ensure!(
                confidence >= T::MinInferenceConfidence::get(),
                Error::<T>::ConfidenceTooLow
            );

            let current_epoch = pallet_cbc_dcf::Pallet::<T>::current_epoch();

            // Store the inference result with current epoch
            InferenceResults::<T>::insert(&who, (result, current_epoch));

            // Emit an event.
            Self::deposit_event(Event::InferenceSubmitted { who, result, confidence });

            Ok(())
        }

        /// Submit a challenge against an inference result.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::challenge_inference())]
        pub fn challenge_inference(
            origin: OriginFor<T>,
            challenged: T::AccountId,
            result: u32,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;

            // Get the inference result and its epoch
            let (stored_result, epoch) = InferenceResults::<T>::get(&challenged)
                .ok_or(Error::<T>::InferenceNotFound)?;

            let current_epoch = pallet_cbc_dcf::Pallet::<T>::current_epoch();

            // Ensure the inference is not too old
            ensure!(
                current_epoch.saturating_sub(epoch) <= T::MaxInferenceAge::get(),
                Error::<T>::InferenceTooOld
            );

            // Ensure we're within the challenge window
            ensure!(
                current_epoch.saturating_sub(epoch) <= T::ChallengeWindow::get(),
                Error::<T>::ChallengeWindowExpired
            );

            // Ensure the inference result matches
            ensure!(
                stored_result == result,
                Error::<T>::InvalidChallenge
            );

            // Store the challenge with current epoch
            Challenges::<T>::insert(&challenger, (&challenged, result, current_epoch));

            // Emit an event.
            Self::deposit_event(Event::InferenceChallenged {
                challenger,
                challenged,
                result,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate a validator's score based on their inference results and challenges
        pub fn calculate_validator_score(validator: &T::AccountId) -> Option<u32> {
            // Get the inference result
            let (score, epoch) = Self::inference_results(validator)?;
            
            // Get current epoch
            let current_epoch = pallet_cbc_dcf::Pallet::<T>::current_epoch();
            
            // Check if the inference is too old
            let max_age = T::MaxInferenceAge::get();
            let epoch_diff = current_epoch.saturating_sub(epoch);
            
            if epoch_diff > max_age {
                return None;
            }
            
            // Count challenges
            let mut challenge_count = 0;
            for (_challenger, (challenged, _result, _epoch)) in Challenges::<T>::iter() {
                if challenged == *validator {
                    challenge_count += 1;
                }
            }
            
            // Adjust score based on challenges
            let adjusted_score = score.saturating_sub(challenge_count * 10);
            
            Some(adjusted_score)
        }
    }

    impl<T: Config> ValidatorScoreProvider for Pallet<T> {
        type AccountId = T::AccountId;

        fn get_validator_score(validator: &Self::AccountId) -> Option<u64> {
            Self::calculate_validator_score(validator).map(|score| score as u64)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            // Any cleanup needed at the end of the block
        }
    }
}
