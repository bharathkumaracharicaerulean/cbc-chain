use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct ConsensusMetrics {
    // Block production metrics
    blocks_produced: u64,
    blocks_missed: u64,
    average_block_time: Duration,
    last_block_time: Option<Instant>,

    // Validator metrics
    validator_blocks: HashMap<Vec<u8>, u64>,
    validator_missed: HashMap<Vec<u8>, u64>,

    // Epoch metrics
    current_epoch: u32,
    epoch_start_time: Instant,
    epoch_blocks: u64,
}

impl ConsensusMetrics {
    pub fn new() -> Self {
        Self {
            blocks_produced: 0,
            blocks_missed: 0,
            average_block_time: Duration::from_secs(0),
            last_block_time: None,
            validator_blocks: HashMap::new(),
            validator_missed: HashMap::new(),
            current_epoch: 0,
            epoch_start_time: Instant::now(),
            epoch_blocks: 0,
        }
    }

    pub fn record_block_production(&mut self, validator: &[u8]) {
        self.blocks_produced += 1;
        self.epoch_blocks += 1;
        *self.validator_blocks.entry(validator.to_vec()).or_insert(0) += 1;

        if let Some(last_time) = self.last_block_time {
            let block_time = last_time.elapsed();
            self.average_block_time = (self.average_block_time * (self.blocks_produced - 1) as u32
                + block_time)
                / self.blocks_produced as u32;
        }
        self.last_block_time = Some(Instant::now());
    }

    pub fn record_missed_block(&mut self, validator: &[u8]) {
        self.blocks_missed += 1;
        *self.validator_missed.entry(validator.to_vec()).or_insert(0) += 1;
    }

    pub fn start_new_epoch(&mut self) {
        self.current_epoch += 1;
        self.epoch_start_time = Instant::now();
        self.epoch_blocks = 0;
    }

    pub fn get_validator_stats(&self, validator: &[u8]) -> (u64, u64) {
        let blocks = self.validator_blocks.get(validator).copied().unwrap_or(0);
        let missed = self.validator_missed.get(validator).copied().unwrap_or(0);
        (blocks, missed)
    }

    pub fn get_epoch_stats(&self) -> (u32, Duration, u64) {
        (
            self.current_epoch,
            self.epoch_start_time.elapsed(),
            self.epoch_blocks,
        )
    }

    pub fn get_block_stats(&self) -> (u64, u64, Duration) {
        (
            self.blocks_produced,
            self.blocks_missed,
            self.average_block_time,
        )
    }
}
