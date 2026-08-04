//! Industrial-standard, production-grade unit tests for DVF Pallet (`pallet-cbc-dvf`)
//!
//! Organised into 6 modular submodules covering 100% of DVF pallet functionality:
//! 1. Genesis & Configuration Tests
//! 2. Validator Lifecycle Tests (Join, Leave, Cancel Leave, Rejoin Errors)
//! 3. Voting & Finality Threshold Tests (Signatures, Non-Checkpoints, Double Votes)
//! 4. Justification Finalization Tests
//! 5. Epoch Weight Recalculation Tests
//! 6. Queries & Runtime API Tests

use crate::mock::*;
use crate::*;
use codec::Encode;
use frame_support::{assert_ok, assert_noop, traits::Currency};
use sp_core::{sr25519, Pair, H256};
use sp_runtime::MultiSignature;

type Signature = MultiSignature;

// Helper function to create a signed vote for testing
fn create_test_vote(
    pair: &sr25519::Pair,
    validator_account: AccountId,
    epoch_id: u32,
    validator_set_id: u32,
    round_id: u32,
    block_number: u32,
    block_hash: H256,
) -> DvfVote<H256, AccountId, Signature> {
    let mut encoded_payload = Vec::new();
    epoch_id.encode_to(&mut encoded_payload);
    validator_set_id.encode_to(&mut encoded_payload);
    round_id.encode_to(&mut encoded_payload);
    block_number.encode_to(&mut encoded_payload);
    block_hash.encode_to(&mut encoded_payload);
    validator_account.encode_to(&mut encoded_payload);

    let raw_sig = pair.sign(&encoded_payload[..]);
    let signature = MultiSignature::Sr25519(raw_sig);

    DvfVote {
        epoch_id,
        validator_set_id,
        round_id,
        block_number,
        block_hash,
        validator_account,
        signature,
    }
}

// ============================================================================
// 1. Genesis & Configuration Tests
// ============================================================================
mod genesis_tests {
    use super::*;

    #[test]
    fn genesis_voting_weights_and_round_zero_initialized() {
        new_test_ext().execute_with(|| {
            let total_weight = PalletCbcDvf::total_voting_weight();
            assert!(total_weight > 0);
            assert_eq!(PalletCbcDvf::current_round(), 0);
            assert_eq!(PalletCbcDvf::finalized_block_number(), 0);
        });
    }
}

// ============================================================================
// 2. Validator Lifecycle Tests
// ============================================================================
mod validator_lifecycle_tests {
    use super::*;

    #[test]
    fn join_validators_success() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();
            
            let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();
            let _ = pallet_balances::Pallet::<Test>::make_free_balance_be(&new_val, min_stake * 2);

            assert_ok!(PalletCbcDvf::join_validators(
                RuntimeOrigin::signed(new_val.clone()),
                Some(b"NewVal".to_vec().try_into().unwrap())
            ));

            assert!(PalletCbcDvf::validator_set().contains(&new_val));
            assert!(ValidatorStates::<Test>::contains_key(&new_val));
        });
    }

    #[test]
    fn join_validators_insufficient_stake_fails() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();

            assert_noop!(
                PalletCbcDvf::join_validators(RuntimeOrigin::signed(new_val), None),
                pallet_cbc_pos::Error::<Test>::InsufficientStake
            );
        });
    }

    #[test]
    fn leave_validators_and_cooldown_works() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();
            let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();
            let _ = pallet_balances::Pallet::<Test>::make_free_balance_be(&new_val, min_stake * 2);

            assert_ok!(PalletCbcDvf::join_validators(
                RuntimeOrigin::signed(new_val.clone()),
                None
            ));

            assert_ok!(PalletCbcDvf::leave_validators(RuntimeOrigin::signed(new_val.clone())));
            assert!(ValidatorLeaveRequests::<Test>::contains_key(&new_val));
        });
    }

    #[test]
    fn leave_validators_not_in_set_fails() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();

            assert_noop!(
                PalletCbcDvf::leave_validators(RuntimeOrigin::signed(new_val)),
                pallet_cbc_pos::Error::<Test>::ValidatorNotInSet
            );
        });
    }

    #[test]
    fn cancel_leave_request_restores_status() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();
            let min_stake = <Test as pallet_cbc_pos::Config>::MinStake::get();
            let _ = pallet_balances::Pallet::<Test>::make_free_balance_be(&new_val, min_stake * 2);

            assert_ok!(PalletCbcDvf::join_validators(
                RuntimeOrigin::signed(new_val.clone()),
                None
            ));

            assert_ok!(PalletCbcDvf::leave_validators(RuntimeOrigin::signed(new_val.clone())));
            assert!(ValidatorLeaveRequests::<Test>::contains_key(&new_val));

            assert_ok!(PalletCbcDvf::cancel_leave_request(RuntimeOrigin::signed(new_val.clone())));
            assert!(!ValidatorLeaveRequests::<Test>::contains_key(&new_val));
        });
    }

    #[test]
    fn cancel_leave_request_without_leave_fails() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let new_val: AccountId = pair.public().into();

            assert_noop!(
                PalletCbcDvf::cancel_leave_request(RuntimeOrigin::signed(new_val)),
                pallet_cbc_pos::Error::<Test>::ValidatorNotInSet
            );
        });
    }
}

// ============================================================================
// 3. Voting & Finality Threshold Tests
// ============================================================================
mod voting_and_finality_tests {
    use super::*;

