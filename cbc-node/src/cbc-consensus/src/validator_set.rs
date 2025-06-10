//! Validator set management implementation
//! 
//! This module handles the management of validators, including adding, removing,
//! and updating validator information and metrics.

use crate::types::{ValidatorInfo, ValidatorMetrics};
use crate::error::{ConsensusError, Result};
use std::collections::HashMap;
use sp_core::sr25519::Public;
use std::fmt;

/// Manages the set of validators and their associated metrics
#[derive(Clone)]
pub struct ValidatorSet {
    max_validators: u32,
    min_stake: u128,
    validators: HashMap<Public, ValidatorInfo>,
    metrics: ValidatorMetrics,
}

impl fmt::Debug for ValidatorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatorSet")
            .field("max_validators", &self.max_validators)
            .field("min_stake", &self.min_stake)
            .field("validators", &self.validators)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl ValidatorSet {
    /// Create a new validator set with the specified parameters
    pub fn new(max_validators: u32, min_stake: u128) -> Self {
        Self {
            max_validators,
            min_stake,
            validators: HashMap::new(),
            metrics: ValidatorMetrics::default(),
        }
    }

    /// Add a new validator to the set
    pub fn add_validator(&mut self, validator: ValidatorInfo) -> Result<()> {
        if self.validators.len() >= self.max_validators as usize {
            return Err(ConsensusError::ValidatorSet("Maximum number of validators reached".into()));
        }

        if validator.stake < self.min_stake {
            return Err(ConsensusError::ValidatorSet("Insufficient stake".into()));
        }

        self.validators.insert(validator.account_id.clone(), validator);
        Ok(())
    }

    /// Remove a validator from the set
    pub fn remove_validator(&mut self, public_key: &Public) -> Result<()> {
        self.validators.remove(public_key);
        Ok(())
    }

    /// Update an existing validator's information
    pub fn update_validator(&mut self, validator: ValidatorInfo) -> Result<()> {
        if !self.validators.contains_key(&validator.account_id) {
            return Err(ConsensusError::ValidatorSet("Validator not found".into()));
        }

        self.validators.insert(validator.account_id.clone(), validator);
        Ok(())
    }

    /// Get a validator by their public key
    pub fn get_validator(&self, public_key: &Public) -> Option<&ValidatorInfo> {
        self.validators.get(public_key)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&ValidatorInfo> {
        self.validators.values().collect()
    }

    /// Update validator metrics
    pub fn update_metrics(&mut self, public_key: &Public, blocks_produced: u32, blocks_missed: u32) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(public_key) {
            validator.blocks_produced = blocks_produced;
            validator.blocks_missed = blocks_missed;
            Ok(())
        } else {
            Err(ConsensusError::ValidatorSet("Validator not found".into()))
        }
    }

    /// Get the current validator metrics
    pub fn get_metrics(&self) -> &ValidatorMetrics {
        &self.metrics
    }
}