//! CBC Consensus Library
//!
//! This library provides the consensus mechanism for the CBC blockchain,
//! implementing a decentralized consensus framework that combines Proof of Stake
//! and Proof of Inference.
//!
//! # Overview
//!
//! The consensus system is composed of several key components:
//!
//! - Core consensus engine (DCF)
//! - Validator management and selection
//! - Block production and import
//! - Epoch management
//! - Finality tracking
//! - Metrics collection
//!
//! # Usage
//!
//! ```rust
//! use cbc_consensus::{
//!     DcfConsensus,
//!     DcfConfig,
//!     ValidatorSet,
//!     EpochManager,
//! };
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

/// Core consensus engine implementation
pub mod dcf;

/// Validator and author selection
pub mod author_selection;
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

// Re-export commonly used types from dcf
pub use dcf::{
    DcfBlockImport,
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

// Re-export commonly used types from other modules
pub use proposer_factory::ProposerFactory;
pub use finality::FinalityEngine;
pub use metrics::ConsensusMetrics;


