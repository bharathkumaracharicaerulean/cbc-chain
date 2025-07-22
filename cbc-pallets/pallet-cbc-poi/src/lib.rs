#![cfg_attr(not(feature = "std"), no_std)]
<<<<<<< HEAD
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
=======
//! # pallet-cbc-poi
//!
//! This pallet implements Proof-of-Inference (PoI) logic for the CBC-Chain. It allows validators to submit inference results, challenge others' results, and provides a mechanism for rewarding or penalizing based on inference correctness and challenge outcomes.
//!
//! ## Main Features
//! - **Inference Submission:** Validators submit inference results with a confidence score.
//! - **Challenge Mechanism:** Validators can challenge others' inference results within a configurable window.
//! - **Epoch Management:** Inference results and challenges are tracked per epoch.
//! - **Configurable Parameters:** Confidence threshold, challenge window, and rewards are all configurable via the runtime.
//! - **Events:** Emits events for inference submissions, challenges, and challenge resolutions.

pub use pallet::*;

// --- Weights Module --- //
pub mod weights;
pub use weights::*;

// --- Test Modules --- //
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// --- Benchmarking (if enabled) --- //
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// --- Imports --- //
use sp_std::prelude::*;

// --- Runtime API Declaration --- //
// Exposes PoI state and queries to the runtime API.
sp_api::decl_runtime_apis! {
    pub trait PoiApi<AccountId>
    where
        AccountId: codec::Codec + Clone + Eq + sp_std::fmt::Debug,
    {
        /// Get the inference result and epoch for a validator.
        fn get_inference_result(validator: AccountId) -> Option<(u32, u32)>;
        /// Get the challenge (challenger, result, epoch) for a validator.
        fn get_challenge(validator: AccountId) -> Option<(AccountId, u32, u32)>;
        /// Get the current inference epoch.
        fn get_current_epoch() -> u32;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::StorageMap};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;

    /// Main pallet struct.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Weight information for extrinsics.
        type WeightInfo: WeightInfo;

        /// Minimum confidence threshold for inference (0-100).
        type MinInferenceConfidence: Get<u32>;
        /// Maximum age of inference in epochs.
        type MaxInferenceAge: Get<u32>;
        /// Number of epochs to challenge an inference.
        type ChallengeWindow: Get<u32>;
        /// Reward for correct inference.
        type InferenceReward: Get<u128>;
        /// Reward for successful challenge.
        type ChallengeReward: Get<u128>;
        /// Interface to PoS pallet for boosting/slashing scores
        type PosInterface: PosInterface<Self::AccountId>;
    }

    /// Trait for PoS score manipulation (to be implemented by PoS pallet or runtime)
    pub trait PosInterface<AccountId> {
        fn boost_score(validator: &AccountId, weight: u32) -> DispatchResult;
        fn slash_score(validator: &AccountId, weight: u32) -> DispatchResult;
    }

    /// Genesis configuration for PoI pallet.
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Initial inference results: (validator, result).
        pub inference_results: Vec<(T::AccountId, u32)>,
        /// Initial challenges: (challenger, challenged, result).
        pub challenges: Vec<(T::AccountId, T::AccountId, u32)>,
        /// Initial epoch number.
        pub current_epoch: u32,
    }

    /// Genesis build logic for initializing storage.
    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Initialize inference results for validators.
            for (account, result) in &self.inference_results {
                InferenceResults::<T>::insert(account, (*result, self.current_epoch));
            }

            // Initialize challenges.
            for (challenger, challenged, result) in &self.challenges {
                Challenges::<T>::insert(challenger, (challenged, *result, self.current_epoch));
            }

            // Set the current epoch.
            CurrentEpoch::<T>::put(self.current_epoch);
        }
    }

    /// Storage for inference result submissions from validators.
    /// Maps validator account to (result, epoch).
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    #[pallet::storage]
    #[pallet::getter(fn inference_results)]
    pub type InferenceResults<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
<<<<<<< HEAD
        u32, // Example: u32 represents the inference result.
        OptionQuery
    >;

    /// Storage for challenge submissions (if a validator disputes an inference).
=======
        (u32, u32), // (result, epoch)
        OptionQuery
    >;

    /// Storage for challenge submissions.
    /// Maps challenger account to (challenged, result, epoch).
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
    #[pallet::storage]
    #[pallet::getter(fn challenges)]
    pub type Challenges<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        T::AccountId, 
<<<<<<< HEAD
        (T::AccountId, u32), // Challenger, disputed inference result.
=======
        (T::AccountId, u32, u32), // (challenged, result, epoch)
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
        OptionQuery
    >;

    /// Storage for the current inference epoch or round.
    #[pallet::storage]
    #[pallet::getter(fn current_epoch)]
    pub type CurrentEpoch<T: Config> = StorageValue<_, u32, ValueQuery>;

