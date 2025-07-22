#![cfg(test)]

use crate as pallet_cbc_dcf;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU32, ConstU64, ConstU128, Everything},
};
use sp_core::H256;
use frame_system as system;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use frame_system::pallet_prelude::BlockNumberFor;
use crate::pallet;

pub(crate) type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Dcf: pallet_cbc_dcf,
        PalletCbcPos: pallet_cbc_pos,
        PalletCbcPoi: pallet_cbc_poi,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub const MaxValidators: u32 = 10;
    pub const DefaultPosWeight: u64 = 50;
    pub const DefaultPoiWeight: u64 = 50;
    pub const MinActiveValidators: u32 = 1;
    pub const MinValidatorScore: u32 = 1;
    pub const ValidatorScoreDecay: u32 = 10;
    pub const MaxValidatorScore: u64 = 1000;
    pub const BlockAuthorshipBoost: u64 = 10;
    pub const MissedBlockPenalty: u64 = 5;
    pub const InferenceBoostLow: u64 = 1;
    pub const InferenceBoostMedium: u64 = 5;
    pub const InferenceBoostHigh: u64 = 10;
    pub const InferencePenaltyLow: u64 = 1;
    pub const InferencePenaltyMedium: u64 = 5;
    pub const InferencePenaltyHigh: u64 = 10;
    pub const MinStake: u64 = 1;
    pub const MaxEpochHistory: u32 = 24;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type RuntimeCall = RuntimeCall;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeTask = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<AccountId>;
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
}

impl pallet_cbc_pos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type MaxValidators = MaxValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxSlashingCount = ConstU32<3>;
    type MinStake = MinStake;
    type Balance = u64;
}

// Dummy PosInterface implementation for pallet_cbc_poi
pub struct DummyPosInterface;
impl pallet_cbc_poi::PosInterface<u64> for DummyPosInterface {
    fn boost_score(_validator: &u64, _weight: u32) -> frame_support::dispatch::DispatchResult { Ok(()) }
    fn slash_score(_validator: &u64, _weight: u32) -> frame_support::dispatch::DispatchResult { Ok(()) }
}

impl pallet_cbc_poi::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinInferenceConfidence = ConstU32<1>;
    type MaxInferenceAge = ConstU32<100>;
    type ChallengeWindow = ConstU32<10>;
    type InferenceReward = ConstU128<1>;
    type ChallengeReward = ConstU128<1>;
    type PosInterface = DummyPosInterface;
}

impl pallet_cbc_dcf::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxValidators = MaxValidators;
    type DefaultPosWeight = DefaultPosWeight;
    type DefaultPoiWeight = DefaultPoiWeight;
    type MinActiveValidators = MinActiveValidators;
    type MinValidatorScore = MinValidatorScore;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxValidatorScore = MaxValidatorScore;
    type BlockAuthorshipBoost = BlockAuthorshipBoost;
    type MissedBlockPenalty = MissedBlockPenalty;
    type InferenceBoostLow = InferenceBoostLow;
    type InferenceBoostMedium = InferenceBoostMedium;
    type InferenceBoostHigh = InferenceBoostHigh;
    type InferencePenaltyLow = InferencePenaltyLow;
    type InferencePenaltyMedium = InferencePenaltyMedium;
    type InferencePenaltyHigh = InferencePenaltyHigh;
    type MinStake = MinStake;
    type Balance = u64;
    type WeightInfo = ();
    type MaxEpochHistory = MaxEpochHistory;
}

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = new_test_ext_with_validators(vec![1, 2, 3]);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

// Helper function to advance blocks
pub(crate) fn run_to_block(n: u64) {
    while System::block_number() < n {
        <pallet::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(System::block_number());
        <frame_system::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        <frame_system::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_initialize(System::block_number());
        <pallet::Pallet<Test> as frame_support::traits::Hooks<BlockNumberFor<Test>>>::on_initialize(System::block_number());
    }
}

// Helper function to initialize test environment with validators
pub(crate) fn new_test_ext_with_validators(validators: Vec<AccountId>) -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap();

    // Initialize validators with default state
    let validator_scores = validators
        .iter()
        .map(|v| (*v, 100))
        .collect::<Vec<_>>();

    pallet_cbc_dcf::GenesisConfig::<Test> {
        validators: validators.clone(),
        validator_scores: validator_scores.into_iter().map(|(_, score)| score).collect(),
        current_epoch: 0,
        epoch_config: pallet_cbc_dcf::EpochConfig {
            blocks_per_epoch: 10,
            min_stake: 1,
            max_validators: 10,
        },
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // Add validators to storage
        <pallet_cbc_dcf::ActiveValidators<Test>>::put(
            frame_support::BoundedVec::truncate_from(validators.clone())
        );
        // Initialize validator state through public functions
        for validator in validators.iter() {
            let _ = Dcf::update_validator_stake_score(RuntimeOrigin::signed(*validator), 1);
            let _ = Dcf::update_validator_inference_score(RuntimeOrigin::signed(*validator), 2);
        }
    });
    ext
}