//! Author selection implementation for the consensus engine
//! 
//! This module handles the selection of block authors based on different strategies.

use crate::types::{AuthorSelectionMode, ValidatorInfo};
use crate::error::{ConsensusError, Result};

/// Author selection engine that implements different selection strategies
pub struct AuthorSelection {
    mode: AuthorSelectionMode,
    current_index: usize,
    validators: Vec<ValidatorInfo>,
}

impl AuthorSelection {
    /// Create a new author selection engine with the specified mode
    pub fn new(mode: AuthorSelectionMode) -> Self {
        Self {
            mode,
            current_index: 0,
            validators: Vec::new(),
        }
    }

    /// Update the list of validators
    pub fn update_validators(&mut self, validators: Vec<ValidatorInfo>) {
        self.validators = validators;
        self.current_index = 0;
    }

    /// Select the next block author based on the current mode
    pub fn select_author(&mut self, slot: u64) -> Result<&ValidatorInfo> {
        if self.validators.is_empty() {
            return Err(ConsensusError::AuthorSelection("No validators available".into()));
        }

        match self.mode {
            AuthorSelectionMode::RoundRobin => self.select_round_robin_author(slot),
            AuthorSelectionMode::PoS => self.select_pos_author(),
            AuthorSelectionMode::Hybrid => self.select_hybrid_author(),
            AuthorSelectionMode::StakeWeighted => self.select_stake_weighted_author(),
            AuthorSelectionMode::PerformanceBased => self.select_performance_based_author(),
        }
    }

    fn select_pos_author(&self) -> Result<&ValidatorInfo> {
        self.validators
            .iter()
            .max_by_key(|v| v.stake)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_hybrid_author(&self) -> Result<&ValidatorInfo> {
        const STAKE_WEIGHT: f64 = 0.7;
        const PERFORMANCE_WEIGHT: f64 = 0.3;

        self.validators
            .iter()
            .max_by(|a, b| {
                let score_a = (a.stake as f64 * STAKE_WEIGHT) + (a.performance_score as f64 * PERFORMANCE_WEIGHT);
                let score_b = (b.stake as f64 * STAKE_WEIGHT) + (b.performance_score as f64 * PERFORMANCE_WEIGHT);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_round_robin_author(&mut self, slot: u64) -> Result<&ValidatorInfo> {
        let index = (slot as usize) % self.validators.len();
        self.validators
            .get(index)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_stake_weighted_author(&self) -> Result<&ValidatorInfo> {
        self.validators
            .iter()
            .max_by_key(|v| v.stake)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_performance_based_author(&self) -> Result<&ValidatorInfo> {
        self.validators
            .iter()
            .max_by_key(|v| v.performance_score)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    /// Set the author selection mode
    pub fn set_mode(&mut self, mode: AuthorSelectionMode) {
        self.mode = mode;
        self.current_index = 0;
    }
}