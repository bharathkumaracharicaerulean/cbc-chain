//! Consensus metrics implementation
//! 
//! This module handles tracking and reporting of consensus-related metrics
//! including validator economics, rewards, and slashing data.

use sp_runtime::traits::Block as BlockTrait;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;

use std::collections::HashMap;
use std::time::Instant;
use std::sync::Arc;
use parking_lot::RwLock;
use prometheus::{
    Registry, Counter, Gauge, Histogram, HistogramOpts, Opts,
    register_counter_with_registry, register_gauge_with_registry, register_histogram_with_registry,
};
use pallet_cbc_dcf::DcfApi;
// Remove unused imports - we'll use the runtime types directly in the trait bound

/// Validator economics metrics for Prometheus
#[derive(Clone)]
pub struct ValidatorEconomicsMetrics {
    /// Number of active validators
    pub active_validators: Gauge,
    /// Total stake across all validators
    pub total_stake: Gauge,
    /// Total rewards distributed
    pub total_rewards_distributed: Counter,
    /// Total amount slashed
    pub total_slashed_amount: Counter,
    /// Average validator score
    pub average_validator_score: Gauge,
    /// Epoch duration histogram
    pub epoch_duration: Histogram,
    /// Rewards per epoch
    pub rewards_per_epoch: Histogram,
    /// Slashing events counter
    pub slashing_events: Counter,
    /// Validator participation rate
    pub validator_participation_rate: Gauge,
    /// Total inference results processed
    pub total_inference_results: Counter,
}

impl ValidatorEconomicsMetrics {
    /// Create new validator economics metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        Ok(Self {
            active_validators: register_gauge_with_registry!(
                Opts::new("cbc_active_validators", "Number of active validators"),
                registry
            )?,
            total_stake: register_gauge_with_registry!(
                Opts::new("cbc_total_stake", "Total stake across all validators"),
                registry
            )?,
            total_rewards_distributed: register_counter_with_registry!(
                Opts::new("cbc_total_rewards_distributed", "Total rewards distributed to validators"),
                registry
            )?,
            total_slashed_amount: register_counter_with_registry!(
                Opts::new("cbc_total_slashed_amount", "Total amount slashed from validators"),
                registry
            )?,
            average_validator_score: register_gauge_with_registry!(
                Opts::new("cbc_average_validator_score", "Average validator performance score"),
                registry
            )?,
            epoch_duration: register_histogram_with_registry!(
                HistogramOpts::new("cbc_epoch_duration_seconds", "Duration of epochs in seconds"),
                registry
            )?,
            rewards_per_epoch: register_histogram_with_registry!(
                HistogramOpts::new("cbc_rewards_per_epoch", "Rewards distributed per epoch"),
                registry
            )?,
            slashing_events: register_counter_with_registry!(
                Opts::new("cbc_slashing_events", "Number of slashing events"),
                registry
            )?,
            validator_participation_rate: register_gauge_with_registry!(
                Opts::new("cbc_validator_participation_rate", "Average validator participation rate"),
                registry
            )?,
            total_inference_results: register_counter_with_registry!(
                Opts::new("cbc_total_inference_results", "Total inference results processed"),
                registry
            )?,
        })
    }

    /// Update active validators count
    pub fn update_active_validators(&self, count: u64) {
        self.active_validators.set(count as f64);
    }

    /// Update total stake
    pub fn update_total_stake(&self, stake: u128) {
        self.total_stake.set(stake as f64);
    }

    /// Record reward distribution
    pub fn record_reward_distribution(&self, amount: u128) {
        self.total_rewards_distributed.inc_by(amount as f64);
    }

    /// Record slashing event
    pub fn record_slashing(&self, amount: u128) {
        self.total_slashed_amount.inc_by(amount as f64);
        self.slashing_events.inc();
    }

    /// Update average validator score
    pub fn update_average_score(&self, score: f64) {
        self.average_validator_score.set(score);
    }

    /// Record epoch duration
    pub fn record_epoch_duration(&self, duration_secs: f64) {
        self.epoch_duration.observe(duration_secs);
    }

    /// Record rewards per epoch
    pub fn record_rewards_per_epoch(&self, rewards: f64) {
        self.rewards_per_epoch.observe(rewards);
    }

    /// Update validator participation rate
    pub fn update_participation_rate(&self, rate: f64) {
        self.validator_participation_rate.set(rate);
    }

    /// Record inference result
    pub fn record_inference_result(&self) {
        self.total_inference_results.inc();
    }
}

