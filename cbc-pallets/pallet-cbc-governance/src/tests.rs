//! Production-grade unit test suite for `pallet-cbc-governance`
//!
//! Directly exercises all pallet extrinsics via `RuntimeOrigin::signed` and `RuntimeOrigin::root`.
//!
//! Submodules:
//! 1. Governance Mode Configuration Tests
//! 2. Proposal Submission Extrinsic Tests
//! 3. Proposal Voting & Quorum Tests
//! 4. Proposal Execution Extrinsic Tests
//! 5. Query & Helper Routine Tests

#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        EjectionReason, Error, Event, ProposalAction, ProposalStatus,
        GovernanceModeEnabled, Proposals, NextProposalId, ProposalVotes,
    };
    use frame_support::{assert_err, assert_noop, assert_ok};
    use sp_runtime::DispatchError;

    // Named account constants
    const PROPOSER: u64 = 1;
    const TARGET_VAL: u64 = 2;
    const VOTER_A: u64 = 3;
    const RATE_LIMITED_USER: u64 = 888;

    /// Helper function to advance block numbers for block-based state transitions
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    // ============================================================================
    // 1. Governance Mode Configuration Tests
    // ============================================================================
    mod governance_mode_tests {
        use super::*;

        #[test]
        fn test_set_governance_mode_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::set_governance_mode(
                    RuntimeOrigin::root(),
                    true
                ));
                assert_eq!(GovernanceModeEnabled::<Test>::get(), true);
                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::GovernanceModeSet {
                    enabled: true,
                }));

                assert_ok!(PalletCbcGovernance::set_governance_mode(
                    RuntimeOrigin::root(),
                    false
                ));
                assert_eq!(GovernanceModeEnabled::<Test>::get(), false);
                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::GovernanceModeSet {
                    enabled: false,
                }));
            });
        }

        #[test]
        fn test_set_governance_mode_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcGovernance::set_governance_mode(RuntimeOrigin::signed(PROPOSER), true),
                    DispatchError::BadOrigin
                );
            });
        }
    }

    // ============================================================================
    // 2. Proposal Submission Extrinsic Tests
    // ============================================================================
    mod submission_tests {
        use super::*;

        #[test]
        fn test_submit_proposal_slash_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let action = ProposalAction::Slash {
                    validator: TARGET_VAL,
                    amount: 1000,
                };
                assert_ok!(PalletCbcGovernance::submit_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    action.clone(),
                    None
                ));

                assert_eq!(NextProposalId::<Test>::get(), 1);
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.proposer, PROPOSER);
                assert_eq!(prop.action, action);
                assert_eq!(prop.status, ProposalStatus::Pending);
                assert_eq!(prop.votes_for, 0);
                assert_eq!(prop.votes_against, 0);

                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalSubmitted {
                    proposal_id: 0,
                    proposer: PROPOSER,
                    action: action.clone(),
                }));
            });
        }

        #[test]
        fn test_propose_slash_validator() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_eq!(NextProposalId::<Test>::get(), 1);
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::Slash {
                        validator: TARGET_VAL,
                        amount: 500
                    }
                );
            });
        }

        #[test]
        fn test_propose_reward_validator() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_reward_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    1500,
                    None
                ));
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::Reward {
                        validator: TARGET_VAL,
                        amount: 1500
                    }
                );
            });
        }

        #[test]
        fn test_propose_default_reward_validator() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_default_reward_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    1000,
                    None
                ));
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::Reward {
                        validator: TARGET_VAL,
                        amount: 1000
                    }
                );
            });
        }

        #[test]
        fn test_propose_reward_multiple_validators() {
            new_test_ext().execute_with(|| {
                assert_ok!(
                    PalletCbcGovernance::propose_reward_multiple_validators(
                        RuntimeOrigin::signed(PROPOSER),
                        vec![TARGET_VAL, VOTER_A],
                        2000,
                        None
                    )
                );
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::RewardMultiple {
                        validators: vec![TARGET_VAL, VOTER_A],
                        amount: 2000
                    }
                );
            });
        }

        #[test]
        fn test_propose_default_reward_multiple_validators() {
            new_test_ext().execute_with(|| {
                assert_ok!(
                    PalletCbcGovernance::propose_default_reward_multiple_validators(
                        RuntimeOrigin::signed(PROPOSER),
                        vec![TARGET_VAL, VOTER_A],
                        2000,
                        None
                    )
                );
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::RewardMultiple {
                        validators: vec![TARGET_VAL, VOTER_A],
                        amount: 2000
                    }
                );
            });
        }

        #[test]
        fn test_propose_reward_all_active_validators() {
            new_test_ext().execute_with(|| {
                assert_ok!(
                    PalletCbcGovernance::propose_reward_all_active_validators(
                        RuntimeOrigin::signed(PROPOSER),
                        3000,
                        None
                    )
                );
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::RewardMultiple {
                        validators: vec![1, 2, 3],
                        amount: 3000
                    }
                );
            });
        }

        #[test]
        fn test_propose_eject_validator() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_eject_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    EjectionReason::GovernanceDecision,
                    None
                ));
                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(
                    prop.action,
                    ProposalAction::Eject {
                        validator: TARGET_VAL,
                        reason: EjectionReason::GovernanceDecision
                    }
                );
            });
        }

        #[test]
        fn test_submit_proposal_rate_limit_violation() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                let action = ProposalAction::Slash {
                    validator: TARGET_VAL,
                    amount: 1000,
                };
                assert_err!(
                    PalletCbcGovernance::submit_proposal(
                        RuntimeOrigin::signed(RATE_LIMITED_USER),
                        action,
                        None
                    ),
                    Error::<Test>::RateLimitExceeded
                );
            });
        }
    }

    // ============================================================================
    // 3. Proposal Voting & Quorum Tests
    // ============================================================================
    mod voting_tests {
        use super::*;

        #[test]
        fn test_vote_proposal_approve_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.votes_for, 1);
                assert_eq!(prop.votes_against, 0);
                assert_eq!(ProposalVotes::<Test>::get(0, PROPOSER), Some(true));

                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalVoted {
                    proposal_id: 0,
                    voter: PROPOSER,
                    approve: true,
                }));
            });
        }

        #[test]
        fn test_vote_proposal_against_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    false
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.votes_for, 0);
                assert_eq!(prop.votes_against, 1);
                assert_eq!(ProposalVotes::<Test>::get(0, PROPOSER), Some(false));
            });
        }

        #[test]
        fn test_vote_proposal_already_voted() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));

                assert_noop!(
                    PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(PROPOSER), 0, true),
                    Error::<Test>::AlreadyVoted
                );
            });
        }

        #[test]
        fn test_vote_proposal_not_found() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(PROPOSER), 999, true),
                    Error::<Test>::ProposalNotFound
                );
            });
        }

        #[test]
        fn test_vote_proposal_quorum_passed() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));

                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Approved);
                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalPassed {
                    proposal_id: 0,
                }));
            });
        }

        #[test]
        fn test_vote_proposal_quorum_rejected() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));

                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    false
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    false
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Rejected);
                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalRejected {
                    proposal_id: 0,
                }));
            });
        }

        #[test]
        fn test_vote_proposal_already_executed() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));

                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                assert_noop!(
                    PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(VOTER_A), 0, true),
                    Error::<Test>::ProposalAlreadyExecuted
                );
            });
        }

        #[test]
        fn test_vote_proposal_rate_limit_violation() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));

                assert_err!(
                    PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(RATE_LIMITED_USER), 0, true),
                    Error::<Test>::RateLimitExceeded
                );
            });
        }
    }

    // ============================================================================
    // 4. Proposal Execution Extrinsic Tests
    // ============================================================================
    mod execution_tests {
        use super::*;

        #[test]
        fn test_execute_proposal_slash_success() {
            new_test_ext().execute_with(|| {
                run_to_block(1);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));

                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Executed);

                System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalExecuted {
                    proposal_id: 0,
                    action: ProposalAction::Slash {
                        validator: TARGET_VAL,
                        amount: 500,
                    },
                }));
            });
        }

        #[test]
        fn test_execute_proposal_reward_success() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_reward_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    1000,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Executed);
            });
        }

        #[test]
        fn test_execute_proposal_reward_multiple_success() {
            new_test_ext().execute_with(|| {
                assert_ok!(
                    PalletCbcGovernance::propose_reward_multiple_validators(
                        RuntimeOrigin::signed(PROPOSER),
                        vec![TARGET_VAL, VOTER_A],
                        1000,
                        None
                    )
                );
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Executed);
            });
        }

        #[test]
        fn test_execute_proposal_eject_success() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_eject_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    EjectionReason::GovernanceDecision,
                    None
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(PROPOSER),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::vote_proposal(
                    RuntimeOrigin::signed(TARGET_VAL),
                    0,
                    true
                ));
                assert_ok!(PalletCbcGovernance::execute_proposal(
                    RuntimeOrigin::root(),
                    0
                ));

                let prop = Proposals::<Test>::get(0).unwrap();
                assert_eq!(prop.status, ProposalStatus::Executed);
            });
        }

        #[test]
        fn test_execute_proposal_not_approved() {
            new_test_ext().execute_with(|| {
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));

                assert_noop!(
                    PalletCbcGovernance::execute_proposal(RuntimeOrigin::root(), 0),
                    Error::<Test>::ProposalNotApproved
                );
            });
        }

        #[test]
        fn test_execute_proposal_not_found() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcGovernance::execute_proposal(RuntimeOrigin::root(), 999),
                    Error::<Test>::ProposalNotFound
                );
            });
        }

        #[test]
        fn test_execute_proposal_bad_origin() {
            new_test_ext().execute_with(|| {
                assert_noop!(
                    PalletCbcGovernance::execute_proposal(RuntimeOrigin::signed(PROPOSER), 0),
                    DispatchError::BadOrigin
                );
            });
        }
    }

    // ============================================================================
    // 5. Query & Helper Routine Tests
    // ============================================================================
    mod helper_tests {
        use super::*;

        #[test]
        fn test_get_proposal_details() {
            new_test_ext().execute_with(|| {
                assert_eq!(PalletCbcGovernance::get_proposal_details(0), None);
                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));

                let details = PalletCbcGovernance::get_proposal_details(0);
                assert!(details.is_some());
                assert_eq!(details.unwrap().proposer, PROPOSER);
            });
        }

        #[test]
        fn test_get_active_proposals() {
            new_test_ext().execute_with(|| {
                assert!(PalletCbcGovernance::get_active_proposals().is_empty());

                assert_ok!(PalletCbcGovernance::propose_slash_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    TARGET_VAL,
                    500,
                    None
                ));
                assert_ok!(PalletCbcGovernance::propose_reward_validator(
                    RuntimeOrigin::signed(PROPOSER),
                    VOTER_A,
                    1000,
                    None
                ));

                let active = PalletCbcGovernance::get_active_proposals();
                assert_eq!(active.len(), 2);
            });
        }
    }
}
