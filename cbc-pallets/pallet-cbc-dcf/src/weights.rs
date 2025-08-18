#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{Weight, constants::RocksDbWeight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for the pallet.
pub trait WeightInfo {
    fn on_initialize() -> Weight;
    fn offchain_worker() -> Weight;
    fn update_validator_stake_score() -> Weight;
    fn update_validator_inference_score() -> Weight;
    fn update_consensus_weights() -> Weight;
    fn set_governance_mode() -> Weight;
    fn sudo_advance_epoch() -> Weight;
    fn submit_proposal() -> Weight;
    fn vote_proposal() -> Weight;
    fn execute_proposal() -> Weight;
    fn join_validator_set() -> Weight;
    fn leave_validator_set() -> Weight;
    fn set_validator_name() -> Weight;
    fn apply_offchain_poi_scores() -> Weight;
    fn on_initialize_with_validators(v: u32) -> Weight;
    fn apply_score_decay_multiple(v: u32) -> Weight;
    fn validator_set_operations(v: u32) -> Weight;
    fn runtime_api_calls(v: u32) -> Weight;
    fn epoch_transition_multiple(v: u32) -> Weight;
    fn governance_with_multiple_voters(v: u32) -> Weight;
    
    // New benchmark functions
    fn join_validators(v: u32) -> Weight;
    fn leave_validators(v: u32) -> Weight;
    fn slash_validator() -> Weight;
    fn execute_proposals_with_transfers() -> Weight;
    fn epoch_auto_transition() -> Weight;
    fn validator_lifecycle_operations(v: u32) -> Weight;
    fn misbehavior_reporting_and_slashing() -> Weight;
    fn increase_validator_stake() -> Weight;
    fn decrease_validator_stake() -> Weight;
    fn slash_multiple_validators(v: u32) -> Weight;
    fn slash_validator_percentage() -> Weight;
    fn set_validator_metadata() -> Weight;
    fn propose_reward_multiple_validators(v: u32) -> Weight;
}

