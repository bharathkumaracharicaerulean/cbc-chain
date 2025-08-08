#![cfg(test)]
use super::*;
use frame_support::{
    assert_noop, assert_ok,
};
use crate::{
    pallet::Error as PalletError,
};
use frame_system::pallet_prelude::BlockNumberFor;

// Re-export mock types for easier access
pub use crate::mock::*;

// --- Test Cases ---

#[test]
fn test_governance_proposal_lifecycle() {
    let mut ext = new_test_ext_with_validators(vec![1]);
    ext.execute_with(|| {
        // Test proposal submission
        let proposal = pallet::ProposalAction::Reward { validator: 1, amount: 10 };
        
        // Only root can submit proposals when governance is enabled
        assert_ok!(Dcf::set_governance_mode(RuntimeOrigin::root(), true));
        assert_ok!(Dcf::submit_proposal(RuntimeOrigin::root(), proposal.clone()));
        
        // Test voting
        assert_ok!(Dcf::vote_proposal(RuntimeOrigin::signed(1), 0, true));
        
        // Test proposal execution after voting period
        run_to_block(10);
        
        // Execute the proposal
        assert_ok!(Dcf::execute_proposal(RuntimeOrigin::root(), 0));
        
        // Verify the validator's score was boosted (Reward action)
        let profile = Dcf::get_validator_profile(1).unwrap();
        assert!(profile.0 > 100); // Score should be higher than initial
    });
}

#[test]
fn test_validator_scoring() {
    let mut ext = new_test_ext_with_validators(vec![1, 2]);
    ext.execute_with(|| {
        // Test updating stake score
        assert_ok!(Dcf::update_validator_stake_score(RuntimeOrigin::signed(1), 1));
        let profile1 = Dcf::get_validator_profile(1).unwrap();
        assert!(profile1.0 > 0);
        
        // Test updating inference score
        assert_ok!(Dcf::update_validator_inference_score(RuntimeOrigin::signed(2), 2));
        let profile2 = Dcf::get_validator_profile(2).unwrap();
        assert!(profile2.0 > 0);
        
        // Test score decay over epochs
        run_to_block(20);
        <pallet::Pallet<mock::Test> as frame_support::traits::Hooks<BlockNumberFor<mock::Test>>>::on_initialize(System::block_number());
        <pallet::Pallet<mock::Test> as frame_support::traits::Hooks<BlockNumberFor<mock::Test>>>::on_finalize(System::block_number());
        System::finalize();
        let profile1_after = Dcf::get_validator_profile(1).unwrap();
        assert!(profile1_after.0 < profile1.0);
    });
}

#[test]
fn test_epoch_transition() {
    let mut ext = new_test_ext_with_validators(vec![1]);
    ext.execute_with(|| {
        let initial_epoch = Dcf::current_epoch();
        
        // Progress through an epoch
        run_to_block(10);
        
        // Manually trigger epoch transition
        <pallet::Pallet<mock::Test> as frame_support::traits::Hooks<BlockNumberFor<mock::Test>>>::on_initialize(System::block_number());
        
        // Check that epoch advanced
        let new_epoch = Dcf::current_epoch();
        assert_eq!(new_epoch, initial_epoch + 1);
        
        // Check that validator state was updated
        let state = Dcf::validator_states(1).unwrap();
        assert_eq!(state.current.epoch, new_epoch); // Current epoch's data
    });
}

#[test]
fn test_validator_management() {
    new_test_ext().execute_with(|| {
        // Test joining the validator set
        assert_ok!(Dcf::join_validator_set(RuntimeOrigin::signed(1)));
        
        // Should be in pending state until next epoch
        assert!(!Dcf::is_validator_active(&1));
        
        // Advance to next epoch
        run_to_block(10);
        
        // Now should be active
        assert!(Dcf::is_validator_active(&1));
        
        // Test leaving the validator set
        assert_ok!(Dcf::leave_validator_set(RuntimeOrigin::signed(1)));
        
        // Still active until next epoch
        assert!(Dcf::is_validator_active(&1));
        
        // Advance to next epoch
        run_to_block(20);
        
        // Now should be inactive
        assert!(!Dcf::is_validator_active(&1));
    });
}

