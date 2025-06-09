#![allow(unused_imports)]
//! Validator set management module
//!
//! This module maintains and updates the current active validator set based on
//! runtime scores, stake amounts, and performance metrics.

use crate::types::{ValidatorInfo, EpochConfig, ValidatorSetConfig};
use crate::error::{ConsensusError, ValidatorSetError};
use sp_runtime::traits::Block as BlockT;
use sp_core::ed25519::Public;
use std::collections::{HashMap, HashSet};

/// Validator set
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// Validator set configuration
    config: ValidatorSetConfig,
    /// Active validators
    validators: HashMap<Public, ValidatorInfo>,
    /// Validator account IDs
    validator_keys: HashMap<Public, Public>,
}

impl ValidatorSet {
    /// Create a new validator set
    pub fn new(config: ValidatorSetConfig) -> Self {
        Self {
            config,
            validators: HashMap::new(),
            validator_keys: HashMap::new(),
        }
    }

    /// Add a validator
    pub fn add_validator(&mut self, validator: ValidatorInfo) -> Result<(), ConsensusError> {
        // Check if validator set is full
        if self.validators.len() >= self.config.max_validators as usize {
            return Err(ConsensusError::ValidatorSet(ValidatorSetError::SetFull));
        }

        // Check if validator already exists
        if self.validators.contains_key(&validator.public_key) {
            return Err(ConsensusError::ValidatorSet(ValidatorSetError::AlreadyExists));
        }

        // Check minimum stake
        if validator.stake < self.config.min_stake.into() {
            return Err(ConsensusError::ValidatorSet(ValidatorSetError::InsufficientStake));
        }

        // Add validator
        self.validators.insert(validator.public_key.clone(), validator.clone());
        self.validator_keys.insert(validator.public_key.clone(), validator.public_key);

        Ok(())
    }

    /// Remove a validator
    pub fn remove_validator(&mut self, public_key: &Public) -> Result<(), ConsensusError> {
        if let Some(validator) = self.validators.remove(public_key) {
            self.validator_keys.remove(&validator.public_key);
            Ok(())
        } else {
            Err(ConsensusError::ValidatorSet(ValidatorSetError::NotFound))
        }
    }

    /// Update validator info
    pub fn update_validator(&mut self, validator: ValidatorInfo) -> Result<(), ConsensusError> {
        if !self.validators.contains_key(&validator.public_key) {
            return Err(ConsensusError::ValidatorSet(ValidatorSetError::NotFound));
        }

        self.validators.insert(validator.public_key.clone(), validator);
        Ok(())
    }

    /// Check if validator is active
    pub fn is_active(&self, public_key: &Public) -> bool {
        self.validators.contains_key(public_key)
    }

    /// Get validator info
    pub fn get_validator(&self, public_key: &Public) -> Option<&ValidatorInfo> {
        self.validators.get(public_key)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&ValidatorInfo> {
        self.validators.values().collect()
    }

    /// Get validator set configuration
    pub fn get_config(&self) -> &ValidatorSetConfig {
        &self.config
    }

    /// Update validator set configuration
    pub fn update_config(&mut self, config: ValidatorSetConfig) {
        self.config = config;
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new(ValidatorSetConfig::default())
    }
} 