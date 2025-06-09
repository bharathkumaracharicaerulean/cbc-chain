#![allow(unused_imports)]
//! Epoch management module
//!
//! This module manages epoch boundaries, validator set rotations, and epoch transitions
//! in the consensus system.

use sp_runtime::traits::Block as BlockT;
use sp_core::ed25519::Public;
use std::collections::HashMap;
use crate::types::{EpochConfig, ValidatorInfo, ValidatorSetConfig};
use crate::error::{ConsensusError, EpochTransitionError};
use crate::validator_set::ValidatorSet;

/// Information about a validator epoch
pub struct EpochInfo {
    /// Epoch number
    pub number: u32,
    /// Validators in this epoch
    pub validators: Vec<ValidatorInfo>,
    /// Start block number
    pub start_block: u32,
    /// End block number
    pub end_block: u32,
}

/// Manages epoch transitions and validator sets
pub struct EpochManager {
    /// Current epoch information
    pub current_epoch: EpochInfo,
    /// The validator set
    pub validator_set: ValidatorSet,
    /// Validator set configuration
    pub config: ValidatorSetConfig,
}

impl EpochManager {
    /// Create a new EpochManager
    pub fn new(config: ValidatorSetConfig, validators: Vec<ValidatorInfo>) -> Self {
        let validator_set = ValidatorSet::new(config.clone());
        let current_epoch = EpochInfo {
            number: 0,
            validators,
            start_block: 0,
            end_block: 0,
        };
        Self { current_epoch, validator_set, config }
    }

    /// Check if a block is in the current epoch
    pub fn is_in_current_epoch(&self, block_number: u32) -> bool {
        block_number >= self.current_epoch.start_block && block_number < self.current_epoch.end_block
    }

    /// Check if a block is at the epoch boundary
    pub fn is_epoch_boundary(&self, block_number: u32) -> bool {
        block_number == self.current_epoch.end_block
    }

    /// Handle an epoch transition
    pub fn handle_epoch_transition(&mut self, block_number: u32) -> Result<(), ConsensusError> {
        if !self.is_epoch_boundary(block_number) {
            return Err(ConsensusError::EpochTransition(
                EpochTransitionError::InvalidEpochNumber
            ));
        }

        // Update epochs
        self.current_epoch = EpochInfo {
            number: self.current_epoch.number + 1,
            start_block: self.current_epoch.end_block,
            end_block: self.current_epoch.end_block + self.config.blocks_per_epoch,
            validators: self.validator_set.get_active_validators().into_iter().cloned().collect(),
        };

        Ok(())
    }

    /// Get the current epoch
    pub fn get_current_epoch(&self) -> &EpochInfo {
        &self.current_epoch
    }

    /// Get the validator set configuration
    pub fn get_config(&self) -> &ValidatorSetConfig {
        &self.config
    }
} 