//! Mock runtime for DCF pallet tests

use super::*;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64, ConstU128, ConstU8},
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
    type DcfInterface = crate::Pallet<Test>;
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

parameter_types! {
    pub const DcfMaxValidators: u32 = 100;
    pub const MaxEpochHistory: u32 = 24;
    pub const DefaultPosWeight: u64 = 6000;
    pub const DefaultPoiWeight: u64 = 4000;
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
    fn get_active_validators() -> Vec<u64> {
        vec![1, 2, 3]
    }
}

// Mock DcfInterface implementation for PoI pallet
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

pub struct MockProposalExecutor;
impl pallet_cbc_governance::ProposalExecutor<u64, u128> for MockProposalExecutor {
    fn slash_validator(_validator: &u64, _amount: u128) -> DispatchResult { Ok(()) }
    fn reward_validator(_validator: &u64, _amount: u128) -> DispatchResult { Ok(()) }
    fn eject_validator(_validator: &u64, _reason: pallet_cbc_governance::EjectionReason) -> DispatchResult { Ok(()) }
    fn add_validator(_validator: &u64) -> DispatchResult { Ok(()) }
    fn remove_validator(_validator: &u64) -> DispatchResult { Ok(()) }
}

impl pallet_cbc_governance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type ProposalExecutor = DcfPallet;
    type ValidatorProvider = DcfPallet;
    type WeightInfo = ();
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
    fn join_validators() -> Weight { Weight::from_parts(10_000, 0) }
    fn leave_validators() -> Weight { Weight::from_parts(10_000, 0) }
    fn slash_validator() -> Weight { Weight::from_parts(10_000, 0) }
    fn cancel_leave_request() -> Weight { Weight::from_parts(10_000, 0) }
    fn report_validator_misbehavior() -> Weight { Weight::from_parts(10_000, 0) }
    fn simulate_inference() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_slash_validator() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_reward_validator() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_eject_validator() -> Weight { Weight::from_parts(10_000, 0) }
    fn increase_validator_stake() -> Weight { Weight::from_parts(10_000, 0) }
    fn decrease_validator_stake() -> Weight { Weight::from_parts(10_000, 0) }
    fn slash_validator_percentage() -> Weight { Weight::from_parts(10_000, 0) }
    fn slash_multiple_validators() -> Weight { Weight::from_parts(10_000, 0) }
    fn execute_proposals() -> Weight { Weight::from_parts(10_000, 0) }
    fn epoch_transition() -> Weight { Weight::from_parts(10_000, 0) }
    fn set_validator_metadata() -> Weight { Weight::from_parts(10_000, 0) }
    fn update_validator_activity() -> Weight { Weight::from_parts(10_000, 0) }
    fn distribute_epoch_rewards() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_default_reward_validator() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_default_reward_multiple_validators() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_reward_all_active_validators() -> Weight { Weight::from_parts(10_000, 0) }
    fn propose_reward_multiple_validators(_v: u32) -> Weight { Weight::from_parts(10_000, 0) }
    fn update_rate_limit_config() -> Weight { Weight::from_parts(10_000, 0) }
    fn validate_genesis_configuration() -> Weight { Weight::from_parts(10_000, 0) }
    fn dry_run_genesis_configuration() -> Weight { Weight::from_parts(10_000, 0) }
    fn enable_private_chain() -> Weight { Weight::from_parts(10_000, 0) }
    fn disable_private_chain() -> Weight { Weight::from_parts(10_000, 0) }
    fn add_validator_to_allowlist() -> Weight { Weight::from_parts(10_000, 0) }
    fn remove_validator_from_allowlist() -> Weight { Weight::from_parts(10_000, 0) }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxEpochHistory = MaxEpochHistory;
    type DefaultPosWeight = DefaultPosWeight;
    type DefaultPoiWeight = DefaultPoiWeight;
    type BlockAuthorshipBoost = BlockAuthorshipBoost;
    type MissedBlockPenalty = MissedBlockPenalty;
    type Balance = u128;
    type WeightInfo = MockWeightInfo;

    // Constants for hardcoded values
    type MaxValidatorHistorySize = ConstU32<10>;
    type MaxValidatorNameSize = ConstU32<32>;
    type MaxRuntimeApiBoundedVecSize = ConstU32<100>;
    type MaxProposalActionBoundedVecSize = ConstU32<100>;
    type AuthorNotActiveErrorCode = ConstU8<1>;
    type AuthorMismatchErrorCode = ConstU8<2>;
    
    // Additional trust score configuration
    type MinTrustScore = ConstU64<1000>;
    type MaxTrustScoreGrowthRate = ConstU32<500>;
    type MaxTrustScoreDecayRate = ConstU32<200>;
    type TrustScoreStabilityFactor = ConstU32<8000>;
    type MaxInactiveEpochs = ConstU32<5>;
    type ScoreDecayInterval = ConstU32<10>;
    type ParticipationUpdateInterval = ConstU32<100>;
    type UnderperformanceCheckInterval = ConstU32<50>;
    type ValidatorProposalInterval = ConstU32<200>;
    type HealthMetricsInterval = ConstU32<1000>;
    type EpochLength = ConstU32<2400>;

    // Trust score weights
    type TrustScoreUptimeWeight = ConstU64<4000>;
    type TrustScoreInferenceWeight = ConstU64<4000>;
    type TrustScoreSlashingWeight = ConstU64<2000>;
    type MaxTrustScore = ConstU64<10000>;

    // Misbehavior reporting
    type MaxEvidenceLength = ConstU32<1000>;
    type MisbehaviorSlashThreshold = ConstU32<3>;
    
    // Off-chain worker configuration
    type OffchainWorkerTimeout = ConstU64<5000>;
    type EstimatedBlockTime = ConstU64<6000>;
    
    // Performance thresholds
    type MinPerformanceScore = ConstU64<30>;
    type MinParticipationRate = ConstU32<50>;
    type HighParticipationRate = ConstU32<90>;
    type MaxMissedBlocks = ConstU32<10>;
    type MaxMissedBlocksHigh = ConstU32<2>;
    type HealthyValidatorScore = ConstU64<50>;
    type HealthyParticipationRate = ConstU32<80>;
    type HealthyMissedBlocksMax = ConstU32<5>;
    
    // Score calculation thresholds
    type ScoreChangeThreshold = ConstU64<1000>;
    type ScoreChangePercentage = ConstU32<10>;
    type ScoreImprovementThreshold = ConstU64<1000>;
    type ScoreImprovementPercentage = ConstU32<10>;
    
    // Contribution balance thresholds
    type MaxPosContribution = ConstU32<90>;
    type MaxPoiContribution = ConstU32<90>;
    type ImbalanceWarningThreshold = ConstU32<85>;
    
    // Block processing intervals
    type LeaveRequestCheckInterval = ConstU32<10>;
    type MetricsUpdateInterval = ConstU32<10>;
    type ScoreRefreshInterval = ConstU32<50>;
    type DetailedLoggingInterval = ConstU32<100>;
    type ImbalanceCheckInterval = ConstU32<500>;
    
    // Validator set limits
    type TopValidatorsDisplayCount = ConstU32<5>;
    type HealthCheckSampleSize = ConstU32<5>;
    

    // Reward distribution percentages
    type BaseRewardPercentage = ConstU32<60>;
    type PerformanceRewardPercentage = ConstU32<25>;
    type TopPerformerRewardPercentage = ConstU32<15>;

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

    // DVF integration — no-op in tests
    type WeightFreezer = ();
    type DvfFinalizedBlockProvider = ();

    type ValidatorRegistry = MockValidatorRegistry;
}

