//! Unit tests for the DCF parameter governance system.
//!
//! This module contains comprehensive tests for the parameter governance functionality
//! including parameter validation, range enforcement, and safety rails.

use super::*;
use crate::mock::*;
use frame_support::{
    assert_err, assert_ok,
};
use sp_runtime::traits::BadOrigin;

// Type alias for easier testing
type DcfModule = DcfPallet;

/// Test parameter governance system initialization
#[test]
fn governance_config_initialized_correctly() {
    new_test_ext().execute_with(|| {
        // Check that governance config is initialized with default values
        let config = DcfModule::governance_config();
        
        // Verify epoch configuration ranges
        assert_eq!(config.epoch_length.min, 50);
        assert_eq!(config.epoch_length.max, 14400);
        assert_eq!(config.epoch_length.current, 10); // From genesis config in mock
        
        // Verify consensus weight ranges
        assert_eq!(config.pos_weight.min, 1000);
        assert_eq!(config.pos_weight.max, 9000);
        assert_eq!(config.poi_weight.min, 1000);
        assert_eq!(config.poi_weight.max, 9000);
        
        // Verify weights sum to precision factor
        assert_eq!(config.pos_weight.current + config.poi_weight.current, 10000);
        
        // Verify performance threshold ranges
        assert!(config.min_performance_score.current < config.high_performance_score.current);
        assert!(config.min_participation_rate.current < config.high_participation_rate.current);
    });
}

/// Test successful parameter update with valid values
#[test]
fn update_parameter_success() {
    new_test_ext().execute_with(|| {
        // Advance to block 1 so events can be registered
        System::set_block_number(1);
        
        // Test updating epoch length within valid range
        let new_epoch_length = 200u32;
        let encoded_value = new_epoch_length.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::EpochLength,
            bounded_value
        ));
        
        // Verify the parameter was updated
        let config = DcfModule::governance_config();
        assert_eq!(config.epoch_length.current, new_epoch_length);
        
        // Check that event was emitted
        System::assert_last_event(RuntimeEvent::DcfPallet(Event::DcfParameterUpdated {
            parameter: ParameterType::EpochLength,
            old_value: 10u32.encode().try_into().unwrap(), // From genesis config
            new_value: new_epoch_length.encode().try_into().unwrap(),
        }));
    });
}

/// Test parameter update with value outside allowed range
#[test]
fn update_parameter_out_of_range() {
    new_test_ext().execute_with(|| {
        // Try to set epoch length below minimum
        let invalid_epoch_length = 10u32; // Below minimum of 50
        let encoded_value = invalid_epoch_length.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::EpochLength,
                bounded_value
            ),
            Error::<Test>::ParameterOutOfRange
        );
        
        // Try to set epoch length above maximum
        let invalid_epoch_length = 20000u32; // Above maximum of 14400
        let encoded_value = invalid_epoch_length.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::EpochLength,
                bounded_value
            ),
            Error::<Test>::ParameterOutOfRange
        );
    });
}

/// Test parameter update without root authorization
#[test]
fn update_parameter_unauthorized() {
    new_test_ext().execute_with(|| {
        let new_epoch_length = 200u32;
        let encoded_value = new_epoch_length.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        // Try to update parameter with signed origin (not root)
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::signed(1),
                ParameterType::EpochLength,
                bounded_value
            ),
            BadOrigin
        );
    });
}

/// Test consensus weight parameter validation
#[test]
fn update_consensus_weights_validation() {
    new_test_ext().execute_with(|| {
        // Test valid PoS weight update
        let new_pos_weight = 6000u64;
        let encoded_value = new_pos_weight.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        // First update PoI weight to maintain sum = 10000
        let new_poi_weight = 4000u64;
        let encoded_poi_value = new_poi_weight.encode();
        let bounded_poi_value = BoundedVec::try_from(encoded_poi_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::PoiWeight,
            bounded_poi_value
        ));
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::PosWeight,
            bounded_value
        ));
        
        // Verify weights were updated and still sum to 10000
        let config = DcfModule::governance_config();
        assert_eq!(config.pos_weight.current, new_pos_weight);
        assert_eq!(config.poi_weight.current, new_poi_weight);
        assert_eq!(config.pos_weight.current + config.poi_weight.current, 10000);
    });
}

