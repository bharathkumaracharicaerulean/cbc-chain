//! Mock runtime for CBC consensus tests

use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, ConstU8},
    weights::Weight,
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, DispatchResult, DispatchError,
};
use sp_core::H256;

// CBC consensus authority types
#[allow(dead_code)]
pub type AuthorityId = u64;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the consensus module
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        DcfPallet: pallet_cbc_dcf,
        PalletCbcPos: pallet_cbc_pos,
        PalletCbcPoi: pallet_cbc_poi,
        PalletCbcGovernance: pallet_cbc_governance,
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
    type Hash = H256;
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
    type MaxReserves = ConstU32<50>;
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
    pub const MinActiveValidators: u32 = 3;
    pub const ValidatorScoreDecay: u32 = 10;
    pub const MinInferenceConfidence: u32 = 10;
    pub const MaxInferenceAge: u32 = 10;
    pub const ChallengeWindow: u32 = 5;
    pub const InferenceReward: u128 = 1000;
    pub const ChallengeReward: u128 = 500;
}

impl pallet_cbc_pos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type Currency = Balances;
    type MinStake = MinStake;
    type MaxValidators = MaxValidators;
    type MaxSlashingCount = MaxSlashingCount;
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type WeightInfo = ();

    type LeaveCooldown = ConstU32<10>;
    type ValidatorReward = ConstU128<10000>;
    type SlashPercent = ConstU32<10>;
    type MaxSlashPerEpoch = ConstU128<50000>;
    type MaxSlashPerValidator = ConstU128<20000>;
    type MaxRewardPerEpoch = ConstU128<30000>;
    type MaxRewardPerValidator = ConstU128<10000>;
    type SlashPenaltyDivisor = ConstU64<1000>;
    type MaxSlashPenalty = ConstU64<50>;
    type RewardBoostDivisor = ConstU64<1000>;
    type MaxRewardBoost = ConstU64<20>;
    type HighPerformanceScore = ConstU64<80>;
    type TopPerformerPercentage = ConstU32<20>;
    type ValidatorHandler = ();
}

impl pallet_cbc_poi::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MinInferenceConfidence = MinInferenceConfidence;
    type MaxInferenceAge = MaxInferenceAge;
    type ChallengeWindow = ChallengeWindow;
    type InferenceReward = InferenceReward;
    type ChallengeReward = ChallengeReward;
    type PosInterface = MockPosInterface;
    type DcfInterface = DcfPallet;
    type WeightInfo = ();

    type InferenceBoostLow = ConstU64<2>;
    type InferenceBoostMedium = ConstU64<5>;
    type InferenceBoostHigh = ConstU64<10>;
    type InferencePenaltyLow = ConstU64<3>;
    type InferencePenaltyMedium = ConstU64<7>;
    type InferencePenaltyHigh = ConstU64<15>;
    type InferenceConfidenceThresholdLow = ConstU32<70>;
    type InferenceConfidenceThresholdHigh = ConstU32<90>;
    type MaxValidatorScore = ConstU64<10000>;
    type PercentagePrecision = ConstU32<10000>;
    type OffchainWorkerInterval = ConstU32<1>;
    type MaxValidatorIterationWeight = ConstU64<1000000>;
    type MaxLoopIterations = ConstU32<10>;
}

pub struct MockProposalExecutor;
impl pallet_cbc_governance::ProposalExecutor<u64, u128> for MockProposalExecutor {
    fn slash_validator(_validator: &u64, _amount: u128) -> DispatchResult { Ok(()) }
    fn reward_validator(_validator: &u64, _amount: u128) -> DispatchResult { Ok(()) }
    fn eject_validator(_validator: &u64, _reason: pallet_cbc_governance::EjectionReason) -> DispatchResult { Ok(()) }
    fn add_validator(_validator: &u64) -> DispatchResult { Ok(()) }
    fn remove_validator(_validator: &u64) -> DispatchResult { Ok(()) }
}

pub struct MockValidatorProvider;
impl pallet_cbc_governance::ValidatorProvider<u64, Weight> for MockValidatorProvider {
    fn active_validators() -> Vec<u64> { vec![1, 2, 3] }
    fn is_private_chain_mode() -> bool { false }
    fn validate_governance_in_private_mode(_proposer: &u64) -> DispatchResult { Ok(()) }
    fn validate_proposal_in_private_mode(_proposer: &u64, _target: &u64) -> DispatchResult { Ok(()) }
    fn check_rate_limits(_proposer: &u64, _op_type: u8, _weight: Weight) -> Result<(), (DispatchError, Option<(u8, u8, u32, u32)>)> { Ok(()) }
    fn record_operation(_proposer: &u64, _op_type: u8) {}
}

impl pallet_cbc_governance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type ProposalExecutor = MockProposalExecutor;
    type ValidatorProvider = MockValidatorProvider;
}

