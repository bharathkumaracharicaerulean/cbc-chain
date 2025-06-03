#![allow(unused_imports)]
#![allow(dead_code)]

use super::*;
use crate::pallet::*;
use frame_support::{
    assert_ok, assert_noop,
    parameter_types,
    traits::{ConstU32, OnFinalize, OnInitialize, PalletInfo as PalletInfoTrait},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use sp_std::convert::TryInto;

// Define the test runtime
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TestRuntime;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!( 
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Dcf: crate,
        Pos: pallet_cbc_pos,
        Poi: pallet_cbc_poi,
    }
);

// Define the Block type
type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u64, RuntimeCall, (), ()>;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
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
    pub const MaxValidators: u32 = 10;
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

// Mock POS pallet implementation
impl pallet_cbc_pos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinValidatorScore = MinValidatorScore;
    type MinActiveValidators = MinActiveValidators;
    type MaxValidators = MaxValidators;
    type ValidatorScoreDecay = ValidatorScoreDecay;
    type MaxSlashingCount = MaxSlashingCount;
}

// Mock POI pallet implementation
impl pallet_cbc_poi::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MinInferenceConfidence = ConstU32<50>;
    type MaxInferenceAge = ConstU32<100>;
    type ChallengeWindow = ConstU32<10>;
    type InferenceReward = InferenceReward;
    type ChallengeReward = ChallengeReward;
}

