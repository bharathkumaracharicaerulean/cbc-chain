//! Tests for deterministic epoch processing functionality.

use super::*;
use crate::mock::*;
use frame_support::{assert_ok, traits::Get};

#[test]
fn test_deterministic_randomness_generation() {
    new_test_ext().execute_with(|| {
        let block_number = 100u32;
        let epoch = 1u32;
        let randomness_salt = [1u8; 32];
        let epoch_salt = [2u8; 16];
        
        // Generate randomness twice with same inputs
        let randomness1 = DcfPallet::generate_deterministic_randomness(
            block_number, epoch, &randomness_salt, &epoch_salt
        );
        let randomness2 = DcfPallet::generate_deterministic_randomness(
            block_number, epoch, &randomness_salt, &epoch_salt
        );
        
        // Should be identical
        assert_eq!(randomness1, randomness2);
        
        // Different inputs should produce different randomness
        let randomness3 = DcfPallet::generate_deterministic_randomness(
            block_number + 1, epoch, &randomness_salt, &epoch_salt
        );
        assert_ne!(randomness1, randomness3);
    });
}

#[test]
fn test_deterministic_author_sequence_generation() {
    new_test_ext().execute_with(|| {
        // Set up validators
        let validators = vec![1u64, 2u64, 3u64];
        for validator in &validators {
            assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(*validator), None));
        }
        
        let active_validators = DcfPallet::active_validators();
        let randomness_seed = [42u8; 32];
        let epoch = 1u32;
        
        // Generate sequence twice with same inputs
        let sequence1 = DcfPallet::generate_deterministic_author_sequence(
            epoch, &active_validators, &randomness_seed
        );
        let sequence2 = DcfPallet::generate_deterministic_author_sequence(
            epoch, &active_validators, &randomness_seed
        );
        
        // Should be identical
        assert_eq!(sequence1, sequence2);
        assert!(!sequence1.is_empty());
        
        // Different randomness should produce different sequence
        let different_seed = [43u8; 32];
        let sequence3 = DcfPallet::generate_deterministic_author_sequence(
            epoch, &active_validators, &different_seed
        );
        
        // Sequences should be different (with high probability)
        assert_ne!(sequence1, sequence3);
    });
}

#[test]
fn test_deterministic_engine_initialization() {
    new_test_ext().execute_with(|| {
        // Initially engine should have default state
        let initial_engine = DcfPallet::deterministic_engine();
        assert_eq!(initial_engine.randomness_salt, [0u8; 32]);
        assert_eq!(initial_engine.last_processed_epoch, 0);
        
        // Trigger epoch transition to initialize engine
        System::set_block_number(100);
        let _weight = DcfPallet::handle_deterministic_epoch_transition(100);
        
        // Engine should now be initialized
        let updated_engine = DcfPallet::deterministic_engine();
        assert_ne!(updated_engine.randomness_salt, [0u8; 32]);
        assert_eq!(updated_engine.last_processed_epoch, 1);
    });
}

#[test]
fn test_expected_author_with_deterministic_sequence() {
    new_test_ext().execute_with(|| {
        // Set up validators
        let validators = vec![1u64, 2u64, 3u64];
        for validator in &validators {
            assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(*validator), None));
        }
        
        // Trigger epoch transition to generate author sequence
        System::set_block_number(100);
        let _weight = DcfPallet::handle_deterministic_epoch_transition(100);
        
        // Test that get_expected_author returns consistent results
        let author1 = DcfPallet::get_expected_author(101);
        let author2 = DcfPallet::get_expected_author(101);
        
        assert_eq!(author1, author2);
        assert!(author1.is_some());
        
        // Different block numbers should potentially return different authors
        let author3 = DcfPallet::get_expected_author(102);
        assert!(author3.is_some());
    });
}

#[test]
fn test_epoch_processing_output_recording() {
    new_test_ext().execute_with(|| {
        // Set up validators
        let validators = vec![1u64, 2u64];
        for validator in &validators {
            assert_ok!(DcfPallet::join_validators(RuntimeOrigin::signed(*validator), None));
        }
        
        // Trigger epoch transition
        System::set_block_number(100);
        let _weight = DcfPallet::handle_deterministic_epoch_transition(100);
        
        // Check that processing output was recorded
        let epoch = 1u32;
        let output = DcfPallet::epoch_processing_outputs(epoch);
        assert!(output.is_some());
        
        let output = output.unwrap();
        assert_eq!(output.epoch, epoch);
        assert_eq!(output.transition_block, 100);
        assert!(!output.active_validators.is_empty());
        assert_ne!(output.output_hash, [0u8; 32]);
    });
}