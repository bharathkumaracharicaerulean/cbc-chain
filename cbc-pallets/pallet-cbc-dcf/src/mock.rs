use frame_support::{
    parameter_types,
    traits::{ConstU32, Everything, PalletInfo as PalletInfoTrait},
    PalletId,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use sp_std::convert::{TryFrom, TryInto};
use crate::{self as pallet_cbc_dcf, *};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Dcf: pallet_cbc_dcf,
        Pos: pallet_cbc_pos,
        Poi: pallet_cbc_poi,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
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

parameter_types! {
    pub const DcfPalletId: PalletId = PalletId(*b"cbc/dcf\x00");
    pub const MaxValidators: u32 = 100;
    pub const DefaultPosWeight: u64 = 60;
    pub const DefaultPoiWeight: u64 = 40;
    pub const MinActiveValidators: u32 = 3;
    pub const MinValidatorScore: u32 = 10;
    pub const ValidatorScoreDecay: u32 = 1;
    pub const MaxSlashingCount: u32 = 3;
    pub const MinStake: u64 = 1000;
    pub const InferenceReward: u128 = 100;
    pub const ChallengeReward: u128 = 50;
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxValidators = MaxValidators;
    type DefaultPosWeight = DefaultPosWeight;
    type DefaultPoiWeight = DefaultPoiWeight;
    type MinActiveValidators = MinActiveValidators;
    type MinValidatorScore = MinValidatorScore;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxSlashingCount = MaxSlashingCount;
    type MinStake = MinStake;
    type Balance = u64;
    type WeightInfo = ();
}

impl pallet_cbc_pos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type MaxValidators = MaxValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxSlashingCount = MaxSlashingCount;
}

impl pallet_cbc_poi::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinInferenceConfidence = ConstU32<50>;
    type MaxInferenceAge = ConstU32<100>;
    type ChallengeWindow = ConstU32<10>;
    type InferenceReward = InferenceReward;
    type ChallengeReward = ChallengeReward;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
} 