/// Weights for the pallet using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: PosWeight (r:1 w:0)
    /// Storage: PoiWeight (r:1 w:0)
    fn update_validator_stake_score() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: PosWeight (r:1 w:0)
    /// Storage: PoiWeight (r:1 w:0)
    fn update_validator_inference_score() -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: PosWeight (r:0 w:1)
    /// Storage: PoiWeight (r:0 w:1)
    fn update_consensus_weights() -> Weight {
        Weight::from_parts(8_000, 0)
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: GovernanceModeEnabled (r:0 w:1)
    fn set_governance_mode() -> Weight {
        Weight::from_parts(5_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: GovernanceModeEnabled (r:1 w:0)
    /// Storage: CurrentEpoch (r:1 w:1)
    /// Storage: ActiveValidators (r:1 w:1)
    /// Storage: PendingValidatorActions (r:0 w:10)
    fn sudo_advance_epoch() -> Weight {
        Weight::from_parts(50_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(12))
    }

    /// Storage: GovernanceModeEnabled (r:1 w:0)
    /// Storage: NextProposalId (r:1 w:1)
    /// Storage: Proposals (r:0 w:1)
    fn submit_proposal() -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: ProposalVotes (r:1 w:1)
    fn vote_proposal() -> Weight {
        Weight::from_parts(18_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: ActiveValidators (r:1 w:1)
    fn execute_proposal() -> Weight {
        Weight::from_parts(35_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: PendingValidatorActions (r:1 w:1)
    /// Storage: ValidatorStates (r:1 w:0)
    fn join_validator_set() -> Weight {
        Weight::from_parts(25_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: PendingValidatorActions (r:1 w:1)
    fn leave_validator_set() -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ValidatorStates (r:1 w:0)
    /// Storage: ValidatorNames (r:0 w:1)
    fn set_validator_name() -> Weight {
        Weight::from_parts(12_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: ValidatorSet (r:1 w:0)
    /// Storage: ValidatorStates (r:10 w:10)
    fn apply_offchain_poi_scores() -> Weight {
        Weight::from_parts(40_000, 0)
            .saturating_add(T::DbWeight::get().reads(11))
            .saturating_add(T::DbWeight::get().writes(10))
    }

    /// Storage: ValidatorSet (r:1 w:0)
    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: CurrentEpoch (r:1 w:0)
    /// Storage: EpochConfigStorage (r:1 w:0)
    /// Storage: ValidatorStates (r:v w:v)
    fn on_initialize_with_validators(v: u32) -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(5_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    /// Storage: ValidatorStates (r:v w:v)
    fn apply_score_decay_multiple(v: u32) -> Weight {
        Weight::from_parts(10_000, 0)
            .saturating_add(Weight::from_parts(8_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    /// Storage: ValidatorSet (r:1 w:0)
    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: ValidatorStates (r:v w:0)
    fn validator_set_operations(v: u32) -> Weight {
        Weight::from_parts(15_000, 0)
            .saturating_add(Weight::from_parts(3_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
    }

    /// Storage: ValidatorSet (r:1 w:0)
    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: CurrentEpoch (r:1 w:0)
    /// Storage: PosWeight (r:1 w:0)
    /// Storage: PoiWeight (r:1 w:0)
    /// Storage: ValidatorStates (r:v w:0)
    fn runtime_api_calls(v: u32) -> Weight {
        Weight::from_parts(25_000, 0)
            .saturating_add(Weight::from_parts(4_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(v.into())))
    }

    /// Storage: CurrentEpoch (r:1 w:1)
    /// Storage: ActiveValidators (r:1 w:1)
    /// Storage: PendingValidatorActions (r:v w:v)
    /// Storage: EpochHistories (r:1 w:1)
    fn epoch_transition_multiple(v: u32) -> Weight {
        Weight::from_parts(40_000, 0)
            .saturating_add(Weight::from_parts(6_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: ProposalVotes (r:v w:v)
    fn governance_with_multiple_voters(v: u32) -> Weight {
        Weight::from_parts(20_000, 0)
            .saturating_add(Weight::from_parts(12_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    fn on_initialize() -> Weight {
        Self::on_initialize_with_validators(10)
    }

    fn offchain_worker() -> Weight {
        Weight::from_parts(50_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: PendingValidatorActions (r:v w:v)
    /// Storage: ValidatorStates (r:v w:0)
    fn join_validators(v: u32) -> Weight {
        Weight::from_parts(30_000, 0)
            .saturating_add(Weight::from_parts(15_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    /// Storage: ActiveValidators (r:1 w:0)
    /// Storage: PendingValidatorActions (r:v w:v)
    fn leave_validators(v: u32) -> Weight {
        Weight::from_parts(25_000, 0)
            .saturating_add(Weight::from_parts(12_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(v.into())))
    }

    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:2 w:2)
    fn slash_validator() -> Weight {
        Weight::from_parts(45_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: Proposals (r:1 w:1)
    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:3 w:3)
    fn execute_proposals_with_transfers() -> Weight {
        Weight::from_parts(65_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    /// Storage: CurrentEpoch (r:1 w:1)
    /// Storage: EpochConfigStorage (r:1 w:0)
    /// Storage: ActiveValidators (r:1 w:1)
    /// Storage: PendingValidatorActions (r:10 w:10)
    /// Storage: ValidatorStates (r:10 w:10)
    /// Storage: EpochHistories (r:1 w:1)
    fn epoch_auto_transition() -> Weight {
        Weight::from_parts(120_000, 0)
            .saturating_add(T::DbWeight::get().reads(24))
            .saturating_add(T::DbWeight::get().writes(23))
    }

    /// Storage: ValidatorStates (r:v w:v)
    /// Storage: PendingValidatorActions (r:v w:v)
    /// Storage: CurrentEpoch (r:1 w:1)
    /// Storage: ActiveValidators (r:1 w:1)
    fn validator_lifecycle_operations(v: u32) -> Weight {
        Weight::from_parts(50_000, 0)
            .saturating_add(Weight::from_parts(25_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(v.into())))
    }

    /// Storage: MisbehaviorReports (r:1 w:1)
    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:2 w:2)
    fn misbehavior_reporting_and_slashing() -> Weight {
        Weight::from_parts(55_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:2 w:2)
    fn increase_validator_stake() -> Weight {
        Weight::from_parts(35_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:2 w:2)
    fn decrease_validator_stake() -> Weight {
        Weight::from_parts(35_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: ValidatorStates (r:v w:v)
    /// Storage: Currency operations (r:2*v w:2*v)
    fn slash_multiple_validators(v: u32) -> Weight {
        Weight::from_parts(40_000, 0)
            .saturating_add(Weight::from_parts(30_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(v.into())))
    }

    /// Storage: ValidatorStates (r:1 w:1)
    /// Storage: Currency operations (r:2 w:2)
    fn slash_validator_percentage() -> Weight {
        Weight::from_parts(40_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Storage: ValidatorStates (r:1 w:0)
    /// Storage: ValidatorMetadataStorage (r:0 w:1)
    fn set_validator_metadata() -> Weight {
        Weight::from_parts(25_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Storage: GovernanceModeEnabled (r:1 w:0)
    /// Storage: NextProposalId (r:1 w:1)
    /// Storage: Proposals (r:0 w:1)
    /// Storage: ValidatorStates (r:v w:0)
    fn propose_reward_multiple_validators(v: u32) -> Weight {
        Weight::from_parts(30_000, 0)
            .saturating_add(Weight::from_parts(8_000, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(v.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn update_validator_stake_score() -> Weight { Weight::from_parts(15_000, 0) }
    fn update_validator_inference_score() -> Weight { Weight::from_parts(15_000, 0) }
    fn update_consensus_weights() -> Weight { Weight::from_parts(8_000, 0) }
    fn set_governance_mode() -> Weight { Weight::from_parts(5_000, 0) }
    fn sudo_advance_epoch() -> Weight { Weight::from_parts(50_000, 0) }
    fn submit_proposal() -> Weight { Weight::from_parts(20_000, 0) }
    fn vote_proposal() -> Weight { Weight::from_parts(18_000, 0) }
    fn execute_proposal() -> Weight { Weight::from_parts(35_000, 0) }
    fn join_validator_set() -> Weight { Weight::from_parts(25_000, 0) }
    fn leave_validator_set() -> Weight { Weight::from_parts(20_000, 0) }
    fn set_validator_name() -> Weight { Weight::from_parts(12_000, 0) }
    fn apply_offchain_poi_scores() -> Weight { Weight::from_parts(40_000, 0) }
    fn on_initialize_with_validators(v: u32) -> Weight { 
        Weight::from_parts(20_000, 0).saturating_add(Weight::from_parts(5_000, 0).saturating_mul(v.into()))
    }
    fn apply_score_decay_multiple(v: u32) -> Weight { 
        Weight::from_parts(10_000, 0).saturating_add(Weight::from_parts(8_000, 0).saturating_mul(v.into()))
    }
    fn validator_set_operations(v: u32) -> Weight { 
        Weight::from_parts(15_000, 0).saturating_add(Weight::from_parts(3_000, 0).saturating_mul(v.into()))
    }
    fn runtime_api_calls(v: u32) -> Weight { 
        Weight::from_parts(25_000, 0).saturating_add(Weight::from_parts(4_000, 0).saturating_mul(v.into()))
    }
    fn epoch_transition_multiple(v: u32) -> Weight { 
        Weight::from_parts(40_000, 0).saturating_add(Weight::from_parts(6_000, 0).saturating_mul(v.into()))
    }
    fn governance_with_multiple_voters(v: u32) -> Weight { 
        Weight::from_parts(20_000, 0).saturating_add(Weight::from_parts(12_000, 0).saturating_mul(v.into()))
    }
    fn on_initialize() -> Weight { Weight::from_parts(50_000, 0) }
    fn offchain_worker() -> Weight { Weight::from_parts(50_000, 0) }
    
    // New benchmark implementations for backwards compatibility
    fn join_validators(v: u32) -> Weight { 
        Weight::from_parts(30_000, 0).saturating_add(Weight::from_parts(15_000, 0).saturating_mul(v.into()))
    }
    fn leave_validators(v: u32) -> Weight { 
        Weight::from_parts(25_000, 0).saturating_add(Weight::from_parts(12_000, 0).saturating_mul(v.into()))
    }
    fn slash_validator() -> Weight { Weight::from_parts(45_000, 0) }
    fn execute_proposals_with_transfers() -> Weight { Weight::from_parts(65_000, 0) }
    fn epoch_auto_transition() -> Weight { Weight::from_parts(120_000, 0) }
    fn validator_lifecycle_operations(v: u32) -> Weight { 
        Weight::from_parts(50_000, 0).saturating_add(Weight::from_parts(25_000, 0).saturating_mul(v.into()))
    }
    fn misbehavior_reporting_and_slashing() -> Weight { Weight::from_parts(55_000, 0) }
    fn increase_validator_stake() -> Weight { Weight::from_parts(35_000, 0) }
    fn decrease_validator_stake() -> Weight { Weight::from_parts(35_000, 0) }
    fn slash_multiple_validators(v: u32) -> Weight { 
        Weight::from_parts(40_000, 0).saturating_add(Weight::from_parts(30_000, 0).saturating_mul(v.into()))
    }
    fn slash_validator_percentage() -> Weight { Weight::from_parts(40_000, 0) }
    fn set_validator_metadata() -> Weight { Weight::from_parts(25_000, 0) }
    fn propose_reward_multiple_validators(v: u32) -> Weight { 
        Weight::from_parts(30_000, 0).saturating_add(Weight::from_parts(8_000, 0).saturating_mul(v.into()))
    }
}