parameter_types! {
    pub const MaxEpochHistory: u32 = 24;
    pub const DefaultPosWeight: u64 = 6000;
    pub const DefaultPoiWeight: u64 = 4000;
    pub const BlockAuthorshipBoost: u64 = 10;
    pub const MissedBlockPenalty: u64 = 5;
    pub const InferenceBoostLow: u64 = 2;
    pub const InferenceBoostMedium: u64 = 5;
    pub const InferenceBoostHigh: u64 = 10;
    pub const InferencePenaltyLow: u64 = 1;
    pub const InferencePenaltyMedium: u64 = 3;
    pub const InferencePenaltyHigh: u64 = 7;
}

pub struct DummyWeightFreezer;
impl pallet_cbc_dcf::traits::WeightFreezer<u64> for DummyWeightFreezer {
    fn freeze_epoch_weights(_epoch: u32, _validators: &[(u64, u128, u128)]) {}
}

pub struct DummyDvfFinalizedBlockProvider;
impl pallet_cbc_dcf::traits::DvfFinalizedBlockProvider for DummyDvfFinalizedBlockProvider {
    fn dvf_finalized_block() -> u32 { 0 }
}

impl pallet_cbc_dcf::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type MaxEpochHistory = MaxEpochHistory;
    type DefaultPosWeight = DefaultPosWeight;
    type DefaultPoiWeight = DefaultPoiWeight;
    type BlockAuthorshipBoost = BlockAuthorshipBoost;
    type MissedBlockPenalty = MissedBlockPenalty;
    type WeightInfo = ();

    type MaxValidatorHistorySize = ConstU32<10>;
    type MaxValidatorNameSize = ConstU32<32>;
    type MaxRuntimeApiBoundedVecSize = ConstU32<100>;
    type MaxProposalActionBoundedVecSize = ConstU32<100>;
    type AuthorNotActiveErrorCode = ConstU8<1>;
    type AuthorMismatchErrorCode = ConstU8<2>;
    type MaxInactiveEpochs = ConstU32<5>;
    type ScoreDecayInterval = ConstU32<10>;
    type ParticipationUpdateInterval = ConstU32<100>;
    type UnderperformanceCheckInterval = ConstU32<50>;
    type ValidatorProposalInterval = ConstU32<200>;
    type HealthMetricsInterval = ConstU32<1000>;
    type OffchainWorkerTimeout = ConstU64<5000>;
    type EstimatedBlockTime = ConstU64<6000>;
    type MinPerformanceScore = ConstU64<30>;
    type MinParticipationRate = ConstU32<50>;
    type HighParticipationRate = ConstU32<90>;
    type MaxMissedBlocks = ConstU32<10>;
    type MaxMissedBlocksHigh = ConstU32<2>;
    type HealthyValidatorScore = ConstU64<50>;
    type HealthyParticipationRate = ConstU32<80>;
    type HealthyMissedBlocksMax = ConstU32<5>;
    type ScoreChangeThreshold = ConstU64<1000>;
    type ScoreChangePercentage = ConstU32<10>;
    type ScoreImprovementThreshold = ConstU64<1000>;
    type ScoreImprovementPercentage = ConstU32<10>;
    type MaxPosContribution = ConstU32<90>;
    type MaxPoiContribution = ConstU32<90>;
    type ImbalanceWarningThreshold = ConstU32<85>;
    type MaxValidatorNameLength = ConstU32<32>;
    type MaxValidatorWebsiteLength = ConstU32<64>;
    type MaxValidatorContactLength = ConstU32<64>;
    type MaxValidatorDescriptionLength = ConstU32<128>;
    type MaxValidatorLocationLength = ConstU32<32>;
    type MaxPerformanceHistoryLength = ConstU32<100>;
    type MaxValidatorHistoryLength = ConstU32<10>;
    type MaxCommissionRate = ConstU32<10000>;
    type FullPercentage = ConstU32<100>;
    type HighPerformancePercentage = ConstU32<80>;
    type BaseRewardPercentage = ConstU32<60>;
    type PerformanceRewardPercentage = ConstU32<25>;
    type TopPerformerRewardPercentage = ConstU32<15>;
    type MinTrustScore = ConstU64<0>;
    type MaxTrustScoreGrowthRate = ConstU32<20>;
    type MaxTrustScoreDecayRate = ConstU32<10>;
    type TrustScoreStabilityFactor = ConstU32<5>;
    type TrustScoreUptimeWeight = ConstU64<4000>;
    type TrustScoreInferenceWeight = ConstU64<4000>;
    type TrustScoreSlashingWeight = ConstU64<2000>;
    type MaxTrustScore = ConstU64<10000>;
    type MaxEvidenceLength = ConstU32<1000>;
    type MisbehaviorSlashThreshold = ConstU32<3>;
    type EpochLength = ConstU32<2400>;

    type LeaveRequestCheckInterval = ConstU32<100>;
    type MetricsUpdateInterval = ConstU32<10>;
    type ScoreRefreshInterval = ConstU32<50>;
    type DetailedLoggingInterval = ConstU32<100>;
    type ImbalanceCheckInterval = ConstU32<100>;
    type TopValidatorsDisplayCount = ConstU32<5>;
    type HealthCheckSampleSize = ConstU32<5>;

    type WeightFreezer = DummyWeightFreezer;
    type DvfFinalizedBlockProvider = DummyDvfFinalizedBlockProvider;
    type ValidatorRegistry = MockValidatorRegistry;
}

