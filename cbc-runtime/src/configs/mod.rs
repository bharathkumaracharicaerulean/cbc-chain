// This file is part of the CBC Runtime.
// It defines configuration settings for the runtime, including consensus parameters,
// block weights, and other system-wide constants.

// === Imports ===
// Core Substrate and FRAME support crates
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8, VariantCountOf},
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        IdentityFee, Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local runtime modules and type aliases
use super::{
    AccountId, Balance, Balances, Block, BlockNumber, Hash, Nonce, PalletInfo, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask,
    System, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
};

use crate::{
    MinValidatorScore, MinActiveValidators, ValidatorScoreDecay, MaxSlashingCount,
    MinInferenceConfidence, MaxInferenceAge, ChallengeWindow, InferenceReward, ChallengeReward, MaxValidatorScore,
    BlockAuthorshipBoost, MissedBlockPenalty,
    InferenceBoostLow, InferenceBoostMedium, InferenceBoostHigh,
    InferencePenaltyLow, InferencePenaltyMedium, InferencePenaltyHigh,

};

// === Constants ===
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

// === Global Runtime Parameters ===
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;

    // Max compute time per block: 2s of compute for a 6s block.
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
        Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
        NORMAL_DISPATCH_RATIO,
    );

    // Max block size: 5MB with 75% for normal transactions.
    pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);

    // Substrate address prefix.
    pub const SS58Prefix: u8 = 42;

    pub const MaxValidators: u32 = 100;
    pub const MinStake: u128 = 1000;
    pub const MaxStake: u128 = 1000000;
    pub const EpochDuration: u32 = 100;
    pub const SlashingPenalty: u32 = 10;
    pub const RewardRate: u32 = 5;
    pub const MaxInferenceResults: u32 = 1000;
    pub const ChallengePeriod: u32 = 10;
    pub const MaxChallenges: u32 = 100;
    pub const PosWeight: u64 = 60;
    pub const PoiWeight: u64 = 40;
    pub const ValidatorScoreDecay: u32 = 5;  // 5% score decay per inactive epoch

    // New parameters for governance
    pub const GovernanceQuorum: u32 = 50;  // 50% of validators needed for quorum
    pub const ProposalLifetime: u32 = 1000; // Blocks until proposal expires
    pub const MinProposalDeposit: Balance = 100_000;
    pub const MaxProposalsPerValidator: u32 = 5;
    
    // Parameters for validator management
    pub const ValidatorUptimeRequirement: u32 = 90; // 90% uptime requirement
    pub const MaxMissedBlocksPerEpoch: u32 = 50;
    pub const MinParticipationRate: u32 = 75; // 75% minimum participation
    pub const InactivityEjectionBlocks: u32 = 1000;
}

// === FRAME System Configuration ===
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type BlockWeights = RuntimeBlockWeights;
    type BlockLength = RuntimeBlockLength;
    type AccountId = AccountId;
    type Nonce = Nonce;
    type Hash = Hash;
    type BlockHashCount = BlockHashCount;
    type DbWeight = RocksDbWeight;
    type Version = Version;
    type AccountData = pallet_balances::AccountData<Balance>;
    type SS58Prefix = SS58Prefix;
    type MaxConsumers = ConstU32<16>;
}

// === Timestamping Configuration ===
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

// === Balances Configuration ===
impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

// === Transaction Payment Configuration ===
parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

// === Sudo (Superuser) Configuration ===
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}


// === CBC POI Pallet Configuration ===
impl pallet_cbc_poi::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_cbc_poi::weights::SubstrateWeight<Runtime>;
    type MinInferenceConfidence = MinInferenceConfidence;
    type MaxInferenceAge = MaxInferenceAge;
    type ChallengeWindow = ChallengeWindow;
    type InferenceReward = InferenceReward;
    type ChallengeReward = ChallengeReward;
}

// === CBC POS Pallet Configuration ===
impl pallet_cbc_pos::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_cbc_pos::weights::SubstrateWeight<Runtime>;
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type MaxValidators = MaxValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxSlashingCount = MaxSlashingCount;
    type MinStake = ConstU128<1000>; // Minimum stake of 1000 units
    type Balance = Balance;
}

// === CBC DCF Pallet Configuration ===
impl pallet_cbc_dcf::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxValidators = MaxValidators;
    type DefaultPosWeight = ConstU64<60>;  // 60% weight for POS
    type DefaultPoiWeight = ConstU64<40>;  // 40% weight for POI
    type MinActiveValidators = MinActiveValidators;
    type MinValidatorScore = MinValidatorScore;
    type MaxValidatorScore = MaxValidatorScore;
    type BlockAuthorshipBoost = BlockAuthorshipBoost;
    type MissedBlockPenalty = MissedBlockPenalty;
    type InferenceBoostLow = InferenceBoostLow;
    type InferenceBoostMedium = InferenceBoostMedium;
    type InferenceBoostHigh = InferenceBoostHigh;
    type InferencePenaltyLow = InferencePenaltyLow;
    type InferencePenaltyMedium = InferencePenaltyMedium;
    type InferencePenaltyHigh = InferencePenaltyHigh;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MinStake = MinStake;
    type Balance = Balance;
    type WeightInfo = pallet_cbc_dcf::weights::SubstrateWeight<Runtime>;
    
    // New configuration items for governance features
    #[cfg(feature = "runtime-benchmarks")]
    type ProposalLifetime = ProposalLifetime;
    #[cfg(feature = "runtime-benchmarks")]
    type GovernanceQuorum = GovernanceQuorum;
    #[cfg(feature = "runtime-benchmarks")]
    type MinProposalDeposit = MinProposalDeposit;
    #[cfg(feature = "runtime-benchmarks")]
    type MaxProposalsPerValidator = MaxProposalsPerValidator;

    // New configuration items for validator management
    #[cfg(feature = "runtime-benchmarks")]
    type ValidatorUptimeRequirement = ValidatorUptimeRequirement;
    #[cfg(feature = "runtime-benchmarks")]
    type MaxMissedBlocksPerEpoch = MaxMissedBlocksPerEpoch;
    #[cfg(feature = "runtime-benchmarks")]
    type MinParticipationRate = MinParticipationRate;
    #[cfg(feature = "runtime-benchmarks")]
    type InactivityEjectionBlocks = InactivityEjectionBlocks;
}