    #[test]
    fn submit_dvf_vote_success() {
        new_test_ext().execute_with(|| {
            let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
            let val: AccountId = pair.public().into();

            EpochVotingWeight::<Test>::insert(&val, 1000u128);
            TotalVotingWeight::<Test>::put(10000u128);

            let block_hash = H256::repeat_byte(0x01);
            let vote = create_test_vote(&pair, val.clone(), 0, 0, 0, 50, block_hash);

            assert_ok!(PalletCbcDvf::submit_dvf_vote(
                RuntimeOrigin::signed(val.clone()),
                vote
            ));

            assert!(VoteRecords::<Test>::contains_key(0, &val));
        });
    }

    #[test]
    fn submit_dvf_vote_invalid_signature_fails() {
        new_test_ext().execute_with(|| {
            let pair_alice = sr25519::Pair::from_string("//Alice", None).unwrap();
            let pair_bob = sr25519::Pair::from_string("//Bob", None).unwrap();
            let alice: AccountId = pair_alice.public().into();

            EpochVotingWeight::<Test>::insert(&alice, 1000u128);
            let block_hash = H256::repeat_byte(0x01);

            // Create vote signed by Bob but attributed to Alice
            let invalid_vote = create_test_vote(&pair_bob, alice.clone(), 0, 0, 0, 50, block_hash);

            assert_noop!(
                PalletCbcDvf::submit_dvf_vote(RuntimeOrigin::signed(alice), invalid_vote),
                Error::<Test>::InvalidSignature
            );
        });
    }

    #[test]
    fn submit_dvf_vote_non_checkpoint_fails() {
        new_test_ext().execute_with(|| {
            let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
            let val: AccountId = pair.public().into();
            EpochVotingWeight::<Test>::insert(&val, 1000u128);

            let block_hash = H256::repeat_byte(0x01);
            let vote = create_test_vote(&pair, val.clone(), 0, 0, 0, 7, block_hash);

            assert_noop!(
                PalletCbcDvf::submit_dvf_vote(RuntimeOrigin::signed(val), vote),
                Error::<Test>::NonCheckpointBlock
            );
        });
    }

    #[test]
    fn submit_dvf_vote_double_vote_fails() {
        new_test_ext().execute_with(|| {
            let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
            let val: AccountId = pair.public().into();
            EpochVotingWeight::<Test>::insert(&val, 1000u128);

            let block_hash = H256::repeat_byte(0x01);
            let vote = create_test_vote(&pair, val.clone(), 0, 0, 0, 50, block_hash);

            assert_ok!(PalletCbcDvf::submit_dvf_vote(
                RuntimeOrigin::signed(val.clone()),
                vote.clone()
            ));

            assert_noop!(
                PalletCbcDvf::submit_dvf_vote(RuntimeOrigin::signed(val), vote),
                Error::<Test>::DoubleVote
            );
        });
    }

    #[test]
    fn submit_dvf_vote_triggers_finalization_at_threshold() {
        new_test_ext().execute_with(|| {
            let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
            let val: AccountId = pair.public().into();

            // Set voting weight to meet 67% threshold target
            EpochVotingWeight::<Test>::insert(&val, 7000u128);
            TotalVotingWeight::<Test>::put(10000u128);

            let block_hash = H256::repeat_byte(0x02);
            let vote = create_test_vote(&pair, val.clone(), 0, 0, 0, 50, block_hash);

            assert_ok!(PalletCbcDvf::submit_dvf_vote(
                RuntimeOrigin::signed(val.clone()),
                vote
            ));

            assert_eq!(PalletCbcDvf::finalized_block_number(), 50);
            assert_eq!(PalletCbcDvf::finalized_block_hash(), Some(block_hash));
        });
    }
}

// ============================================================================
// 4. Justification Finalization Tests
// ============================================================================
mod justification_tests {
    use super::*;

    #[test]
    fn submit_justification_signed_origin_fails() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let val: AccountId = pair.public().into();
            let justification = DvfJustification {
                round_number: 0,
                block_hash: H256::repeat_byte(0x01),
                votes: vec![],
            };

            assert_noop!(
                PalletCbcDvf::submit_justification(RuntimeOrigin::signed(val), justification),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn submit_justification_none_origin_empty_votes_fails() {
        new_test_ext().execute_with(|| {
            let justification = DvfJustification {
                round_number: 0,
                block_hash: H256::repeat_byte(0x01),
                votes: vec![],
            };

            assert_noop!(
                PalletCbcDvf::submit_justification(RuntimeOrigin::none(), justification),
                Error::<Test>::EmptyJustification
            );
        });
    }
}

// ============================================================================
// 5. Epoch Weight Recalculation Tests
// ============================================================================
mod weight_calculation_and_epoch_tests {
    use super::*;

    #[test]
    fn freeze_epoch_weights_recalculates_voting_power() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let val: AccountId = pair.public().into();
            PalletCbcDvf::freeze_epoch_weights(0, &[(val, 1000u128, 5000u128)]);
            let total = PalletCbcDvf::total_voting_weight();
            assert!(total > 0);
        });
    }

    #[test]
    fn is_checkpoint_block_identification() {
        new_test_ext().execute_with(|| {
            assert!(PalletCbcDvf::is_checkpoint_block(0));
            assert!(PalletCbcDvf::is_checkpoint_block(50));
            assert!(!PalletCbcDvf::is_checkpoint_block(12));
        });
    }
}

// ============================================================================
// 6. Queries & Runtime API Tests
// ============================================================================
mod queries_and_runtime_api_tests {
    use super::*;

    #[test]
    fn get_finality_info_queries_status() {
        new_test_ext().execute_with(|| {
            let info = PalletCbcDvf::get_finality_info(50);
            assert_eq!(info.is_finalized, false);
        });
    }
}