use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static MOCK_ACTIVE_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    static MOCK_VALIDATOR_SET: RefCell<Vec<u64>> = RefCell::new(Vec::new());
    static MOCK_VALIDATOR_STATES: RefCell<BTreeMap<u64, crate::traits::ValidatorState>> = RefCell::new(BTreeMap::new());
    static MOCK_VALIDATOR_NAMES: RefCell<BTreeMap<u64, Vec<u8>>> = RefCell::new(BTreeMap::new());
    static MOCK_VALIDATOR_METADATA: RefCell<BTreeMap<u64, crate::ValidatorMetadataInfo>> = RefCell::new(BTreeMap::new());
    static MOCK_PENDING_ACTIONS: RefCell<BTreeMap<u64, crate::ValidatorAction>> = RefCell::new(BTreeMap::new());
    static MOCK_JOIN_TIMES: RefCell<BTreeMap<u64, u32>> = RefCell::new(BTreeMap::new());
    static MOCK_LEAVE_REQUESTS: RefCell<BTreeMap<u64, u32>> = RefCell::new(BTreeMap::new());
    static MOCK_RECENTLY_REMOVED: RefCell<BTreeMap<u64, u32>> = RefCell::new(BTreeMap::new());
    static MOCK_PERF_HISTORIES: RefCell<BTreeMap<u64, Vec<crate::PerformanceRecord>>> = RefCell::new(BTreeMap::new());
}

pub fn clear_mock_registry() {
    MOCK_ACTIVE_VALIDATORS.with(|v| v.borrow_mut().clear());
    MOCK_VALIDATOR_SET.with(|v| v.borrow_mut().clear());
    MOCK_VALIDATOR_STATES.with(|v| v.borrow_mut().clear());
    MOCK_VALIDATOR_NAMES.with(|v| v.borrow_mut().clear());
    MOCK_VALIDATOR_METADATA.with(|v| v.borrow_mut().clear());
    MOCK_PENDING_ACTIONS.with(|v| v.borrow_mut().clear());
    MOCK_JOIN_TIMES.with(|v| v.borrow_mut().clear());
    MOCK_LEAVE_REQUESTS.with(|v| v.borrow_mut().clear());
    MOCK_RECENTLY_REMOVED.with(|v| v.borrow_mut().clear());
    MOCK_PERF_HISTORIES.with(|v| v.borrow_mut().clear());
}

