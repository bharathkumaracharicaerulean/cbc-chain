#![allow(unused_imports)]
//! Author selection module
//!
//! This module implements the validator selection logic for the CBC consensus,
//! combining Proof of Stake (PoS) and Proof of Inference (PoI) scores to select
//! the next block author.

use crate::types::{ValidatorInfo, AuthorSelectionConfig, AuthorSelectionCriteria};
use crate::error::{ConsensusError, AuthorSelectionError};
use sp_core::ed25519::Public;
use std::collections::HashMap;

/// Author selection service
pub struct AuthorSelection {
    /// Author selection configuration
    config: AuthorSelectionConfig,
    /// Validators
    validators: Vec<ValidatorInfo>,
    /// Validators in cooldown
    cooldown: HashMap<Public, u32>,
}

impl AuthorSelection {
    /// Create a new author selection service
    pub fn new(config: AuthorSelectionConfig) -> Self {
        Self {
            config,
            validators: Vec::new(),
            cooldown: HashMap::new(),
        }
    }

    /// Set validators
    pub fn set_validators(&mut self, validators: Vec<ValidatorInfo>) {
        self.validators = validators;
    }

    /// Select next block author
    pub fn select_author(&self) -> Result<&ValidatorInfo, ConsensusError> {
        // Filter eligible validators
        let eligible: Vec<_> = self.validators
            .iter()
            .filter(|v| self.is_eligible(v))
            .collect();

        if eligible.is_empty() {
            return Err(ConsensusError::AuthorSelection(AuthorSelectionError::NoValidators));
        }

        // Select author based on criteria
        match self.config.criteria {
            AuthorSelectionCriteria::ProofOfStake => {
                eligible.iter()
                    .max_by_key(|v| v.stake)
                    .map(|v| *v)
                    .ok_or_else(|| ConsensusError::AuthorSelection(AuthorSelectionError::NoValidators))
            }
            AuthorSelectionCriteria::ProofOfInference => {
                eligible.iter()
                    .max_by_key(|v| v.metrics.blocks_produced)
                    .map(|v| *v)
                    .ok_or_else(|| ConsensusError::AuthorSelection(AuthorSelectionError::NoValidators))
            }
            AuthorSelectionCriteria::Hybrid => {
                eligible.iter()
                    .max_by_key(|v| {
                        let stake_score = v.stake as u128;
                        let performance_score = v.metrics.blocks_produced as u128;
                        stake_score + performance_score
                    })
                    .map(|v| *v)
                    .ok_or_else(|| ConsensusError::AuthorSelection(AuthorSelectionError::NoValidators))
            }
        }
    }

    /// Check if validator is eligible
    fn is_eligible(&self, validator: &ValidatorInfo) -> bool {
        // Check if validator is in cooldown
        if self.cooldown.contains_key(&validator.public_key) {
            return false;
        }

        // Check eligibility based on criteria
        match self.config.criteria {
            AuthorSelectionCriteria::ProofOfStake => {
                validator.stake >= self.config.min_stake
            }
            AuthorSelectionCriteria::ProofOfInference => {
                validator.metrics.blocks_produced > 0
            }
            AuthorSelectionCriteria::Hybrid => {
                validator.stake >= self.config.min_stake &&
                validator.metrics.blocks_produced > 0
            }
        }
    }

    /// Add validator to cooldown
    pub fn add_to_cooldown(&mut self, public_key: Public) {
        self.cooldown.insert(public_key, self.config.cooldown_period);
    }

    /// Update cooldown periods
    pub fn update_cooldown(&mut self) {
        self.cooldown.retain(|_, count| {
            let _ = *count > 0;
            *count -= 1;
            *count > 0
        });
    }

    /// Get author selection configuration
    pub fn get_config(&self) -> &AuthorSelectionConfig {
        &self.config
    }

    /// Update author selection configuration
    pub fn update_config(&mut self, config: AuthorSelectionConfig) {
        self.config = config;
    }
}

impl Default for AuthorSelection {
    fn default() -> Self {
        Self::new(AuthorSelectionConfig {
            criteria: AuthorSelectionCriteria::Hybrid,
            min_stake: 1000,
            cooldown_period: 10,
        })
    }
} 