/// Enhanced ConsensusMetrics struct for comprehensive monitoring
#[derive(Clone)]
pub struct ConsensusMetrics {
    /// Number of active validators
    pub active_validators: Gauge,
    /// Total reserved stake across all validators
    pub total_reserved_stake: Gauge,
    /// Current epoch number
    pub current_epoch: Gauge,
    /// Total rewards distributed to validators
    pub total_rewards_distributed: Counter,
    /// Total amount slashed from validators
    pub total_slashed_amount: Counter,
}

impl ConsensusMetrics {
    /// Create new ConsensusMetrics with Prometheus registry
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        Ok(Self {
            active_validators: register_gauge_with_registry!(
                Opts::new("cbc_consensus_active_validators", "Number of active validators in the consensus"),
                registry
            )?,
            total_reserved_stake: register_gauge_with_registry!(
                Opts::new("cbc_consensus_total_reserved_stake", "Total reserved stake across all validators"),
                registry
            )?,
            current_epoch: register_gauge_with_registry!(
                Opts::new("cbc_consensus_current_epoch", "Current epoch number"),
                registry
            )?,
            total_rewards_distributed: register_counter_with_registry!(
                Opts::new("cbc_consensus_total_rewards_distributed", "Total rewards distributed to validators"),
                registry
            )?,
            total_slashed_amount: register_counter_with_registry!(
                Opts::new("cbc_consensus_total_slashed_amount", "Total amount slashed from validators"),
                registry
            )?,
        })
    }

    /// Update metrics from runtime state
    pub fn update_from_runtime<B, C>(&self, client: &Arc<C>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        B: BlockTrait,
        C: ProvideRuntimeApi<B> + HeaderBackend<B>,
        C::Api: DcfApi<B, sp_runtime::AccountId32, u128, u32>,
    {
        let best_hash = client.info().best_hash;
        let api = client.runtime_api();

        // Update active validators count
        let active_validators = api.get_active_validators(best_hash)
            .map_err(|e| format!("Failed to get active validators: {:?}", e))?;
        self.active_validators.set(active_validators.len() as f64);

        // Update current epoch
        let current_epoch = api.get_current_epoch(best_hash)
            .map_err(|e| format!("Failed to get current epoch: {:?}", e))?;
        self.current_epoch.set(current_epoch as f64);

        // Calculate total reserved stake
        let mut total_stake: u128 = 0;
        for validator in &active_validators {
            let stake = api.get_validator_stake(best_hash, validator.clone())
                .map_err(|e| format!("Failed to get validator stake: {:?}", e))?;
            total_stake = total_stake.saturating_add(stake);
        }
        self.total_reserved_stake.set(total_stake as f64);

        Ok(())
    }

    /// Record reward distribution
    pub fn record_reward_distribution(&self, amount: u128) {
        self.total_rewards_distributed.inc_by(amount as f64);
    }

    /// Record slashing event
    pub fn record_slashing(&self, amount: u128) {
        self.total_slashed_amount.inc_by(amount as f64);
    }

    /// Update active validators count manually
    pub fn update_active_validators(&self, count: u64) {
        self.active_validators.set(count as f64);
    }

    /// Update total reserved stake manually
    pub fn update_total_reserved_stake(&self, stake: u128) {
        self.total_reserved_stake.set(stake as f64);
    }

    /// Update current epoch manually
    pub fn update_current_epoch(&self, epoch: u32) {
        self.current_epoch.set(epoch as f64);
    }
}