#[test]
fn test_block_authorship() {
    new_test_ext_with_validators(vec![1, 2]).execute_with(|| {
        // Record block authorship
        Dcf::record_block_authorship(&1).unwrap();
        
        // Check score was boosted
        let state1 = Dcf::validator_states(1).unwrap();
        assert_eq!(state1.current.authored_blocks, 1);
        
        // Record missed block
        Dcf::record_missed_block(&2).unwrap();
        
        // Check penalty was applied
        let state2 = Dcf::validator_states(2).unwrap();
        assert_eq!(state2.current.missed_blocks, 1);
    });
}

#[test]
fn test_consensus_weights() {
    new_test_ext().execute_with(|| {
        // Test getting initial weights
        let pos_weight = Dcf::pos_weight();
        let poi_weight = Dcf::poi_weight();
        assert_eq!(pos_weight, 50); // Default from mock
        assert_eq!(poi_weight, 50); // Default from mock
        
        // Test updating weights
        assert_ok!(Dcf::update_consensus_weights(
            RuntimeOrigin::root(),
            60, // pos_weight
            40  // poi_weight
        ));
        
        // Verify weights were updated
        let pos_weight = Dcf::pos_weight();
        let poi_weight = Dcf::poi_weight();
        assert_eq!(pos_weight, 60);
        assert_eq!(poi_weight, 40);
        
        // Test invalid weights (sum != 100)
        assert_noop!(
            Dcf::update_consensus_weights(RuntimeOrigin::root(), 60, 50),
            PalletError::<Test>::InvalidWeight
        );
    });
}

#[test]
fn test_validator_score_decay_and_ejection() {
    new_test_ext_with_validators(vec![1]).execute_with(|| {
        // Get the validator we just created
        let validator = 1;
        
        // Simulate inactivity and decay
        assert_ok!(Dcf::apply_score_decay(&validator, 10));
        let state = Dcf::validator_states(validator).unwrap();
        assert!(state.current.final_score < 100);

        // Simulate score below threshold triggers ejection
        Dcf::validator_states(validator).map(|mut s| {
            s.current.final_score = 0;
            <pallet::ValidatorStates<Test>>::insert(validator, s);
        });
        assert_ok!(Dcf::apply_score_decay(&validator, 20));
        assert!(!Dcf::active_validators().contains(&validator));
    });
}

#[test]
fn test_epoch_transition_and_block_author_tracking() {
    new_test_ext_with_validators(vec![1]).execute_with(|| {
        let validator = 1;
        
        // Simulate block authorship
        assert_ok!(Dcf::record_block_authorship(&validator));
        let state = Dcf::validator_states(validator).unwrap();
        assert_eq!(state.current.authored_blocks, 1);

        // Simulate missed block
        assert_ok!(Dcf::record_missed_block(&validator));
        let state = Dcf::validator_states(validator).unwrap();
        assert_eq!(state.current.missed_blocks, 1);

        // Epoch transition
        <pallet::EpochConfigStorage<Test>>::put(crate::EpochConfig { 
            blocks_per_epoch: 1, 
            min_stake: 0, 
            max_validators: 10 
        });
        <pallet::CurrentEpoch<Test>>::put(0);

        // Advance to next block to trigger epoch transition
        run_to_block(2);

        // After epoch transition, epoch should increment
        assert_eq!(Dcf::current_epoch(), 1);
    });
}

#[test]
fn score_integration_works() {
    new_test_ext().execute_with(|| {
        let validator = 1u64;
        
        // Setup mock scores
        MockPos::set_score(validator, 1000);
        MockPoi::set_score(validator, 500);
        
        // Update weights
        assert_ok!(DcfModule::set_weights(Origin::root(), 60, 40));
        
        // Update and check scores
        assert_ok!(DcfModule::update_validator_scores(&validator));
        
        let breakdown = DcfModule::get_score_breakdown(&validator).unwrap();
        assert_eq!(breakdown.pos_score, 1000);
        assert_eq!(breakdown.poi_score, 500);
        assert_eq!(breakdown.final_score, 800); // (1000 * 60 + 500 * 40) / 100
    });
}
