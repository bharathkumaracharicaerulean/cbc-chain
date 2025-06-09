use crate::consensus::types::{EpochConfig, ValidatorInfo};
use crate::consensus::error::{ConsensusError, Result};
use crate::consensus::validator_set::ValidatorSet;
use std::collections::HashMap;

pub struct EpochManager {
    config: EpochConfig,
    current_epoch: u32,
    current_slot: u32,
    validator_set: ValidatorSet,
    validator_scores: HashMap<Vec<u8>, u32>,
}

impl EpochManager {
    pub fn new(config: EpochConfig) -> Self {
        Self {
            config,
            current_epoch: 0,
            current_slot: 0,
            validator_set: ValidatorSet::new(config.max_validators, config.min_stake),
            validator_scores: HashMap::new(),
        }
    }

    pub fn advance_slot(&mut self) -> Result<bool> {
        self.current_slot += 1;
        
        if self.current_slot >= self.config.slots_per_epoch {
            self.transition_epoch()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn transition_epoch(&mut self) -> Result<()> {
        // Update validator scores based on performance
        self.update_validator_scores()?;

        // Select top validators for next epoch
        let mut validators: Vec<ValidatorInfo> = self.validator_scores
            .iter()
            .map(|(pubkey, score)| {
                let mut validator = self.validator_set
                    .get_validator(pubkey)
                    .cloned()
                    .unwrap_or_else(|| ValidatorInfo {
                        public_key: pubkey.clone(),
                        stake: 0,
                        poi_score: 0,
                        blocks_produced: 0,
                        blocks_missed: 0,
                    });
                validator.poi_score = *score;
                validator
            })
            .collect();

        // Sort by combined score (stake + POI)
        validators.sort_by(|a, b| {
            let score_a = (a.stake as f64 * 0.7) + (a.poi_score as f64 * 0.3);
            let score_b = (b.stake as f64 * 0.7) + (b.poi_score as f64 * 0.3);
            score_b.partial_cmp(&score_a).unwrap()
        });

        // Take top N validators
        validators.truncate(self.config.max_validators as usize);

        // Update validator set
        self.validator_set = ValidatorSet::new(self.config.max_validators, self.config.min_stake);
        for validator in validators {
            self.validator_set.add_validator(validator)?;
        }

        // Reset for new epoch
        self.current_epoch += 1;
        self.current_slot = 0;
        self.validator_scores.clear();

        Ok(())
    }

    fn update_validator_scores(&mut self) -> Result<()> {
        let validators = self.validator_set.get_active_validators();
        
        for validator in validators {
            let performance_score = if validator.blocks_produced + validator.blocks_missed > 0 {
                (validator.blocks_produced as f64 / (validator.blocks_produced + validator.blocks_missed) as f64) * 100.0
            } else {
                0.0
            };

            self.validator_scores.insert(
                validator.public_key.clone(),
                performance_score as u32,
            );
        }

        Ok(())
    }

    pub fn get_current_epoch(&self) -> u32 {
        self.current_epoch
    }

    pub fn get_current_slot(&self) -> u32 {
        self.current_slot
    }

    pub fn get_validator_set(&self) -> &ValidatorSet {
        &self.validator_set
    }

    pub fn update_validator_stake(&mut self, public_key: &[u8], new_stake: u128) -> Result<()> {
        if let Some(validator) = self.validator_set.get_validator(public_key) {
            let mut updated = validator.clone();
            updated.stake = new_stake;
            self.validator_set.update_validator(updated)
        } else {
            Err(ConsensusError::ValidatorSetUpdate("Validator not found".into()))
        }
    }
}
