use crate as pallet_cbc_governance;
use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
    weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, DispatchError, DispatchResult,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        PalletCbcGovernance: pallet_cbc_governance,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
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
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
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
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<500>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

pub struct MockProposalExecutor;
impl pallet_cbc_governance::ProposalExecutor<u64, u64> for MockProposalExecutor {
    fn slash_validator(_validator: &u64, _amount: u64) -> DispatchResult {
        Ok(())
    }
    fn reward_validator(_validator: &u64, _amount: u64) -> DispatchResult {
        Ok(())
    }
    fn eject_validator(
        _validator: &u64,
        _reason: pallet_cbc_governance::EjectionReason,
    ) -> DispatchResult {
        Ok(())
    }
    fn add_validator(_validator: &u64) -> DispatchResult {
        Ok(())
    }
    fn remove_validator(_validator: &u64) -> DispatchResult {
        Ok(())
    }
}

// Designated test fixture accounts for scenario testing
pub const FORBIDDEN_PROPOSER_ACCOUNT: u64 = 999;
pub const FORBIDDEN_TARGET_ACCOUNT: u64 = 999;
pub const RATE_LIMITED_ACCOUNT: u64 = 888;

pub struct MockValidatorProvider;
impl pallet_cbc_governance::ValidatorProvider<u64, Weight> for MockValidatorProvider {
    fn active_validators() -> Vec<u64> {
        vec![1, 2, 3]
    }
    fn is_private_chain_mode() -> bool {
        false
    }
    fn validate_governance_in_private_mode(proposer: &u64) -> DispatchResult {
        if *proposer == FORBIDDEN_PROPOSER_ACCOUNT {
            return Err(DispatchError::Other("PrivateModeGovernanceForbidden"));
        }
        Ok(())
    }
    fn validate_proposal_in_private_mode(_proposer: &u64, target: &u64) -> DispatchResult {
        if *target == FORBIDDEN_TARGET_ACCOUNT {
            return Err(DispatchError::Other("PrivateModeTargetForbidden"));
        }
        Ok(())
    }
    fn check_rate_limits(
        proposer: &u64,
        op_type: u8,
        _weight: Weight,
    ) -> Result<(), (DispatchError, Option<(u8, u8, u32, u32)>)> {
        if *proposer == RATE_LIMITED_ACCOUNT {
            return Err((
                crate::Error::<Test>::RateLimitExceeded.into(),
                Some((op_type, 1u8, 10u32, 5u32)),
            ));
        }
        Ok(())
    }
    fn record_operation(_proposer: &u64, _op_type: u8) {}
}

impl pallet_cbc_governance::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u64;
    type ProposalExecutor = MockProposalExecutor;
    type ValidatorProvider = MockValidatorProvider;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10000), (2, 10000), (3, 10000)],
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}