pub struct MockValidatorRegistry;

impl crate::traits::ValidatorRegistryProvider<u64, u128, u64> for MockValidatorRegistry {
    fn get_active_validators() -> Vec<u64> {
        MOCK_ACTIVE_VALIDATORS.with(|v| v.borrow().clone())
    }
    fn update_active_validators(active: Vec<u64>) {
        MOCK_ACTIVE_VALIDATORS.with(|v| *v.borrow_mut() = active);
    }
    fn get_validator_set() -> Vec<u64> {
        MOCK_VALIDATOR_SET.with(|v| v.borrow().clone())
    }
    fn update_validator_set(set: Vec<u64>) {
        MOCK_VALIDATOR_SET.with(|v| *v.borrow_mut() = set);
    }
    fn is_validator_active(validator: &u64) -> bool {
        MOCK_ACTIVE_VALIDATORS.with(|v| v.borrow().contains(validator))
    }
    fn get_validator_profile(_validator: &u64) -> Option<crate::ValidatorProfile<u64, u128, u64>> {
        None
    }
    fn get_validator_status(_validator: &u64) -> Option<crate::ValidatorStatus> {
        None
    }
    fn eject_validator(_validator: &u64, _reason: crate::EjectionReason) -> sp_runtime::DispatchResult {
        Ok(())
    }
    fn get_validator_state(validator: &u64) -> Option<crate::traits::ValidatorState> {
        MOCK_VALIDATOR_STATES.with(|m| m.borrow().get(validator).cloned())
    }
    fn update_validator_state(validator: &u64, state: crate::traits::ValidatorState) {
        MOCK_VALIDATOR_STATES.with(|m| m.borrow_mut().insert(*validator, state));
    }
    fn contains_validator_state(validator: &u64) -> bool {
        MOCK_VALIDATOR_STATES.with(|m| m.borrow().contains_key(validator))
    }
    fn remove_validator_state(validator: &u64) {
        MOCK_VALIDATOR_STATES.with(|m| m.borrow_mut().remove(validator));
    }
    fn get_validator_name(validator: &u64) -> Option<Vec<u8>> {
        MOCK_VALIDATOR_NAMES.with(|m| m.borrow().get(validator).cloned())
    }
    fn set_validator_name(validator: &u64, name: Vec<u8>) {
        MOCK_VALIDATOR_NAMES.with(|m| m.borrow_mut().insert(*validator, name));
    }
    fn get_validator_metadata(validator: &u64) -> Option<crate::ValidatorMetadataInfo> {
        MOCK_VALIDATOR_METADATA.with(|m| m.borrow().get(validator).cloned())
    }
    fn set_validator_metadata(validator: &u64, metadata: crate::ValidatorMetadataInfo) {
        MOCK_VALIDATOR_METADATA.with(|m| m.borrow_mut().insert(*validator, metadata));
    }
    fn remove_validator_metadata(validator: &u64) {
        MOCK_VALIDATOR_METADATA.with(|m| m.borrow_mut().remove(validator));
    }
    fn get_validator_detailed_cooldown_status(_validator: &u64) -> Option<(u32, bool)> {
        None
    }
    fn get_validator_leave_request(validator: &u64) -> Option<u32> {
        MOCK_LEAVE_REQUESTS.with(|m| m.borrow().get(validator).cloned())
    }
    fn set_validator_leave_request(validator: &u64, val: u32) {
        MOCK_LEAVE_REQUESTS.with(|m| m.borrow_mut().insert(*validator, val));
    }
    fn remove_validator_leave_request(validator: &u64) {
        MOCK_LEAVE_REQUESTS.with(|m| m.borrow_mut().remove(validator));
    }
    fn get_pending_actions() -> Vec<(u64, crate::ValidatorAction)> {
        MOCK_PENDING_ACTIONS.with(|m| m.borrow().iter().map(|(k, v)| (*k, v.clone())).collect())
    }
    fn remove_pending_action(validator: &u64) {
        MOCK_PENDING_ACTIONS.with(|m| m.borrow_mut().remove(validator));
    }
    fn add_pending_action(validator: &u64, action: crate::ValidatorAction) {
        MOCK_PENDING_ACTIONS.with(|m| m.borrow_mut().insert(*validator, action));
    }
    fn get_validator_join_time(validator: &u64) -> Option<u32> {
        MOCK_JOIN_TIMES.with(|m| m.borrow().get(validator).cloned())
    }
    fn set_validator_join_time(validator: &u64, val: u32) {
        MOCK_JOIN_TIMES.with(|m| m.borrow_mut().insert(*validator, val));
    }
    fn remove_validator_join_time(validator: &u64) {
        MOCK_JOIN_TIMES.with(|m| m.borrow_mut().remove(validator));
    }
    fn get_recently_removed_validator(validator: &u64) -> Option<u32> {
        MOCK_RECENTLY_REMOVED.with(|m| m.borrow().get(validator).cloned())
    }
    fn set_recently_removed_validator(validator: &u64, val: u32) {
        MOCK_RECENTLY_REMOVED.with(|m| m.borrow_mut().insert(*validator, val));
    }
    fn remove_recently_removed_validator(validator: &u64) {
        MOCK_RECENTLY_REMOVED.with(|m| m.borrow_mut().remove(validator));
    }
    fn get_validator_performance_history(validator: &u64) -> Vec<crate::PerformanceRecord> {
        MOCK_PERF_HISTORIES.with(|m| m.borrow().get(validator).cloned().unwrap_or_default())
    }
    fn set_validator_performance_history(validator: &u64, history: Vec<crate::PerformanceRecord>) {
        MOCK_PERF_HISTORIES.with(|m| m.borrow_mut().insert(*validator, history));
    }
    fn remove_validator_performance_history(validator: &u64) {
        MOCK_PERF_HISTORIES.with(|m| m.borrow_mut().remove(validator));
    }
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



// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    clear_mock_registry();
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 10000),
            (2, 10000),
            (3, 10000),
            (4, 10000),
            (10, 100000),
            (11, 100000),
            (12, 100000),
            (13, 100000),
            (14, 100000),
            (15, 100000),
            (16, 100000),
            (17, 100000),
            (18, 100000),
            (19, 100000),
            (20, 100000),
            (21, 100000),
        ],
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    pallet_cbc_pos::GenesisConfig::<Test> {
        validators: vec![1, 2, 3],
        validator_scores: vec![6000, 7000, 8000],
        current_epoch: 0,
        slashing_count: vec![],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    pallet_cbc_poi::GenesisConfig::<Test> {
        inference_results: vec![(1, 6000), (2, 7000), (3, 8000)],
        challenges: vec![],
        current_epoch: 0,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    crate::GenesisConfig::<Test> {
        validators: vec![1, 2, 3],
        validator_scores: vec![6000, 7000, 8000],
        validator_stakes: vec![1000, 1000, 1000],
        validator_names: vec![],
        current_epoch: 0,
        epoch_config: EpochConfig {
            blocks_per_epoch: 10,
            min_stake: 1000,
            max_validators: 100,
        },
        strict_validation: false,
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    let mut ext: sp_io::TestExternalities = storage.into();
    let (offchain, _offchain_state) = sp_core::offchain::testing::TestOffchainExt::new();
    ext.register_extension(sp_core::offchain::OffchainDbExt::new(offchain.clone()));
    ext.register_extension(sp_core::offchain::OffchainWorkerExt::new(offchain));
    ext
}

pub fn new_test_ext_with_genesis(genesis_config: crate::GenesisConfig<Test>) -> sp_io::TestExternalities {
    clear_mock_registry();
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // Initialize balances for all potential test accounts (including genesis validators)
    let mut balances = vec![
        (1, 10000), (2, 10000), (3, 10000), (4, 10000), (5, 10000),
    ];

    // Add balances for genesis validators
    for (i, validator) in genesis_config.validators.iter().enumerate() {
        let balance = if i < genesis_config.validator_stakes.len() {
            genesis_config.validator_stakes[i] * 2
        } else {
            2000 // Default balance
        };
        balances.push((*validator, balance));
    }

    pallet_balances::GenesisConfig::<Test> {
        balances,
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let pos_scores: Vec<u32> = genesis_config.validator_scores.iter().map(|s| *s as u32).collect();
    pallet_cbc_pos::GenesisConfig::<Test> {
        validators: genesis_config.validators.clone(),
        validator_scores: pos_scores.clone(),
        current_epoch: 0,
        slashing_count: vec![],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let poi_results: Vec<(u64, u32)> = genesis_config.validators.iter().zip(pos_scores.iter()).map(|(v, s)| (*v, *s)).collect();
    pallet_cbc_poi::GenesisConfig::<Test> {
        inference_results: poi_results,
        challenges: vec![],
        current_epoch: 0,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    // Use the provided genesis config
    genesis_config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();
    let (offchain, _offchain_state) = sp_core::offchain::testing::TestOffchainExt::new();
    ext.register_extension(sp_core::offchain::OffchainDbExt::new(offchain.clone()));
    ext.register_extension(sp_core::offchain::OffchainWorkerExt::new(offchain));
    ext
}