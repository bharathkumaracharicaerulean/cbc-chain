use crate::consensus::types::{ValidatorInfo, ValidatorMetrics};
use crate::consensus::error::{ConsensusError, Result};
use std::collections::HashMap;

pub struct ValidatorSet {
    validators: HashMap<Vec<u8>, ValidatorInfo>,
    metrics: ValidatorMetrics,
    max_validators: u32,
    min_stake: u128,
}

impl ValidatorSet {
    pub fn new(max_validators: u32, min_stake: u128) -> Self {
        Self {
            validators: HashMap::new(),
            metrics: ValidatorMetrics::default(),
            max_validators,
            min_stake,
        }
    }

    pub fn add_validator(&mut self, validator: ValidatorInfo) -> Result<()> {
        if validator.stake < self.min_stake {
            return Err(ConsensusError::ValidatorSetUpdate(
                "Stake below minimum threshold".into(),
            ));
        }

        if self.validators.len() >= self.max_validators as usize {
            return Err(ConsensusError::ValidatorSetUpdate(
                "Maximum validator count reached".into(),
            ));
        }

        self.validators.insert(validator.public_key.clone(), validator);
        Ok(())
    }

    pub fn remove_validator(&mut self, public_key: &[u8]) -> Result<()> {
        self.validators
            .remove(public_key)
            .ok_or_else(|| ConsensusError::ValidatorSetUpdate("Validator not found".into()))?;
        Ok(())
    }

    pub fn update_validator(&mut self, validator: ValidatorInfo) -> Result<()> {
        if !self.validators.contains_key(&validator.public_key) {
            return Err(ConsensusError::ValidatorSetUpdate(
                "Validator not found".into(),
            ));
        }

        self.validators.insert(validator.public_key.clone(), validator);
        Ok(())
    }

    pub fn get_validator(&self, public_key: &[u8]) -> Option<&ValidatorInfo> {
        self.validators.get(public_key)
    }

    pub fn get_active_validators(&self) -> Vec<&ValidatorInfo> {
        self.validators.values().collect()
    }

    pub fn update_metrics(&mut self, public_key: &[u8], blocks_produced: u32, blocks_missed: u32) -> Result<()> {
        if let Some(validator) = self.validators.get_mut(public_key) {
            validator.blocks_produced += blocks_produced;
            validator.blocks_missed += blocks_missed;
            self.metrics.total_blocks += blocks_produced;
            self.metrics.total_missed += blocks_missed;
            Ok(())
        } else {
            Err(ConsensusError::ValidatorSetUpdate("Validator not found".into()))
        }
    }

    pub fn get_metrics(&self) -> &ValidatorMetrics {
        &self.metrics
    }
}