pub struct MockValidatorRegistry;
impl pallet_cbc_dcf::traits::ValidatorRegistryProvider<u64, u128, u64> for MockValidatorRegistry {
    fn get_active_validators() -> Vec<u64> {
        vec![1, 2, 3]
    }
    fn get_validator_profile(_validator: &u64) -> Option<pallet_cbc_dcf::ValidatorProfile<u64, u128, u64>> { None }
    fn get_validator_status(_validator: &u64) -> Option<pallet_cbc_dcf::ValidatorStatus> { None }
    fn get_validator_set() -> Vec<u64> { vec![1, 2, 3] }
    fn update_validator_set(_set: Vec<u64>) {}
    fn is_validator_active(_validator: &u64) -> bool { true }
    fn eject_validator(_validator: &u64, _reason: pallet_cbc_dcf::EjectionReason) -> DispatchResult { Ok(()) }
    fn get_validator_state(_validator: &u64) -> Option<pallet_cbc_dcf::traits::ValidatorState> { None }
    fn update_validator_state(_validator: &u64, _state: pallet_cbc_dcf::traits::ValidatorState) {}
    fn contains_validator_state(_validator: &u64) -> bool { false }
    fn remove_validator_state(_validator: &u64) {}
    fn get_validator_name(_validator: &u64) -> Option<Vec<u8>> { None }
    fn set_validator_name(_validator: &u64, _name: Vec<u8>) {}
    fn get_validator_metadata(_validator: &u64) -> Option<pallet_cbc_dcf::ValidatorMetadataInfo> { None }
    fn set_validator_metadata(_validator: &u64, _metadata: pallet_cbc_dcf::ValidatorMetadataInfo) {}
    fn remove_validator_metadata(_validator: &u64) {}
    fn get_validator_detailed_cooldown_status(_validator: &u64) -> Option<(u32, bool)> { None }
    fn get_validator_leave_request(_validator: &u64) -> Option<u32> { None }
    fn get_pending_actions() -> Vec<(u64, pallet_cbc_dcf::ValidatorAction)> { Vec::new() }
    fn remove_pending_action(_validator: &u64) {}
    fn add_pending_action(_validator: &u64, _action: pallet_cbc_dcf::ValidatorAction) {}
    fn update_active_validators(_active: Vec<u64>) {}
    fn get_validator_join_time(_validator: &u64) -> Option<u32> { None }
    fn set_validator_join_time(_validator: &u64, _val: u32) {}
    fn remove_validator_join_time(_validator: &u64) {}
    fn set_validator_leave_request(_validator: &u64, _val: u32) {}
    fn remove_validator_leave_request(_validator: &u64) {}
    fn get_recently_removed_validator(_validator: &u64) -> Option<u32> { None }
    fn set_recently_removed_validator(_validator: &u64, _val: u32) {}
    fn remove_recently_removed_validator(_validator: &u64) {}
    fn get_validator_performance_history(_validator: &u64) -> Vec<pallet_cbc_dcf::PerformanceRecord> { Vec::new() }
    fn set_validator_performance_history(_validator: &u64, _history: Vec<pallet_cbc_dcf::PerformanceRecord>) {}
    fn remove_validator_performance_history(_validator: &u64) {}
    fn get_validator_last_seen(_validator: &u64) -> u32 { 0 }
    fn set_validator_last_seen(_validator: &u64, _val: u32) {}
    fn remove_validator_last_seen(_validator: &u64) {}
    fn get_validator_blocks_authored(_validator: &u64) -> u32 { 0 }
    fn set_validator_blocks_authored(_validator: &u64, _val: u32) {}
    fn remove_validator_blocks_authored(_validator: &u64) {}
    fn get_validator_blocks_missed(_validator: &u64) -> u32 { 0 }
    fn set_validator_blocks_missed(_validator: &u64, _val: u32) {}
    fn remove_validator_blocks_missed(_validator: &u64) {}
}

// Mock interfaces for CBC pallets
pub struct MockPosInterface;
impl pallet_cbc_poi::PosInterface<u64> for MockPosInterface {
    fn boost_score(_validator: &u64, _weight: u32) -> DispatchResult {
        Ok(())
    }
    
    fn slash_score(_validator: &u64, _weight: u32) -> DispatchResult {
        Ok(())
    }

    fn get_active_validators() -> Vec<u64> {
        vec![1, 2, 3]
    }
}

#[allow(dead_code)]
pub struct MockDcfInterface;
impl pallet_cbc_poi::DcfInterface<u64> for MockDcfInterface {
    fn record_inference_activity(_validator: &u64) -> DispatchResult {
        Ok(())
    }
    fn eject_validator(_validator: &u64, _reason: &'static str) -> DispatchResult {
        Ok(())
    }
    fn update_final_score(_validator: &u64) -> DispatchResult {
        Ok(())
    }
}

// Build genesis storage according to mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        let active: frame_support::BoundedVec<u64, MaxValidators> = vec![1, 2, 3].try_into().unwrap();
        pallet_cbc_dcf::ActiveValidators::<Test>::put(active);
    });
    ext
}