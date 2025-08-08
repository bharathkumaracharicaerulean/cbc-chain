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

use sp_std::prelude::*;

// Runtime API declaration
sp_api::decl_runtime_apis! {
    pub trait PosApi<AccountId, Balance> 
    where
        AccountId: codec::Codec + Clone + Eq + sp_std::fmt::Debug,
        Balance: codec::Codec + Clone + Eq + sp_runtime::traits::AtLeast32BitUnsigned,
    {
        fn get_validator_stake(validator: AccountId) -> Balance;
        fn get_validator_score(validator: AccountId) -> u32;
        fn get_active_validators() -> Vec<AccountId>;
        fn get_slashing_count(validator: AccountId) -> u32;
        fn get_slashing_history(validator: AccountId) -> Vec<SlashingEvent<Balance>>;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, storage::types::{StorageMap, StorageValue}, BoundedVec};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;
    use sp_runtime::traits::{AtLeast32BitUnsigned, SaturatedConversion};
    use codec::MaxEncodedLen;

    /// Represents a slashing event with timestamp, reason, and penalty amount
    #[derive(Clone, Encode, Decode, PartialEq, Eq, Debug, scale_info::TypeInfo)]
    pub struct SlashingEvent<Balance> {
        /// Timestamp when the slashing occurred (block number)
        pub timestamp: u32,
        /// Reason for the slashing
        pub reason: Vec<u8>,
        /// Penalty amount
        pub penalty_amount: Balance,
        /// Slashing count after this event
        pub slashing_count: u32,
    }

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
        /// Minimum stake amount required for validators
        type MinStake: Get<BalanceOf<Self>>;
        /// The balance type
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

        #[pallet::constant]
        type MaxSlashingHistory: Get<u32>;
    }

    type BalanceOf<T> = <T as Config>::Balance;

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

    #[pallet::storage]
    #[pallet::getter(fn stake)]
    pub type Stake<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    /// Storage map to track slashing history per validator
    /// Key: AccountId, Value: Vec<SlashingEvent>
    #[pallet::storage]
    #[pallet::getter(fn slashing_history)]
    pub type SlashingHistory<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<SlashingEvent<BalanceOf<T>>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_uptime)]
    pub type ValidatorUptimeHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        EpochId,
        ValidatorUptime<T>,
        ValueQuery
    >;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorUptime<T: Config> {
        blocks_authored: u32,
        blocks_missed: u32,
        last_updated: BlockNumberFor<T>,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorRegistered { validator: T::AccountId },
        ScoreSubmitted { validator: T::AccountId, score: u32 },
        ValidatorSlashed { validator: T::AccountId, slashing_count: u32 },
        ValidatorRemoved { validator: T::AccountId, reason: Vec<u8> },
        StakeBonded { validator: T::AccountId, amount: BalanceOf<T> },
        StakeUnbonded { validator: T::AccountId, amount: BalanceOf<T> },
        /// New event for slashing history tracking
        ValidatorSlashedWithHistory { 
            validator: T::AccountId, 
            slashing_count: u32,
            reason: Vec<u8>,
            penalty_amount: BalanceOf<T>,
            timestamp: u32,
        },
        ValidatorUptimeUpdated {
            validator: T::AccountId,
            epoch: EpochId,
            blocks_authored: u32,
            blocks_missed: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyRegistered,
        ValidatorNotRegistered,
        InvalidScore,
        TooManyValidators,
        ScoreTooLow,
        MaxSlashingCountReached,
        InsufficientStake,
        InvalidStakeAmount,
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
            Self::deposit_event(Event::ValidatorRegistered { validator: who.clone() });
            // --- Telemetry: Validator registered ---
            ::log::info!("[cerulea::pos][prometheus] validator_registered{{validator={:?}}} 1", who);
            // --- Telemetry: Active validator count ---
            let active_count = Validators::<T>::iter().filter(|(_, active)| *active).count();
            ::log::info!("[cerulea::pos][prometheus] active_validators_count{{}} {}", active_count);
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_score())]
        pub fn submit_score(origin: OriginFor<T>, validator: T::AccountId, score: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            ensure!(score >= T::MinValidatorScore::get(), Error::<T>::ScoreTooLow);
            ValidatorScores::<T>::insert(&validator, score);
            Self::deposit_event(Event::ScoreSubmitted { validator: validator.clone(), score });
            // --- Telemetry: Score submitted ---
            ::log::info!("[cerulea::pos][prometheus] score_submitted{{validator={:?}}} {}", validator, score);
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::slash_validator())]
        pub fn slash_validator(origin: OriginFor<T>, validator: T::AccountId, reason: Vec<u8>, penalty_amount: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);
            
            let count = SlashingCount::<T>::get(&validator).unwrap_or(0) + 1;
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Create slashing event
            let slashing_event = SlashingEvent {
                timestamp: current_block.saturated_into(),
                reason: reason.clone(),
                penalty_amount,
                slashing_count: count,
            };
            
            // Update slashing history
            let mut history = SlashingHistory::<T>::get(&validator);
            history.push(slashing_event);
            SlashingHistory::<T>::insert(&validator, history);
            
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
            
            Self::deposit_event(Event::ValidatorSlashed { validator: validator.clone(), slashing_count: count });
            Self::deposit_event(Event::ValidatorSlashedWithHistory { 
                validator: validator.clone(), 
                slashing_count: count,
                reason,
                penalty_amount,
                timestamp: current_block.saturated_into(),
            });
            // --- Telemetry: Validator slashed ---
            ::log::info!("[cerulea::pos][prometheus] validator_slashed{{validator={:?}}} {}", validator, count);
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::bond_stake())]
        pub fn bond_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            ensure!(amount >= T::MinStake::get(), Error::<T>::InsufficientStake);
            ensure!(Validators::<T>::contains_key(&who), Error::<T>::ValidatorNotRegistered);
            
            let current_stake = Stake::<T>::get(&who);
            let new_stake = current_stake.checked_add(&amount).ok_or(Error::<T>::InvalidStakeAmount)?;
            
            Stake::<T>::insert(&who, new_stake);
            Self::deposit_event(Event::StakeBonded { validator: who, amount });
            
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::unbond_stake())]
        pub fn unbond_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&who), Error::<T>::ValidatorNotRegistered);
            
            let current_stake = Stake::<T>::get(&who);
            ensure!(current_stake >= amount, Error::<T>::InsufficientStake);
            
            let new_stake = current_stake.checked_sub(&amount).ok_or(Error::<T>::InvalidStakeAmount)?;
            ensure!(new_stake >= T::MinStake::get(), Error::<T>::InsufficientStake);
            
            Stake::<T>::insert(&who, new_stake);
            Self::deposit_event(Event::StakeUnbonded { validator: who, amount });
            
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::submit_score())] 
        pub fn boost_score(origin: OriginFor<T>, validator: T::AccountId, weight: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);

            let current_score = ValidatorScores::<T>::get(&validator).unwrap_or(0);
            let new_score = current_score.saturating_add(weight);
            ValidatorScores::<T>::insert(&validator, new_score);

            Self::deposit_event(Event::ScoreSubmitted { validator, score: new_score });
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::submit_score())] 
        pub fn slash_score(origin: OriginFor<T>, validator: T::AccountId, weight: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(Validators::<T>::contains_key(&validator), Error::<T>::ValidatorNotRegistered);

            let current_score = ValidatorScores::<T>::get(&validator).unwrap_or(0);
            let new_score = current_score.saturating_sub(weight);
            ValidatorScores::<T>::insert(&validator, new_score);

            Self::deposit_event(Event::ScoreSubmitted { validator, score: new_score });
            Ok(())
        }

        pub fn record_block_authored(validator: &T::AccountId) -> DispatchResult {
            let current_epoch = Self::current_epoch();
            ValidatorUptimeHistory::<T>::try_mutate(validator, current_epoch, |uptime| {
                uptime.blocks_authored = uptime.blocks_authored.saturating_add(1);
                uptime.last_updated = frame_system::Pallet::<T>::block_number();
                Ok(())
            })
        }

        pub fn record_block_missed(validator: &T::AccountId) -> DispatchResult {
            let current_epoch = Self::current_epoch();
            ValidatorUptimeHistory::<T>::try_mutate(validator, current_epoch, |uptime| {
                uptime.blocks_missed = uptime.blocks_missed.saturating_add(1);
                uptime.last_updated = frame_system::Pallet::<T>::block_number();
                Ok(())
            })
        }

        fn on_epoch_ending(epoch: EpochId) {
            // Emit events for all validators' uptime
            for validator in Self::active_validators() {
                if let Some(uptime) = Self::validator_uptime(&validator, epoch) {
                    Self::deposit_event(Event::ValidatorUptimeUpdated {
                        validator: validator.clone(),
                        epoch,
                        blocks_authored: uptime.blocks_authored,
                        blocks_missed: uptime.blocks_missed,
                    });
                }
            }
        }
    }

    impl<T: Config> ScoreProvider<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn get_score(validator: &T::AccountId) -> BalanceOf<T> {
            Self::validator_stake(validator)
        }
    }

    impl<T: Config> ScoreProvider<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn get_score(validator: &T::AccountId) -> BalanceOf<T> {
            Self::validator_inference_score(validator)
        }
    }
}
