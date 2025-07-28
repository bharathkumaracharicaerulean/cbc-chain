//! Mock runtime for DCF pallet tests

use super::*;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128},
    weights::Weight,
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DcfPallet: crate,
        PalletCbcPos: pallet_cbc_pos,
        PalletCbcPoi: pallet_cbc_poi,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

impl pallet_balances::Config for Test {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinStake: u128 = 1000;
    pub const MaxValidators: u32 = 100;
    pub const MaxSlashingCount: u32 = 3;
    pub const MinValidatorScore: u32 = 50;
    pub const ValidatorScoreDecay: u32 = 10;
    pub const MinInferenceConfidence: u32 = 80;
    pub const MaxInferenceAge: u32 = 10;
    pub const ChallengeWindow: u32 = 5;
    pub const InferenceReward: u128 = 1000;
    pub const ChallengeReward: u128 = 500;
}

impl pallet_cbc_pos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type MinStake = MinStake;
    type MaxValidators = MaxValidators;
    type MaxSlashingCount = MaxSlashingCount;
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type WeightInfo = ();
}

impl pallet_cbc_poi::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MinInferenceConfidence = MinInferenceConfidence;
    type MaxInferenceAge = MaxInferenceAge;
    type ChallengeWindow = ChallengeWindow;
    type InferenceReward = InferenceReward;
    type ChallengeReward = ChallengeReward;
    type PosInterface = MockPosInterface;
    type WeightInfo = ();
}

parameter_types! {
    pub const DcfMaxValidators: u32 = 100;
    pub const MaxEpochHistory: u32 = 24;
    pub const DefaultPosWeight: u64 = 60;
    pub const DefaultPoiWeight: u64 = 40;
    pub const MinActiveValidators: u32 = 3;
    pub const DcfMinValidatorScore: u32 = 50;
    pub const DcfValidatorScoreDecay: u32 = 10;
    pub const MaxValidatorScore: u64 = 100;
    pub const BlockAuthorshipBoost: u64 = 10;
    pub const MissedBlockPenalty: u64 = 5;
    pub const InferenceBoostLow: u64 = 2;
    pub const InferenceBoostMedium: u64 = 5;
    pub const InferenceBoostHigh: u64 = 10;
    pub const InferencePenaltyLow: u64 = 1;
    pub const InferencePenaltyMedium: u64 = 3;
    pub const InferencePenaltyHigh: u64 = 7;
    pub const DcfMinStake: u128 = 1000;
}

// Mock PosInterface implementation
pub struct MockPosInterface;
impl pallet_cbc_poi::PosInterface<u64> for MockPosInterface {
    fn boost_score(_validator: &u64, _weight: u32) -> DispatchResult {
        Ok(())
    }
    fn slash_score(_validator: &u64, _weight: u32) -> DispatchResult {
        Ok(())
    }
}

// Mock weight info
pub struct MockWeightInfo;
impl WeightInfo for MockWeightInfo {
    fn on_initialize() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn offchain_worker() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn update_validator_stake_score() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn update_validator_inference_score() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn update_consensus_weights() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn set_governance_mode() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn sudo_advance_epoch() -> Weight {
        Weight::from_parts(50_000, 0)
    }
    fn submit_proposal() -> Weight {
        Weight::from_parts(20_000, 0)
    }
    fn vote_proposal() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    fn execute_proposal() -> Weight {
        Weight::from_parts(30_000, 0)
    }
    fn join_validator_set() -> Weight {
        Weight::from_parts(25_000, 0)
    }
    fn leave_validator_set() -> Weight {
        Weight::from_parts(25_000, 0)
    }
    fn set_validator_name() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    fn apply_offchain_poi_scores() -> Weight {
        Weight::from_parts(40_000, 0)
    }
    fn on_initialize_with_validators(v: u32) -> Weight {
        Weight::from_parts(10_000u64.saturating_mul(v as u64), 0)
    }
    fn apply_score_decay_multiple(v: u32) -> Weight {
        Weight::from_parts(5_000u64.saturating_mul(v as u64), 0)
    }
    fn validator_set_operations(v: u32) -> Weight {
        Weight::from_parts(3_000u64.saturating_mul(v as u64), 0)
    }
    fn runtime_api_calls(v: u32) -> Weight {
        Weight::from_parts(2_000u64.saturating_mul(v as u64), 0)
    }
    fn epoch_transition_multiple(v: u32) -> Weight {
        Weight::from_parts(15_000u64.saturating_mul(v as u64), 0)
    }
    fn governance_with_multiple_voters(v: u32) -> Weight {
        Weight::from_parts(8_000u64.saturating_mul(v as u64), 0)
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxValidators = DcfMaxValidators;
    type MaxEpochHistory = MaxEpochHistory;
    type DefaultPosWeight = DefaultPosWeight;
    type DefaultPoiWeight = DefaultPoiWeight;
    type MinActiveValidators = MinActiveValidators;
    type MinValidatorScore = DcfMinValidatorScore;
    type ValidatorScoreDecay = DcfValidatorScoreDecay;
    type MaxValidatorScore = MaxValidatorScore;
    type BlockAuthorshipBoost = BlockAuthorshipBoost;
    type MissedBlockPenalty = MissedBlockPenalty;
    type InferenceBoostLow = InferenceBoostLow;
    type InferenceBoostMedium = InferenceBoostMedium;
    type InferenceBoostHigh = InferenceBoostHigh;
    type InferencePenaltyLow = InferencePenaltyLow;
    type InferencePenaltyMedium = InferencePenaltyMedium;
    type InferencePenaltyHigh = InferencePenaltyHigh;
    type MinStake = DcfMinStake;
    type Balance = u128;
    type WeightInfo = MockWeightInfo;
}



// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),
            (2, 10000),
            (3, 10000),
            (4, 10000),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    crate::GenesisConfig::<Test> {
        validators: vec![1, 2, 3],
        validator_scores: vec![60, 70, 80],
        current_epoch: 0,
        epoch_config: EpochConfig {
            blocks_per_epoch: 10,
            min_stake: 1000,
            max_validators: 100,
        },
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}