/// Test consensus weight validation failure when sum is incorrect
#[test]
fn update_consensus_weights_invalid_sum() {
    new_test_ext().execute_with(|| {
        // Try to set PoS weight that would make sum != 10000
        let invalid_pos_weight = 7000u64; // With current PoI weight of 5000, sum would be 12000
        let encoded_value = invalid_pos_weight.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::PosWeight,
                bounded_value
            ),
            Error::<Test>::InvalidWeight
        );
    });
}

/// Test performance threshold parameter consistency
#[test]
fn update_performance_thresholds_consistency() {
    new_test_ext().execute_with(|| {
        // Try to set min performance score higher than high performance score
        let config = DcfModule::governance_config();
        let invalid_min_score = config.high_performance_score.current + 100;
        let encoded_value = invalid_min_score.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::MinPerformanceScore,
                bounded_value
            ),
            Error::<Test>::InvalidGovernanceConfig
        );
    });
}

/// Test participation rate parameter consistency
#[test]
fn update_participation_rates_consistency() {
    new_test_ext().execute_with(|| {
        // Try to set min participation rate higher than high participation rate
        let config = DcfModule::governance_config();
        
        // First, let's try with a value that's within the min participation rate range
        // but higher than the high participation rate
        let high_rate = config.high_participation_rate.current;
        let min_rate_max = config.min_participation_rate.max;
        
        // If the high rate is less than the min rate max, we can test the consistency check
        if high_rate < min_rate_max {
            let invalid_min_rate = high_rate + 1;
            let encoded_value = invalid_min_rate.encode();
            let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
            
            assert_err!(
                DcfModule::update_dcf_parameter(
                    RuntimeOrigin::root(),
                    ParameterType::MinParticipationRate,
                    bounded_value
                ),
                Error::<Test>::InvalidGovernanceConfig
            );
        } else {
            // If we can't test consistency, test that we get ParameterOutOfRange
            // when trying to set a value above the max
            let invalid_min_rate = min_rate_max + 1;
            let encoded_value = invalid_min_rate.encode();
            let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
            
            assert_err!(
                DcfModule::update_dcf_parameter(
                    RuntimeOrigin::root(),
                    ParameterType::MinParticipationRate,
                    bounded_value
                ),
                Error::<Test>::ParameterOutOfRange
            );
        }
    });
}

/// Test slash percent parameter validation
#[test]
fn update_slash_percent_validation() {
    new_test_ext().execute_with(|| {
        // Test valid slash percent
        let valid_slash_percent = 25u32;
        let encoded_value = valid_slash_percent.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::SlashPercent,
            bounded_value
        ));
        
        // Test invalid slash percent (over 100%)
        let invalid_slash_percent = 150u32;
        let encoded_value = invalid_slash_percent.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::SlashPercent,
                bounded_value
            ),
            Error::<Test>::ParameterOutOfRange
        );
    });
}

/// Test parameter application to active system configuration
#[test]
fn parameter_application_to_active_config() {
    new_test_ext().execute_with(|| {
        // Update PoS weight and verify it's applied to active storage
        let new_pos_weight = 6000u64;
        let new_poi_weight = 4000u64;
        
        // Update both weights to maintain sum
        let encoded_poi_value = new_poi_weight.encode();
        let bounded_poi_value = BoundedVec::try_from(encoded_poi_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::PoiWeight,
            bounded_poi_value
        ));
        
        let encoded_pos_value = new_pos_weight.encode();
        let bounded_pos_value = BoundedVec::try_from(encoded_pos_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::PosWeight,
            bounded_pos_value
        ));
        
        // Verify active storage was updated
        assert_eq!(DcfModule::pos_weight(), new_pos_weight);
        assert_eq!(DcfModule::poi_weight(), new_poi_weight);
    });
}