/// Legacy metrics for tracking consensus performance (kept for backward compatibility)
pub struct LegacyConsensusMetrics<B: BlockTrait> {
    /// Total number of blocks produced
    pub total_blocks: u32,
    /// Total number of transactions processed
    pub total_transactions: u32,
    /// Average time between blocks
    pub average_block_time: u64,
    /// Per-validator metrics
    pub validator_metrics: HashMap<B::Hash, u32>,
    /// Start time of the current epoch
    pub epoch_start_time: Instant,
    /// Validator economics metrics
    pub economics_metrics: Option<ValidatorEconomicsMetrics>,
    /// Epoch rewards tracking
    pub epoch_rewards: Arc<RwLock<u128>>,
}

impl<B: BlockTrait> LegacyConsensusMetrics<B> {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            total_blocks: 0,
            total_transactions: 0,
            average_block_time: 0,
            validator_metrics: HashMap::new(),
            epoch_start_time: Instant::now(),
            economics_metrics: None,
            epoch_rewards: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new metrics tracker with Prometheus registry
    pub fn new_with_registry(registry: &Registry) -> Result<Self, prometheus::Error> {
        let economics_metrics = ValidatorEconomicsMetrics::new(registry)?;
        
        Ok(Self {
            total_blocks: 0,
            total_transactions: 0,
            average_block_time: 0,
            validator_metrics: HashMap::new(),
            epoch_start_time: Instant::now(),
            economics_metrics: Some(economics_metrics),
            epoch_rewards: Arc::new(RwLock::new(0)),
        })
    }

    /// Update block metrics with new block time and transaction count
    pub fn update_block_metrics(&mut self, block_time: u64, tx_count: u32) {
        self.total_blocks = self.total_blocks.saturating_add(1);
        self.total_transactions = self.total_transactions.saturating_add(tx_count);
        self.average_block_time = (self.average_block_time + block_time) / 2;
    }

    /// Update validator metrics with new score
    pub fn update_validator_metrics(&mut self, validator: B::Hash, score: u32) {
        self.validator_metrics.insert(validator, score);
    }

    /// Get the duration of the current epoch
    pub fn get_epoch_duration(&self) -> u64 {
        self.epoch_start_time.elapsed().as_secs()
    }

    /// Update validator economics metrics
    pub fn update_validator_economics(&self, 
        active_count: u64, 
        total_stake: u128, 
        avg_score: f64,
        participation_rate: f64
    ) {
        if let Some(ref metrics) = self.economics_metrics {
            metrics.update_active_validators(active_count);
            metrics.update_total_stake(total_stake);
            metrics.update_average_score(avg_score);
            metrics.update_participation_rate(participation_rate);
        }
    }

    /// Record reward distribution
    pub fn record_reward(&self, amount: u128) {
        if let Some(ref metrics) = self.economics_metrics {
            metrics.record_reward_distribution(amount);
        }
        
        // Track epoch rewards
        let mut epoch_rewards = self.epoch_rewards.write();
        *epoch_rewards = epoch_rewards.saturating_add(amount);
    }

    /// Record slashing event
    pub fn record_slashing(&self, amount: u128) {
        if let Some(ref metrics) = self.economics_metrics {
            metrics.record_slashing(amount);
        }
    }

    /// Record inference result
    pub fn record_inference(&self) {
        if let Some(ref metrics) = self.economics_metrics {
            metrics.record_inference_result();
        }
    }

    /// Start new epoch (reset epoch-specific metrics)
    pub fn start_new_epoch(&mut self) {
        let epoch_duration = self.get_epoch_duration() as f64;
        let epoch_rewards = {
            let mut rewards = self.epoch_rewards.write();
            let current_rewards = *rewards;
            *rewards = 0; // Reset for new epoch
            current_rewards
        };

        if let Some(ref metrics) = self.economics_metrics {
            metrics.record_epoch_duration(epoch_duration);
            metrics.record_rewards_per_epoch(epoch_rewards as f64);
        }

        self.epoch_start_time = Instant::now();
    }
}