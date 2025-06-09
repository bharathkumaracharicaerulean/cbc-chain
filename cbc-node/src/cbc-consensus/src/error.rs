#![allow(unused_imports)]
//! Consensus error handling
//!
//! This module defines the error types used throughout the consensus system,
//! providing a centralized way to handle and propagate errors.

use thiserror::Error;
use sp_runtime::traits::Block as BlockT;
use sp_core::ed25519::Public;

/// Consensus error types
#[derive(Error, Debug)]
pub enum ConsensusError {
    /// Block validation error
    #[error("Block validation error: {0}")]
    BlockValidation(#[from] BlockValidationError),
    /// Epoch transition error
    #[error("Epoch transition error: {0}")]
    EpochTransition(#[from] EpochTransitionError),
    /// Author selection error
    #[error("Author selection error: {0}")]
    AuthorSelection(#[from] AuthorSelectionError),
    /// Block import error
    #[error("Block import error: {0}")]
    BlockImport(#[from] BlockImportError),
    /// Finality error
    #[error("Finality error: {0}")]
    Finality(#[from] FinalityError),
    /// Validator set error
    #[error("Validator set error: {0}")]
    ValidatorSet(#[from] ValidatorSetError),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Block validation error
#[derive(Error, Debug)]
pub enum BlockValidationError {
    /// Invalid block number
    #[error("Invalid block number")]
    InvalidBlockNumber,
    /// Invalid block size
    #[error("Invalid block size")]
    InvalidBlockSize,
    /// Invalid block author
    #[error("Invalid block author")]
    InvalidAuthor,
    /// Invalid block signature
    #[error("Invalid block signature")]
    InvalidSignature,
    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Epoch transition error
#[derive(Error, Debug)]
pub enum EpochTransitionError {
    /// Invalid epoch number
    #[error("Invalid epoch number")]
    InvalidEpochNumber,
    /// Invalid epoch transition
    #[error("Invalid epoch transition")]
    InvalidTransition,
    /// Invalid epoch configuration
    #[error("Invalid epoch configuration")]
    InvalidConfig,
}

/// Author selection error
#[derive(Error, Debug)]
pub enum AuthorSelectionError {
    /// No validators available
    #[error("No validators available")]
    NoValidators,
    /// Invalid validator
    #[error("Invalid validator")]
    InvalidValidator,
    /// Invalid selection criteria
    #[error("Invalid selection criteria")]
    InvalidCriteria,
    /// Criteria not met
    #[error("Criteria not met: {0}")]
    CriteriaNotMet(String),
}

/// Block import error
#[derive(Error, Debug)]
pub enum BlockImportError {
    /// Block already exists
    #[error("Block already exists: {0}")]
    BlockExists(String),
    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    /// Import failed
    #[error("Import failed: {0}")]
    ImportFailed(String),
}

/// Finality error
#[derive(Error, Debug)]
pub enum FinalityError {
    /// Block not found
    #[error("Block not found")]
    BlockNotFound,
    /// Block not final
    #[error("Block not final: {0}")]
    NotFinal(String),
    /// Invalid finality status
    #[error("Invalid finality status")]
    InvalidStatus,
}

/// Validator set error
#[derive(Error, Debug)]
pub enum ValidatorSetError {
    #[error("Validator set is full")]
    SetFull,
    #[error("Validator already exists")]
    AlreadyExists,
    #[error("Insufficient stake")]
    InsufficientStake,
    #[error("Validator not found")]
    NotFound,
    /// Invalid validator
    #[error("Invalid validator")]
    InvalidValidator,
    /// Validator set full
    #[error("Validator set full")]
    ValidatorSetFull,
}

impl From<&str> for BlockValidationError {
    fn from(msg: &str) -> Self {
        BlockValidationError::Unknown(msg.to_string())
    }
} 