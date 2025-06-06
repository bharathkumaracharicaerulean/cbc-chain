#![allow(unused_imports)]
//! Finality module
//!
//! This module implements the soft-finality rules for the CBC consensus,
//! determining when blocks can be considered final based on block depth and time.

use crate::types::FinalityConfig;
use crate::error::{ConsensusError, FinalityError};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::generic::BlockId;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sp_core::ed25519::Public;

/// Finality status
#[derive(Clone, Debug, PartialEq)]
pub enum FinalityStatus {
    /// Block is not yet final
    NotFinal,
    /// Block is final
    Final,
}

/// Finality engine
pub struct FinalityEngine<Block: BlockT> {
    /// Finality configuration
    config: FinalityConfig,
    /// Block status tracking
    block_status: HashMap<Block::Hash, (FinalityStatus, Instant)>,
}

impl<Block: BlockT> FinalityEngine<Block> {
    /// Create a new finality engine
    pub fn new(config: FinalityConfig) -> Self {
        Self {
            config,
            block_status: HashMap::new(),
        }
    }

    /// Get block status
    pub fn get_block_status(&self, block_hash: &Block::Hash) -> Option<&FinalityStatus> {
        self.block_status.get(block_hash).map(|(status, _)| status)
    }

    /// Check if block is finalized
    pub fn is_finalized(&self, block_hash: &Block::Hash) -> bool {
        self.get_block_status(block_hash)
            .map(|status| *status == FinalityStatus::Final)
            .unwrap_or(false)
    }

    /// Check block finality
    pub fn check_finality(&self, block_hash: Block::Hash, block_number: u32, latest_block: u32) -> Result<(), ConsensusError> {
        let depth = latest_block.saturating_sub(block_number);
        if depth < self.config.finality_blocks {
            return Err(ConsensusError::Finality(
                FinalityError::NotFinal(format!(
                    "Block {} is not deep enough (depth: {})",
                    block_number,
                    depth
                ))
            ));
        }

        if let Some((status, time)) = self.block_status.get(&block_hash) {
            if time.elapsed() > Duration::from_millis(self.config.max_finality_time) {
                return Err(ConsensusError::Finality(
                    FinalityError::NotFinal("Block has exceeded maximum finality time".to_string())
                ));
            }
            if status == &FinalityStatus::Final {
                return Ok(());
            }
        }

        Err(ConsensusError::Finality(
            FinalityError::NotFinal("Block is not final".to_string())
        ))
    }

    /// Update block finality status
    pub fn update_finality(&mut self, block_hash: Block::Hash, status: FinalityStatus) {
        let now = Instant::now();
        self.block_status.insert(block_hash, (status, now));
    }

    /// Add new block
    pub fn add_block(&mut self, block_hash: Block::Hash) {
        let now = Instant::now();
        self.block_status.insert(block_hash, (FinalityStatus::NotFinal, now));
    }

    /// Prune old entries
    pub fn prune_old_entries(&mut self, _current_block: u32) {
        self.block_status.retain(|_, (status, time)| {
            status == &FinalityStatus::Final || time.elapsed() <= Duration::from_millis(self.config.max_finality_time)
        });
    }

    /// Get finality configuration
    pub fn get_config(&self) -> &FinalityConfig {
        &self.config
    }
}

impl<Block: BlockT> Default for FinalityEngine<Block> {
    fn default() -> Self {
        Self::new(FinalityConfig::default())
    }
} 