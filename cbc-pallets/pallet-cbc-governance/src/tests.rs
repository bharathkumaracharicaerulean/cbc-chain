#[cfg(test)]
mod tests {
    use crate::mock::*;
    use crate::{
        EjectionReason, Error, Event, ProposalAction, ProposalStatus,
    };
    use frame_support::{assert_err, assert_noop, assert_ok};
    use sp_runtime::DispatchError;

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            System::set_block_number(System::block_number() + 1);
        }
    }

    // ================================================================================================
    // 1. Governance Mode Configuration Tests
    // ================================================================================================

    #[test]
    fn test_set_governance_mode_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::set_governance_mode(
                RuntimeOrigin::root(),
                true
            ));
            assert_eq!(PalletCbcGovernance::governance_mode_enabled(), true);
            System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::GovernanceModeSet {
                enabled: true,
            }));

            assert_ok!(PalletCbcGovernance::set_governance_mode(
                RuntimeOrigin::root(),
                false
            ));
            assert_eq!(PalletCbcGovernance::governance_mode_enabled(), false);
            System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::GovernanceModeSet {
                enabled: false,
            }));
        });
    }

    #[test]
    fn test_set_governance_mode_bad_origin() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcGovernance::set_governance_mode(RuntimeOrigin::signed(1), true),
                DispatchError::BadOrigin
            );
        });
    }

    // ================================================================================================
    // 2. Proposal Submission Tests
    // ================================================================================================

    #[test]
    fn test_submit_proposal_slash_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            let action = ProposalAction::Slash {
                validator: 2,
                amount: 1000,
            };
            assert_ok!(PalletCbcGovernance::submit_proposal(
                RuntimeOrigin::signed(1),
                action.clone(),
                None
            ));

            assert_eq!(PalletCbcGovernance::next_proposal_id(), 1);
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.proposer, 1);
            assert_eq!(prop.action, action);
            assert_eq!(prop.status, ProposalStatus::Pending);
            assert_eq!(prop.votes_for, 0);
            assert_eq!(prop.votes_against, 0);

            System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalSubmitted {
                proposal_id: 0,
                proposer: 1,
                action: action.clone(),
            }));
        });
    }

    #[test]
    fn test_propose_slash_validator() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));
            assert_eq!(PalletCbcGovernance::next_proposal_id(), 1);
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::Slash {
                    validator: 2,
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
                RuntimeOrigin::signed(1),
                2,
                1500,
                None
            ));
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::Reward {
                    validator: 2,
                    amount: 1500
                }
            );
        });
    }

    #[test]
    fn test_propose_default_reward_validator() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::propose_default_reward_validator(
                RuntimeOrigin::signed(1),
                2,
                1000,
                None
            ));
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::Reward {
                    validator: 2,
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
                    RuntimeOrigin::signed(1),
                    vec![2, 3],
                    2000,
                    None
                )
            );
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::RewardMultiple {
                    validators: vec![2, 3],
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
                    RuntimeOrigin::signed(1),
                    vec![2, 3],
                    2000,
                    None
                )
            );
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::RewardMultiple {
                    validators: vec![2, 3],
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
                    RuntimeOrigin::signed(1),
                    3000,
                    None
                )
            );
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            // MockValidatorProvider returns active validators vec![1, 2, 3]
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
                RuntimeOrigin::signed(1),
                2,
                EjectionReason::GovernanceDecision,
                None
            ));
            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(
                prop.action,
                ProposalAction::Eject {
                    validator: 2,
                    reason: EjectionReason::GovernanceDecision
                }
            );
        });
    }

    #[test]
    fn test_submit_proposal_rate_limit_violation() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            // Proposer 888 triggers rate limit check failure in MockValidatorProvider
            let action = ProposalAction::Slash {
                validator: 2,
                amount: 1000,
            };
            assert_err!(
                PalletCbcGovernance::submit_proposal(
                    RuntimeOrigin::signed(888),
                    action,
                    None
                ),
                Error::<Test>::RateLimitExceeded
            );
        });
    }

    // ================================================================================================
    // 3. Proposal Voting & Quorum Tests
    // ================================================================================================

    #[test]
    fn test_vote_proposal_approve_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.votes_for, 1);
            assert_eq!(prop.votes_against, 0);
            assert_eq!(PalletCbcGovernance::proposal_votes(0, 1), Some(true));

            System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalVoted {
                proposal_id: 0,
                voter: 1,
                approve: true,
            }));
        });
    }

    #[test]
    fn test_vote_proposal_against_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                false
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.votes_for, 0);
            assert_eq!(prop.votes_against, 1);
            assert_eq!(PalletCbcGovernance::proposal_votes(0, 1), Some(false));
        });
    }

    #[test]
    fn test_vote_proposal_already_voted() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));

            assert_noop!(
                PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(1), 0, true),
                Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn test_vote_proposal_not_found() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(1), 999, true),
                Error::<Test>::ProposalNotFound
            );
        });
    }

    #[test]
    fn test_vote_proposal_quorum_passed() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            // Active validators count = 3. Quorum = (3 + 1) / 2 = 2.
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
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
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                false
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                false
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
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
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            // Quorum pass
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));

            // Execute
            assert_ok!(PalletCbcGovernance::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));

            // Try to vote on executed proposal
            assert_noop!(
                PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(3), 0, true),
                Error::<Test>::ProposalAlreadyExecuted
            );
        });
    }

    #[test]
    fn test_vote_proposal_rate_limit_violation() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            // Voter 888 triggers rate limit check failure
            assert_err!(
                PalletCbcGovernance::vote_proposal(RuntimeOrigin::signed(888), 0, true),
                Error::<Test>::RateLimitExceeded
            );
        });
    }

    // ================================================================================================
    // 4. Proposal Execution Tests
    // ================================================================================================

    #[test]
    fn test_execute_proposal_slash_success() {
        new_test_ext().execute_with(|| {
            run_to_block(1);
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));

            assert_ok!(PalletCbcGovernance::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.status, ProposalStatus::Executed);

            System::assert_has_event(RuntimeEvent::PalletCbcGovernance(Event::ProposalExecuted {
                proposal_id: 0,
                action: ProposalAction::Slash {
                    validator: 2,
                    amount: 500,
                },
            }));
        });
    }

    #[test]
    fn test_execute_proposal_reward_success() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::propose_reward_validator(
                RuntimeOrigin::signed(1),
                2,
                1000,
                None
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.status, ProposalStatus::Executed);
        });
    }

    #[test]
    fn test_execute_proposal_reward_multiple_success() {
        new_test_ext().execute_with(|| {
            assert_ok!(
                PalletCbcGovernance::propose_reward_multiple_validators(
                    RuntimeOrigin::signed(1),
                    vec![2, 3],
                    1000,
                    None
                )
            );
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.status, ProposalStatus::Executed);
        });
    }

    #[test]
    fn test_execute_proposal_eject_success() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::propose_eject_validator(
                RuntimeOrigin::signed(1),
                2,
                EjectionReason::GovernanceDecision,
                None
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(1),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::vote_proposal(
                RuntimeOrigin::signed(2),
                0,
                true
            ));
            assert_ok!(PalletCbcGovernance::execute_proposal(
                RuntimeOrigin::root(),
                0
            ));

            let prop = PalletCbcGovernance::proposals(0).unwrap();
            assert_eq!(prop.status, ProposalStatus::Executed);
        });
    }

    #[test]
    fn test_execute_proposal_not_approved() {
        new_test_ext().execute_with(|| {
            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            // Executing while status is still Pending
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
                PalletCbcGovernance::execute_proposal(RuntimeOrigin::signed(1), 0),
                DispatchError::BadOrigin
            );
        });
    }

    // ================================================================================================
    // 5. Query & Helper Routine Tests
    // ================================================================================================

    #[test]
    fn test_get_proposal_details() {
        new_test_ext().execute_with(|| {
            assert_eq!(PalletCbcGovernance::get_proposal_details(0), None);

            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));

            let details = PalletCbcGovernance::get_proposal_details(0);
            assert!(details.is_some());
            assert_eq!(details.unwrap().proposer, 1);
        });
    }

    #[test]
    fn test_get_active_proposals() {
        new_test_ext().execute_with(|| {
            assert!(PalletCbcGovernance::get_active_proposals().is_empty());

            assert_ok!(PalletCbcGovernance::propose_slash_validator(
                RuntimeOrigin::signed(1),
                2,
                500,
                None
            ));
            assert_ok!(PalletCbcGovernance::propose_reward_validator(
                RuntimeOrigin::signed(1),
                3,
                1000,
                None
            ));

            let active = PalletCbcGovernance::get_active_proposals();
            assert_eq!(active.len(), 2);
        });
    }
}
