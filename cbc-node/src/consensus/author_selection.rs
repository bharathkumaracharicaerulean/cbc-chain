use crate::consensus::types::{AuthorSelectionMode, ValidatorInfo};
use crate::consensus::error::{ConsensusError, Result};
use std::collections::HashMap;

pub struct AuthorSelection {
    mode: AuthorSelectionMode,
    current_index: usize,
    validators: Vec<ValidatorInfo>,
}

impl AuthorSelection {
    pub fn new(mode: AuthorSelectionMode) -> Self {
        Self {
            mode,
            current_index: 0,
            validators: Vec::new(),
        }
    }

    pub fn update_validators(&mut self, validators: Vec<ValidatorInfo>) {
        self.validators = validators;
        self.current_index = 0;
    }

    pub fn select_author(&mut self, slot: u64) -> Result<&ValidatorInfo> {
        if self.validators.is_empty() {
            return Err(ConsensusError::AuthorSelection("No validators available".into()));
        }

        match self.mode {
            AuthorSelectionMode::PoS => self.select_pos_author(),
            AuthorSelectionMode::Hybrid => self.select_hybrid_author(),
            AuthorSelectionMode::RoundRobin => self.select_round_robin_author(slot),
        }
    }

    fn select_pos_author(&self) -> Result<&ValidatorInfo> {
        self.validators
            .iter()
            .max_by_key(|v| v.stake)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_hybrid_author(&self) -> Result<&ValidatorInfo> {
        self.validators
            .iter()
            .max_by_key(|v| {
                // Combine stake and POI score with weights
                let stake_weight = 0.7;
                let poi_weight = 0.3;
                (v.stake as f64 * stake_weight) + (v.poi_score as f64 * poi_weight)
            })
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    fn select_round_robin_author(&mut self, slot: u64) -> Result<&ValidatorInfo> {
        let index = (slot as usize) % self.validators.len();
        self.validators
            .get(index)
            .ok_or_else(|| ConsensusError::AuthorSelection("No validators available".into()))
    }

    pub fn set_mode(&mut self, mode: AuthorSelectionMode) {
        self.mode = mode;
        self.current_index = 0;
    }
}
