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
    use frame_support::sp_runtime::traits::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
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
        pub current_epoch: u32,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Initialize inference results
            for (account, result) in &self.inference_results {
                InferenceResults::<T>::insert(account, (*result, self.current_epoch));
            }

            // Initialize challenges
            for (challenger, challenged, result) in &self.challenges {
                Challenges::<T>::insert(challenger, (challenged, *result, self.current_epoch));
            }

            // Initialize current epoch
            CurrentEpoch::<T>::put(self.current_epoch);
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

    /// Storage for the current inference epoch or round.
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An inference result was submitted. [who, result, confidence]
        InferenceSubmitted { who: T::AccountId, result: u32, confidence: u32 },
        /// An inference result was challenged. [challenger, challenged, result]
        InferenceChallenged { challenger: T::AccountId, challenged: T::AccountId, result: u32 },
        /// A challenge was resolved. [challenger, challenged, result, success]
        ChallengeResolved { challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool },
        /// A new epoch has started. [epoch_number]
        EpochTransitioned { epoch_number: u32 },
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
        #[pallet::weight(T::WeightInfo::submit_inference())]
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

            let current_epoch = CurrentEpoch::<T>::get();

            // Store the inference result with current epoch
            InferenceResults::<T>::insert(&who, (result, current_epoch));

            // Emit an event.
            Self::deposit_event(Event::InferenceSubmitted { who, result, confidence });

            Ok(())
        }

        /// Submit a challenge against an inference result.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::challenge_inference())]
        pub fn challenge_inference(
            origin: OriginFor<T>,
            challenged: T::AccountId,
            result: u32,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;

            // Get the inference result and its epoch
            let (stored_result, epoch) = InferenceResults::<T>::get(&challenged)
                .ok_or(Error::<T>::InferenceNotFound)?;

            let current_epoch = CurrentEpoch::<T>::get();

            // Ensure the inference is not too old
            ensure!(
                current_epoch - epoch <= T::MaxInferenceAge::get(),
                Error::<T>::InferenceTooOld
            );

            // Ensure we're within the challenge window
            ensure!(
                current_epoch - epoch <= T::ChallengeWindow::get(),
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
            let (score, block_number) = Self::inference_results(validator)?;
            
            // Get current block number
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Check if the inference is too old
            let max_age = T::MaxInferenceAge::get();
            let block_diff = current_block.saturating_sub(block_number.into());
            
            if block_diff > max_age.into() {
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

        /// Handle epoch transition
        fn handle_epoch_transition() {
            let current_epoch = CurrentEpoch::<T>::get();
            let new_epoch = current_epoch + 1;
            
            // Update epoch counter
            CurrentEpoch::<T>::put(new_epoch);

            // Emit epoch transition event
            Self::deposit_event(Event::EpochTransitioned { epoch_number: new_epoch });

            // Clear old inference results and challenges
            let max_age = T::MaxInferenceAge::get();
            
            // Clear old inference results
            for (account, (_, epoch)) in InferenceResults::<T>::iter() {
                if current_epoch - epoch > max_age {
                    InferenceResults::<T>::remove(account);
                }
            }

            // Clear old challenges
            for (challenger, (_, _, epoch)) in Challenges::<T>::iter() {
                if current_epoch - epoch > max_age {
                    Challenges::<T>::remove(challenger);
                }
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // Check if we need to transition to a new epoch (every 10 blocks)
            if n % 10u32.into() == 0u32.into() {
                Self::handle_epoch_transition();
            }
            Weight::zero()
        }

        fn on_finalize(_n: BlockNumberFor<T>) {
            // Any cleanup needed at the end of the block
        }
    }
}
