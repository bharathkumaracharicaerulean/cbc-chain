//! Epoch management implementation
//!
//! This module provides epoch transition logic and validator set management
//! that works in conjunction with the DCF runtime pallet.

use crate::{
    error::{ConsensusError, Result},
    types::{EpochConfig, ValidatorInfo},
};
use std::sync::Arc;
use log::{info, warn, error};
use sp_runtime::traits::{Block as BlockTrait};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use pallet_cbc_dcf::DcfApi as RuntimeDcfApi;
use cbc_runtime::AccountId;

/// Epoch manager for handling epoch transitions and validator set updates
pub struct EpochManager<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    client: Arc<C>,
    epoch_config: EpochConfig,
    _phantom: std::marker::PhantomData<B>,
}

impl<B, C> EpochManager<B, C>
where
    B: BlockTrait,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: RuntimeDcfApi<B, AccountId>,
{
    /// Create a new epoch manager
    pub fn new(client: Arc<C>, epoch_config: EpochConfig) -> Self {
        Self {
            client,
            epoch_config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Check if an epoch transition should occur
    pub fn should_transition_epoch(&self, current_block: u32) -> Result<bool> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let current_epoch = api.get_current_epoch(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get current epoch: {:?}", e)))?;
        
        let runtime_epoch_config = api.get_epoch_config(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get epoch config: {:?}", e)))?;
        
        let blocks_per_epoch = runtime_epoch_config.blocks_per_epoch;
        let epoch_start_block = current_epoch.saturating_mul(blocks_per_epoch);
        let should_transition = current_block >= epoch_start_block + blocks_per_epoch;
        
        if should_transition {
            info!("DCF EpochManager: Epoch transition needed at block {} (epoch {} -> {})", 
                  current_block, current_epoch, current_epoch + 1);
        }
        
        Ok(should_transition)
    }

    /// Handle epoch transition logic
    pub fn handle_epoch_transition(&self, current_block: u32) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let current_epoch = api.get_current_epoch(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get current epoch: {:?}", e)))?;
        
        let next_epoch = current_epoch.saturating_add(1);
        
        info!("DCF EpochManager: Processing epoch transition {} -> {} at block {}", 
              current_epoch, next_epoch, current_block);

        // 1. Apply validator score decay for inactive validators
        self.apply_validator_score_decay(current_epoch)?;

        // 2. Update active validator set based on scores and stake
        self.update_active_validator_set()?;

        // 3. Handle validator join/leave requests
        self.process_validator_set_changes()?;

        // 4. Update validator participation rates
        self.update_validator_participation_rates()?;

        // 5. Check for underperforming validators
        self.handle_underperforming_validators()?;

        info!("DCF EpochManager: Epoch transition completed successfully");
        Ok(())
    }

    /// Apply score decay to inactive validators
    fn apply_validator_score_decay(&self, current_epoch: u32) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let all_validators = api.get_validator_scores(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get validator scores: {:?}", e)))?;
        
        let mut decayed_count = 0;
        
        for (validator, _score) in all_validators {
            let last_active = api.get_validator_last_active(best_hash, validator.clone())
                .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get validator last active: {:?}", e)))?;
            
            let inactive_epochs = current_epoch.saturating_sub(last_active);
            
            if inactive_epochs > 0 {
                // Score decay is handled by the runtime pallet automatically
                // We just log the information here
                info!("DCF EpochManager: Validator {:?} inactive for {} epochs", validator, inactive_epochs);
                decayed_count += 1;
            }
        }
        
        if decayed_count > 0 {
            info!("DCF EpochManager: Applied score decay to {} inactive validators", decayed_count);
        }
        
        Ok(())
    }

    /// Update the active validator set based on current scores and stake
    fn update_active_validator_set(&self) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let current_active = api.get_active_validators(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get active validators: {:?}", e)))?;
        
        let all_scores = api.get_validator_scores(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get validator scores: {:?}", e)))?;
        
        // Filter validators that meet minimum requirements
        let mut eligible_validators: Vec<(AccountId, u64)> = Vec::new();
        
        for (validator, score) in all_scores {
            // Check minimum score requirement
            if score >= 50 { // Minimum score threshold
                // Check if validator has sufficient stake (this would be checked via PoS pallet)
                eligible_validators.push((validator, score));
            }
        }
        
        // Sort by score (descending) and take top validators up to max limit
        eligible_validators.sort_by(|a, b| b.1.cmp(&a.1));
        
        let max_validators = self.epoch_config.max_validators as usize;
        if eligible_validators.len() > max_validators {
            eligible_validators.truncate(max_validators);
        }
        
        let new_active_count = eligible_validators.len();
        let old_active_count = current_active.len();
        
        info!("DCF EpochManager: Validator set update - {} -> {} active validators", 
              old_active_count, new_active_count);
        
        // The actual validator set update is handled by the runtime pallet
        // This is just for monitoring and validation
        
        Ok(())
    }

    /// Process validator join/leave requests
    fn process_validator_set_changes(&self) -> Result<()> {
        // Validator join/leave requests are processed automatically by the runtime pallet
        // during epoch transitions. This function provides additional validation and logging.
        
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get active validators: {:?}", e)))?;
        
        // Validate minimum validator count
        if active_validators.len() < self.epoch_config.min_validators as usize {
            warn!("DCF EpochManager: Active validator count ({}) below minimum ({})", 
                  active_validators.len(), self.epoch_config.min_validators);
        }
        
        // Validate maximum validator count
        if active_validators.len() > self.epoch_config.max_validators as usize {
            error!("DCF EpochManager: Active validator count ({}) exceeds maximum ({})", 
                   active_validators.len(), self.epoch_config.max_validators);
        }
        
        info!("DCF EpochManager: Processed validator set changes - {} active validators", 
              active_validators.len());
        
        Ok(())
    }

    /// Update validator participation rates
    fn update_validator_participation_rates(&self) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get active validators: {:?}", e)))?;
        
        let mut total_participation = 0u32;
        let mut validator_count = 0u32;
        
        for validator in active_validators {
            if let Ok((authored, missed)) = api.get_validator_participation(best_hash, validator.clone()) {
                let total_blocks = authored + missed;
                let participation_rate = if total_blocks > 0 {
                    (authored * 100) / total_blocks
                } else {
                    100 // New validators get 100% initially
                };
                
                total_participation += participation_rate;
                validator_count += 1;
                
                if participation_rate < 80 {
                    warn!("DCF EpochManager: Low participation rate for validator {:?}: {}%", 
                          validator, participation_rate);
                }
            }
        }
        
        let average_participation = if validator_count > 0 {
            total_participation / validator_count
        } else {
            0
        };
        
        info!("DCF EpochManager: Average validator participation rate: {}%", average_participation);
        
        Ok(())
    }

    /// Handle underperforming validators
    fn handle_underperforming_validators(&self) -> Result<()> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get active validators: {:?}", e)))?;
        
        let mut underperforming_count = 0;
        
        for validator in active_validators {
            if let Ok(Some((combined_score, _pos_score, _poi_score, _uptime, _inference_count, participation_rate, missed_blocks))) = 
                api.get_validator_profile(best_hash, validator.clone()) {
                
                // Check for underperformance criteria
                let is_underperforming = combined_score < 30 || participation_rate < 50 || missed_blocks > 10;
                
                if is_underperforming {
                    warn!("DCF EpochManager: Underperforming validator detected: {:?} - Combined Score: {}, Participation: {}%, Missed: {}", 
                          validator, combined_score, participation_rate, missed_blocks);
                    underperforming_count += 1;
                    
                    // The runtime pallet will handle automatic ejection if score falls below threshold
                }
            }
        }
        
        if underperforming_count > 0 {
            info!("DCF EpochManager: Found {} underperforming validators", underperforming_count);
        }
        
        Ok(())
    }

    /// Get current epoch information
    pub fn get_current_epoch_info(&self) -> Result<(u32, Vec<AccountId>)> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        let current_epoch = api.get_current_epoch(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get current epoch: {:?}", e)))?;
        
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| ConsensusError::EpochTransition(format!("Failed to get active validators: {:?}", e)))?;
        
        Ok((current_epoch, active_validators))
    }

    /// Get validator information for a specific validator
    pub fn get_validator_info(&self, validator: &AccountId) -> Result<Option<ValidatorInfo>> {
        let api = self.client.runtime_api();
        let best_hash = self.client.info().best_hash;
        
        if let Ok(Some((combined_score, pos_score, poi_score, uptime, inference_count, participation_rate, missed_blocks))) = 
            api.get_validator_profile(best_hash, validator.clone()) {
            
            let stake_score = pos_score; // Use the fresh PoS score from the profile
            
            let validator_info = ValidatorInfo {
                account_id: sp_core::sr25519::Public::from_raw(*validator.as_ref()),
                stake: stake_score as u128,
                performance_score: combined_score as u32,
                blocks_produced: uptime, // Using uptime as blocks produced approximation
                blocks_missed: missed_blocks,
            };
            
            Ok(Some(validator_info))
        } else {
            Ok(None)
        }
    }
}