// Helper function to create a new test environment
fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(1);
    });
    ext
}

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        Error, EpochConfig, EpochConfigStorage, CurrentEpoch,
        ValidatorStakeScores, ValidatorInferenceScores, PosWeight, PoiWeight,
        ValidatorFinalScores, GenesisConfig, ValidatorSet, ActiveValidators
    };
    use frame_support::{
        assert_ok, assert_noop,
        traits::{OnInitialize, OnFinalize},
        BoundedVec,
    };
    use sp_runtime::BuildStorage;

    #[test]
    fn test_update_validator_stake_score() {
        new_test_ext().execute_with(|| {
            // Set up initial validator score in POS pallet
            let validator = 1;
            let score = 100;
            pallet_cbc_pos::ValidatorScores::<Test>::insert(validator, score);

            // Update stake score
            assert_ok!(Dcf::update_validator_stake_score(
                RuntimeOrigin::signed(validator),
                validator
            ));

            // Check if score was updated correctly
            assert_eq!(Dcf::validator_stake_scores(validator), score as u64);
        });
    }

    #[test]
    fn test_update_validator_inference_score() {
        new_test_ext().execute_with(|| {
            let validator = 1;
            let score = 80;
            
            // Set up initial inference result in POI pallet with current block number
            let current_block = frame_system::Pallet::<Test>::block_number() as u32;
            pallet_cbc_poi::InferenceResults::<Test>::insert(validator, (score, current_block));

            // Update inference score
            assert_ok!(Dcf::update_validator_inference_score(
                RuntimeOrigin::signed(validator),
                validator
            ));

            // Check if score was updated correctly
            assert_eq!(Dcf::validator_inference_scores(validator), score as u64);
        });
    }

    #[test]
    fn test_update_validator_inference_score_not_found() {
        new_test_ext().execute_with(|| {
            let validator = 1;
            
            // Don't set up any inference result, so calculate_validator_score will return None

            // Should fail with ValidatorNotFound error
            assert_noop!(
                Dcf::update_validator_inference_score(
                    RuntimeOrigin::signed(validator),
                    validator
                ),
                Error::<Test>::ValidatorNotFound
            );
        });
    }

    #[test]
    fn test_update_consensus_weights() {
        new_test_ext().execute_with(|| {
            let pos_weight = 70;
            let poi_weight = 30;

            // Update weights
            assert_ok!(Dcf::update_consensus_weights(
                RuntimeOrigin::root(),
                pos_weight,
                poi_weight
            ));

            // Check if weights were updated correctly
            assert_eq!(Dcf::pos_weight(), pos_weight);
            assert_eq!(Dcf::poi_weight(), poi_weight);
        });
    }

    #[test]
    fn test_invalid_consensus_weights() {
        new_test_ext().execute_with(|| {
            let pos_weight = 70;
            let poi_weight = 40; // Sum > 100

            // Should fail with InvalidWeight error
            assert_noop!(
                Dcf::update_consensus_weights(
                    RuntimeOrigin::root(),
                    pos_weight,
                    poi_weight
                ),
                Error::<Test>::InvalidWeight
            );
        });
    }

    #[test]
    fn test_epoch_transition() {
        new_test_ext().execute_with(|| {
            // Set up epoch config
            let epoch_config = EpochConfig {
                blocks_per_epoch: 10,
                min_stake: 1000,
                max_validators: 5,
            };
            EpochConfigStorage::<Test>::put(epoch_config);

            // Set up initial validators
            let validators = vec![1, 2, 3, 4, 5];
            let scores = vec![100, 90, 80, 70, 60];
            
            // Initialize validator set
            let bounded_validators: BoundedVec<_, MaxValidators> = validators.clone().try_into().unwrap();
            ValidatorSet::<Test>::put(bounded_validators.clone());
            ActiveValidators::<Test>::put(bounded_validators);
            
            for (validator, score) in validators.iter().zip(scores.iter()) {
                pallet_cbc_pos::ValidatorScores::<Test>::insert(validator, *score);
                pallet_cbc_poi::InferenceResults::<Test>::insert(validator, (*score, 0));
            }

            // Set initial epoch
            CurrentEpoch::<Test>::put(0);

            // Advance blocks to trigger epoch transition
            for i in 1..=10 {
                frame_system::Pallet::<Test>::set_block_number(i);
                // Call POI's on_initialize first to ensure proper epoch synchronization
                <pallet_cbc_poi::Pallet<Test> as OnInitialize<u64>>::on_initialize(i);
                <Dcf as OnInitialize<u64>>::on_initialize(i);
                <Dcf as OnFinalize<u64>>::on_finalize(i);
            }

            // Check if epoch was updated
            assert_eq!(Dcf::current_epoch(), 1);
            // Verify POI's epoch is also updated
            assert_eq!(pallet_cbc_poi::Pallet::<Test>::current_epoch(), 1);
        });
    }

    #[test]
    fn test_final_score_calculation() {
        new_test_ext().execute_with(|| {
            let validator = 1;
            let stake_score: u64 = 100;
            let inference_score: u64 = 80;
            let pos_weight = 60;
            let poi_weight = 40;

            // Set up scores in both POS and POI pallets
            pallet_cbc_pos::ValidatorScores::<Test>::insert(validator, stake_score as u32);
            pallet_cbc_poi::InferenceResults::<Test>::insert(validator, (inference_score as u32, 0));
            
            // Set up weights
            PosWeight::<Test>::put(pos_weight);
            PoiWeight::<Test>::put(poi_weight);

            // First update stake score
            assert_ok!(Dcf::update_validator_stake_score(
                RuntimeOrigin::signed(validator),
                validator
            ));

            // Then update inference score
            assert_ok!(Dcf::update_validator_inference_score(
                RuntimeOrigin::signed(validator),
                validator
            ));

            // Expected final score: (100 * 60 + 80 * 40) / 100 = 92
            let expected_final_score = 92;
            let validator_score = ValidatorFinalScores::<Test>::get(validator);
            assert_eq!(validator_score.final_score, expected_final_score);
            
            // Verify individual scores
            assert_eq!(validator_score.stake_weight, stake_score);
            assert_eq!(validator_score.inference_weight, inference_score);
        });
    }

    #[test]
    fn test_genesis_config() {
        let validators = vec![1, 2, 3];
        let scores = vec![100, 90, 80];
        let current_epoch = 0;
        let epoch_config = EpochConfig {
            blocks_per_epoch: 10,
            min_stake: 1000,
            max_validators: 5,
        };

        let genesis_config = GenesisConfig::<Test> {
            validators: validators.clone(),
            validator_scores: scores.clone(),
            current_epoch,
            epoch_config: epoch_config.clone(),
        };

        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        
        genesis_config.assimilate_storage(&mut t).unwrap();
        
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| {
            // Check if validators were set correctly
            let bounded_validators: BoundedVec<_, MaxValidators> = validators.clone().try_into().unwrap();
            assert_eq!(Dcf::validator_set(), bounded_validators);

            // Check if scores were set correctly
            for (validator, score) in validators.iter().zip(scores.iter()) {
                assert_eq!(Dcf::validator_stake_scores(*validator), *score as u64);
                assert_eq!(Dcf::validator_inference_scores(*validator), *score as u64);
            }

            // Check if epoch config was set correctly
            assert_eq!(Dcf::epoch_config(), epoch_config);

            // Check if current epoch was set correctly
            assert_eq!(Dcf::current_epoch(), current_epoch);

            // Check if default weights were set correctly
            assert_eq!(Dcf::pos_weight(), DefaultPosWeight::get());
            assert_eq!(Dcf::poi_weight(), DefaultPoiWeight::get());
        });
    }
}