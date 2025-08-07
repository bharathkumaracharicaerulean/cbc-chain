//! Performance optimizations for the DCF pallet

use super::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Optimized batch score update for multiple validators
    pub fn batch_update_validator_scores(
        validators: &[<T as frame_system::Config>::AccountId],
        update_pos: bool,
        update_poi: bool,
    ) -> Result<u32, DispatchError> {
        let mut successful = 0u32;
        
        for validator in validators {
            let mut success = true;
            
            if update_pos {
                if Self::update_pos_score_internal(validator).is_err() {
                    success = false;
                }
            }
            
            if update_poi && success {
                if Self::update_poi_score_internal(validator).is_err() {
                    success = false;
                }
            }
            
            if success {
                successful += 1;
                // Note: final score recalculation would be done by the calling code
            }
        }
        
        Ok(successful)
    }
    
    /// Internal optimized PoS score update
    fn update_pos_score_internal(validator: &<T as frame_system::Config>::AccountId) -> Result<(), DispatchError> {
        let stake = pos::Pallet::<T>::stake(validator);
        let stake_score = stake.saturated_into::<u64>();
        
        ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
            let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
            state.current.stake_score = stake_score;
            Ok::<(), Error<T>>(())
        }).map_err(|e| sp_runtime::DispatchError::from(e))?;
        
        Ok(())
    }
    
    /// Internal optimized PoI score update
    fn update_poi_score_internal(validator: &<T as frame_system::Config>::AccountId) -> Result<(), DispatchError> {
        if let Some((result, _)) = poi::Pallet::<T>::inference_results(validator) {
            let inference_score = result as u64;
            ValidatorStates::<T>::try_mutate(validator, |maybe_state| {
                let state = maybe_state.as_mut().ok_or(Error::<T>::ValidatorNotFound)?;
                state.current.inference_score = inference_score;
                Ok::<(), Error<T>>(())
            }).map_err(|e| sp_runtime::DispatchError::from(e))?;
        }
        
        Ok(())
    }
    
    /// Optimized validator ranking with caching
    pub fn get_validator_rankings_cached() -> Vec<(<T as frame_system::Config>::AccountId, u64)> {
        let validators = Self::active_validators();
        let mut rankings = Vec::new();
        
        for validator in validators.iter() {
            if let Some(state) = Self::validator_states(validator) {
                rankings.push((validator.clone(), state.current.final_score));
            }
        }
        
        // Sort by score (descending)
        rankings.sort_by(|a, b| b.1.cmp(&a.1));
        rankings
    }
    
    /// Optimized author selection for block authorship
    pub fn optimized_select_author(block_number: u32) -> Option<<T as frame_system::Config>::AccountId> {
        let active_validators = Self::active_validators();
        if active_validators.is_empty() {
            return None;
        }
        
        // Use round-robin with score weighting for better performance
        let base_index = (block_number as usize) % active_validators.len();
        
        // Try to find the highest scoring validator starting from base_index
        let mut best_validator = &active_validators[base_index];
        let mut best_score = Self::validator_states(best_validator)
            .map(|s| s.current.final_score)
            .unwrap_or(0);
        
        // Check a few validators around the base index for better selection
        let check_range = core::cmp::min(5, active_validators.len());
        for i in 0..check_range {
            let index = (base_index + i) % active_validators.len();
            let validator = &active_validators[index];
            
            if let Some(state) = Self::validator_states(validator) {
                if state.current.final_score > best_score {
                    best_validator = validator;
                    best_score = state.current.final_score;
                }
            }
        }
        
        Some(best_validator.clone())
    }
    
    /// Memory-efficient validator state query
    pub fn get_validator_states_batch(validators: &[<T as frame_system::Config>::AccountId]) -> Vec<Option<ValidatorState>> {
        validators.iter()
            .map(|v| Self::validator_states(v))
            .collect()
    }
}