<<<<<<< HEAD
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
=======
    /// Events emitted by the PoI pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An inference result was submitted. [who, result, confidence]
        InferenceSubmitted { who: T::AccountId, result: u32, confidence: u32 },
        /// An inference result was accepted. [validator, confidence]
        InferenceAccepted { validator: T::AccountId, confidence: u32 },
        /// An inference result was rejected. [validator, confidence]
        InferenceRejected { validator: T::AccountId, confidence: u32 },
        /// An inference result was challenged. [challenger, challenged, result]
        InferenceChallenged { challenger: T::AccountId, challenged: T::AccountId, result: u32 },
        /// A challenge was resolved. [challenger, challenged, result, success]
        ChallengeResolved { challenger: T::AccountId, challenged: T::AccountId, result: u32, success: bool },
        /// A validator was slashed. [validator, reason]
        ValidatorSlashed { validator: T::AccountId, reason: Vec<u8> },
    }

    /// Errors returned by the PoI pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// The inference result already exists for this validator.
        InferenceAlreadySubmitted,
        /// The inference result does not exist.
        InferenceNotFound,
        /// The challenge is invalid (e.g., wrong result).
        InvalidChallenge,
        /// The confidence level is too low.
        ConfidenceTooLow,
        /// The challenge window has expired.
        ChallengeWindowExpired,
        /// The inference is too old to be challenged.
        InferenceTooOld,
    }

    /// Dispatchable functions (extrinsics) for the PoI pallet.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit an inference result for the current epoch.
        ///
        /// - `result`: The inference result value.
        /// - `confidence`: Confidence score (must meet minimum threshold).
        ///
        /// Emits `InferenceSubmitted` event.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_inference())]
        pub fn submit_inference(
            origin: OriginFor<T>,
            result: u32,
            confidence: u32,
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure the inference result is not already submitted.
            ensure!(
                !InferenceResults::<T>::contains_key(&who),
                Error::<T>::InferenceAlreadySubmitted
            );

<<<<<<< HEAD
            // Store the inference result.
            InferenceResults::<T>::insert(&who, result);

            // Emit an event.
            Self::deposit_event(Event::InferenceSubmitted { who, result });

            Ok(())
        }

        /// Submit a challenge against an inference result.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::challenge_inference())] // Updated weight function
=======
            // Ensure confidence meets minimum threshold.
            ensure!(
                confidence >= T::MinInferenceConfidence::get(),
                Error::<T>::ConfidenceTooLow
            );

            let current_epoch = CurrentEpoch::<T>::get();

            // Store the inference result with current epoch.
            InferenceResults::<T>::insert(&who, (result, current_epoch));

            // Emit an event.
            Self::deposit_event(Event::InferenceSubmitted { who: who.clone(), result, confidence });
            // --- Telemetry: Inference submitted ---
            ::log::info!("[cerulea::poi][prometheus] inference_submitted{{validator={:?}}} {{result={},confidence={}}}", who, result, confidence);

            // --- PoS boost logic ---
            let boost = if confidence >= 90 {
                10
            } else if confidence >= 70 {
                5
            } else {
                2
            };
            if let Err(e) = T::PosInterface::boost_score(&who, boost) {
                ::log::warn!("[cerulea::poi][prometheus] boost_score_failed{{validator={:?}}} {:?}", who, e);
            } else {
                Self::deposit_event(Event::InferenceAccepted { validator: who.clone(), confidence });
                // --- Telemetry: Inference accepted ---
                ::log::info!("[cerulea::poi][prometheus] inference_accepted{{validator={:?}}} {{confidence={}}}", who, confidence);
            }
            Ok(())
        }

        /// Submit a challenge against another validator's inference result.
        ///
        /// - `challenged`: The validator being challenged.
        /// - `result`: The result being challenged.
        ///
        /// Emits `InferenceChallenged` event.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::challenge_inference())]
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
        pub fn challenge_inference(
            origin: OriginFor<T>,
            challenged: T::AccountId,
            result: u32,
        ) -> DispatchResult {
            let challenger = ensure_signed(origin)?;

<<<<<<< HEAD
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
=======
            // Get the inference result and its epoch.
            let (stored_result, epoch) = InferenceResults::<T>::get(&challenged)
                .ok_or(Error::<T>::InferenceNotFound)?;

            let current_epoch = CurrentEpoch::<T>::get();

            // Ensure the inference is not too old.
            ensure!(
                current_epoch - epoch <= T::MaxInferenceAge::get(),
                Error::<T>::InferenceTooOld
            );

            // Ensure we're within the challenge window.
            ensure!(
                current_epoch - epoch <= T::ChallengeWindow::get(),
                Error::<T>::ChallengeWindowExpired
            );

            // Ensure the inference result matches.
            ensure!(
                stored_result == result,
                Error::<T>::InvalidChallenge
            );

            // Store the challenge with current epoch.
            Challenges::<T>::insert(&challenger, (&challenged, result, current_epoch));

            // Emit an event.
            Self::deposit_event(Event::InferenceChallenged {
                challenger: challenger.clone(),
                challenged: challenged.clone(),
                result,
            });
            // --- Telemetry: Inference challenged ---
            ::log::info!("[cerulea::poi][prometheus] inference_challenged{{challenger={:?},challenged={:?}}} {{result={}}}", challenger, challenged, result);

            // --- PoS slash logic ---
            let slash = 7; // Example: fixed penalty, could be parameterized
            if let Err(e) = T::PosInterface::slash_score(&challenged, slash) {
                ::log::warn!("[cerulea::poi][prometheus] slash_score_failed{{validator={:?}}} {:?}", challenged, e);
            } else {
                Self::deposit_event(Event::InferenceRejected { validator: challenged.clone(), confidence: 0 });
                Self::deposit_event(Event::ValidatorSlashed { validator: challenged.clone(), reason: b"Invalid inference".to_vec() });
                // --- Telemetry: Inference rejected and validator slashed ---
                ::log::info!("[cerulea::poi][prometheus] inference_rejected{{validator={:?}}} 1", challenged);
                ::log::info!("[cerulea::poi][prometheus] validator_slashed{{validator={:?}}} 1", challenged);
            }
            Ok(())
        }
    }
}
>>>>>>> c0e1c816d4065ea122aac1de1ee507dc1010eacc
