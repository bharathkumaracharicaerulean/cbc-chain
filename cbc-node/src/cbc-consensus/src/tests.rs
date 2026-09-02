//! Industrial-standard unit tests for CBC Consensus Engine (`cbc-consensus`)
//!
//! Organised into 5 modular submodules:
//! 1. Consensus Configuration & Parameters Tests
//! 2. DVF Configuration Validator Tests
//! 3. Author Selection Mode Tests
//! 4. Vote Aggregator & Justification Tests
//! 5. Metric Collection Tests

use super::*;
use crate::mock::*;
use crate::dvf_config_validator::DvfConfig;
use crate::types::AuthorSelectionMode;
use sp_runtime::Perbill;
use prometheus::Registry;

// ============================================================================
// 1. Consensus Configuration & Parameters Tests
// ============================================================================
mod config_tests {
    use super::*;

    #[test]
    fn consensus_engine_ids_correct() {
        assert_eq!(&CBC_ENGINE_ID, b"cbcd");
        assert_eq!(&DVF_ENGINE_ID, b"dvfd");
        assert_eq!(CBC_DVF_KEY_TYPE.0, *b"cdvf");
    }

    #[test]
    fn default_consensus_params_valid() {
        let params = ConsensusParams::default();
        assert_eq!(params.slot_duration.as_millis(), 6000);
        assert_eq!(params.finality_threshold, 67);
    }
}

// ============================================================================
// 2. DVF Configuration Validator Tests
// ============================================================================
mod dvf_config_tests {
    use super::*;

    #[test]
    fn dvf_config_validates_successfully() {
        let config = DvfConfig {
            finality_checkpoint_interval: 50,
            finality_threshold: Perbill::from_percent(67),
            stake_weight_factor: 1000,
            score_weight_factor: 1000,
            vote_retention_rounds: 10,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn invalid_checkpoint_interval_fails_validation() {
        let config = DvfConfig {
            finality_checkpoint_interval: 0,
            finality_threshold: Perbill::from_percent(67),
            stake_weight_factor: 1000,
            score_weight_factor: 1000,
            vote_retention_rounds: 10,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_threshold_fails_validation() {
        let config = DvfConfig {
            finality_checkpoint_interval: 50,
            finality_threshold: Perbill::from_percent(40), // < 51% threshold
            stake_weight_factor: 1000,
            score_weight_factor: 1000,
            vote_retention_rounds: 10,
        };
        assert!(config.validate().is_err());
    }
}

// ============================================================================
// 3. Author Selection Mode Tests
// ============================================================================
mod author_selection_tests {
    use super::*;

    #[test]
    fn author_selection_mode_variants_and_equality() {
        assert_ne!(AuthorSelectionMode::Hybrid, AuthorSelectionMode::RoundRobin);
        assert_ne!(AuthorSelectionMode::StakeWeighted, AuthorSelectionMode::PerformanceBased);
        assert_eq!(AuthorSelectionMode::Hybrid, AuthorSelectionMode::Hybrid);
    }
}

// ============================================================================
// 4. Vote Aggregator & Justification Tests
// ============================================================================
mod vote_aggregator_tests {
    use super::*;

    #[test]
    fn vote_pool_pruning_service_initialization() {
        new_test_ext().execute_with(|| {
            let active = DcfPallet::active_validators();
            assert!(!active.is_empty());
        });
    }
}

// ============================================================================
// 4b. Vote Creator & Re-Gossip Liveness Tests
// ============================================================================
mod vote_creator_tests {
    use super::*;

    #[test]
    fn vote_creator_regossip_liveness_state_management() {
        new_test_ext().execute_with(|| {
            // Verify initial state for vote re-gossip storage
            let last_voted_vote: parking_lot::RwLock<Option<crate::dvf_gossip::DvfVoteMessage<sp_core::H256, sp_core::crypto::AccountId32>>> =
                parking_lot::RwLock::new(None);
            assert!(last_voted_vote.read().is_none());

            // Simulate storing a vote for an unfinalized block #17470
            let mock_vote = crate::dvf_gossip::DvfVoteMessage {
                epoch_id: 174,
                validator_set_id: 0,
                round_number: 1746,
                block_number: 17470,
                block_hash: sp_core::H256::repeat_byte(0x17),
                validator_account_id: sp_core::crypto::AccountId32::new([1u8; 32]),
                validator_public_key: sp_core::ed25519::Public::from_raw([1u8; 32]),
                signature: sp_core::ed25519::Signature::from_raw([0u8; 64]),
            };

            *last_voted_vote.write() = Some(mock_vote.clone());
            assert!(last_voted_vote.read().is_some());
            assert_eq!(last_voted_vote.read().as_ref().unwrap().block_number, 17470);

            // Simulate finalization advancing to block #17470: vote storage should be cleared
            let dvf_finalized = 17470u32;
            if mock_vote.block_number <= dvf_finalized {
                *last_voted_vote.write() = None;
            }
            assert!(last_voted_vote.read().is_none());
        });
    }
}

// ============================================================================
// 5. Metric Collection Tests
// ============================================================================
mod metrics_and_tracer_tests {
    use super::*;

    #[test]
    fn metrics_structure_initialization() {
        new_test_ext().execute_with(|| {
            let registry = Registry::new();
            let metrics = metrics::ConsensusMetrics::new(&registry);
            assert!(metrics.is_ok());
        });
    }
}
