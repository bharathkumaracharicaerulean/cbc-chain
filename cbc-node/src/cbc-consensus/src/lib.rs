//! CBC Consensus Engine
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

/// Core consensus engine implementation
pub mod dcf;

/// Validator and author selection
pub mod author_selection;
/// Validator set management and operations
pub mod validator_set;

/// Block production and import
pub mod proposer_factory;
pub mod import_queue;

/// Epoch management
pub mod epoch_manager;

/// Core types and error handling
pub mod types;
pub mod error;

/// Monitoring and finality
pub mod metrics;
pub mod finality;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod metrics_test;

#[cfg(test)]
mod validator_set_test;

#[cfg(test)]
mod epoch_manager_test;

// Re-export commonly used types from dcf
pub use dcf::{
    RealBlockImport,
    DcfConsensus,
};

// Re-export commonly used types from types module
pub use types::{
    ValidatorInfo,
    ValidatorMetrics,
    EpochConfig,
    AuthorSelectionMode,
    ConsensusParams,
    BlockT,
};

// Re-export commonly used error types
pub use error::{
    ConsensusError,
    Result,
};

pub use proposer_factory::ProposerFactory;
pub use finality::FinalityEngine;