/// Test trust score weight parameter updates
#[test]
fn update_trust_score_weights() {
    new_test_ext().execute_with(|| {
        // Update trust score uptime weight
        let new_uptime_weight = 50u32;
        let encoded_value = new_uptime_weight.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::TrustScoreUptimeWeight,
            bounded_value
        ));
        
        // Verify the trust score config was updated
        let trust_config = DcfModule::trust_score_config();
        assert_eq!(trust_config.uptime_weight, new_uptime_weight);
        
        // Verify governance config was also updated
        let gov_config = DcfModule::governance_config();
        assert_eq!(gov_config.trust_score_uptime_weight.current, new_uptime_weight);
    });
}

/// Test multiple parameter updates in sequence
#[test]
fn multiple_parameter_updates() {
    new_test_ext().execute_with(|| {
        // Update multiple parameters
        let updates = vec![
            (ParameterType::EpochLength, 300u32.encode()),
            (ParameterType::ValidatorReward, 50000u128.encode()),
            (ParameterType::BlockAuthorshipBoost, 15u64.encode()),
            (ParameterType::MissedBlockPenalty, 8u64.encode()),
        ];
        
        for (param_type, encoded_value) in updates {
            let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
            assert_ok!(DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                param_type,
                bounded_value
            ));
        }
        
        // Verify all parameters were updated
        let config = DcfModule::governance_config();
        assert_eq!(config.epoch_length.current, 300);
        assert_eq!(config.validator_reward.current, 50000);
        assert_eq!(config.block_authorship_boost.current, 15);
        assert_eq!(config.missed_block_penalty.current, 8);
    });
}

/// Test parameter range bounds enforcement
#[test]
fn parameter_range_bounds_enforcement() {
    new_test_ext().execute_with(|| {
        let config = DcfModule::governance_config();
        
        // Test minimum stake parameter bounds
        let below_min = config.min_stake.min.saturating_sub(1u128);
        let encoded_value = below_min.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::MinStake,
                bounded_value
            ),
            Error::<Test>::ParameterOutOfRange
        );
        
        // Test maximum validators parameter bounds
        let above_max = config.max_validators.max + 1;
        let encoded_value = above_max.encode();
        let bounded_value = BoundedVec::try_from(encoded_value).unwrap();
        
        assert_err!(
            DcfModule::update_dcf_parameter(
                RuntimeOrigin::root(),
                ParameterType::MaxValidators,
                bounded_value
            ),
            Error::<Test>::ParameterOutOfRange
        );
    });
}

/// Test parameter governance event emission
#[test]
fn parameter_governance_events() {
    new_test_ext().execute_with(|| {
        // Advance to block 1 so events can be registered
        System::set_block_number(1);
        
        // Clear existing events
        System::reset_events();
        
        // Update a parameter
        let new_value = 250u32;
        let old_value = DcfModule::governance_config().epoch_length.current;
        let encoded_value = new_value.encode();
        let bounded_value = BoundedVec::try_from(encoded_value.clone()).unwrap();
        
        assert_ok!(DcfModule::update_dcf_parameter(
            RuntimeOrigin::root(),
            ParameterType::EpochLength,
            bounded_value
        ));
        
        // Verify event was emitted with correct details
        let events = System::events();
        assert_eq!(events.len(), 1);
        
        match &events[0].event {
            RuntimeEvent::DcfPallet(Event::DcfParameterUpdated {
                parameter,
                old_value: emitted_old_value,
                new_value: emitted_new_value,
            }) => {
                assert_eq!(*parameter, ParameterType::EpochLength);
                assert_eq!(emitted_old_value.clone().into_inner(), old_value.encode());
                assert_eq!(emitted_new_value.clone().into_inner(), encoded_value);
            },
            _ => panic!("Expected DcfParameterUpdated event"),
        }
    });
}