#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use crate::types::{ContractError, Grant, GrantFund, GrantStatus, Milestone, MilestoneState};
    use crate::StellarGrantsContract;
    use crate::StellarGrantsContractClient;
    use soroban_sdk::{testutils::Address as _, token, Address, Env, Map, String, Vec};

    fn setup_test(
        env: &Env,
    ) -> (
        StellarGrantsContractClient<'_>,
        Address,
        soroban_sdk::Address,
    ) {
        let contract_id = env.register(StellarGrantsContract, ());
        let client = StellarGrantsContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        (client, admin, contract_id)
    }

    fn create_grant(
        env: &Env,
        contract_id: &soroban_sdk::Address,
        grant_id: u64,
        owner: Address,
        token: Address,
        reviewers: Vec<Address>,
    ) {
        env.as_contract(contract_id, || {
            let grant = Grant {
                id: grant_id,
                owner,
                token,
                status: GrantStatus::Active,
                total_amount: 1000,
                reviewers,
                total_milestones: 1,
                milestones_paid_out: 0,
                escrow_balance: 1000,
                funders: Vec::new(env),
                reason: None,
                timestamp: env.ledger().timestamp(),
            };
            Storage::set_grant(env, grant_id, &grant);
        });
    }

    fn create_milestone(
        env: &Env,
        contract_id: &soroban_sdk::Address,
        grant_id: u64,
        milestone_idx: u32,
        state: MilestoneState,
    ) {
        env.as_contract(contract_id, || {
            let milestone = Milestone {
                idx: milestone_idx,
                description: String::from_str(env, "Description"),
                amount: 100,
                state,
                votes: Map::new(env),
                approvals: 0,
                rejections: 0,
                reasons: Map::new(env),
                status_updated_at: 0,
                proof_url: Some(String::from_str(env, "https://proof.url")),
                submission_timestamp: env.ledger().timestamp(),
            };
            Storage::set_milestone(env, grant_id, milestone_idx, &milestone);
        });
    }

    #[test]
    fn test_get_milestone_success() {
        let env = Env::default();
        let (client, _, contract_id) = setup_test(&env);
        let grant_id = 1;
        let milestone_idx = 0;
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewer = Address::generate(&env);

        let mut reviewers = Vec::new(&env);
        reviewers.push_back(reviewer.clone());
        create_grant(&env, &contract_id, grant_id, owner, token, reviewers);
        create_milestone(
            &env,
            &contract_id,
            grant_id,
            milestone_idx,
            MilestoneState::Submitted,
        );

        let milestone = client.get_milestone(&grant_id, &milestone_idx);
        assert_eq!(milestone.state, MilestoneState::Submitted);
        assert_eq!(milestone.description, String::from_str(&env, "Description"));
    }

    #[test]
    fn test_get_milestone_grant_not_found() {
        let env = Env::default();
        let (client, _, _) = setup_test(&env);
        let result = client.try_get_milestone(&99, &0);
        assert_eq!(result, Err(Ok(ContractError::GrantNotFound.into())));
    }

    #[test]
    fn test_successful_vote() {
        let env = Env::default();
        let (client, _, contract_id) = setup_test(&env);
        let grant_id = 1;
        let milestone_idx = 0;
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewer = Address::generate(&env);

        let mut reviewers = Vec::new(&env);
        reviewers.push_back(reviewer.clone());
        create_grant(&env, &contract_id, grant_id, owner, token, reviewers);
        create_milestone(
            &env,
            &contract_id,
            grant_id,
            milestone_idx,
            MilestoneState::Submitted,
        );

        env.mock_all_auths();
        let result = client.milestone_vote(&grant_id, &milestone_idx, &reviewer, &true);

        assert_eq!(result, true); // Quorum reached (1/1)

        env.as_contract(&contract_id, || {
            let updated_milestone = Storage::get_milestone(&env, grant_id, milestone_idx).unwrap();
            assert_eq!(updated_milestone.approvals, 1);
            assert_eq!(updated_milestone.state, MilestoneState::Approved);
            assert!(updated_milestone.votes.get(reviewer).unwrap());
        });
    }

    #[test]
    fn test_grant_cancel_success_multiple_funders() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, contract_id) = setup_test(&env);
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_id = token_contract.address();
        let token_admin = token::StellarAssetClient::new(&env, &token_id);

        let owner = Address::generate(&env);
        let funder1 = Address::generate(&env);
        let funder2 = Address::generate(&env);

        let total_funded = 1000i128;
        let fund1 = 600i128;
        let fund2 = 400i128;
        let remaining = 1000i128;
        let grant_id = 1u64;

        token_admin.mint(&contract_id, &remaining);

        let mut funders = Vec::new(&env);
        funders.push_back(GrantFund {
            funder: funder1.clone(),
            amount: fund1,
        });
        funders.push_back(GrantFund {
            funder: funder2.clone(),
            amount: fund2,
        });

        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            token: token_id.clone(),
            status: GrantStatus::Active,
            total_amount: total_funded,
            reviewers: Vec::new(&env),
            total_milestones: 1,
            milestones_paid_out: 0,
            escrow_balance: remaining,
            funders,
            reason: None,
            timestamp: env.ledger().timestamp(),
        };

        env.as_contract(&contract_id, || {
            Storage::set_grant(&env, grant_id, &grant);
        });

        let reason = String::from_str(&env, "Project discontinued");
        client.grant_cancel(&grant_id, &owner, &reason);

        let token_client = token::Client::new(&env, &token_id);
        assert_eq!(token_client.balance(&funder1), 600);
        assert_eq!(token_client.balance(&funder2), 400);

        env.as_contract(&contract_id, || {
            let updated_grant = Storage::get_grant(&env, grant_id).unwrap();
            assert_eq!(updated_grant.status, GrantStatus::Cancelled);
        });
    }

    #[test]
    fn test_grant_cancel_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, contract_id) = setup_test(&env);
        let owner = Address::generate(&env);
        let wrong_owner = Address::generate(&env);
        let token = Address::generate(&env);

        let grant_id = 1u64;
        create_grant(&env, &contract_id, grant_id, owner, token, Vec::new(&env));

        let reason = String::from_str(&env, "test");
        let result = client.try_grant_cancel(&grant_id, &wrong_owner, &reason);

        assert_eq!(result, Err(Ok(ContractError::Unauthorized.into())));
    }

    #[test]
    fn test_grant_cancel_invalid_state() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, contract_id) = setup_test(&env);
        let owner = Address::generate(&env);
        let token = Address::generate(&env);

        let grant_id = 1u64;
        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            token: token.clone(),
            status: GrantStatus::Completed,
            total_amount: 100,
            reviewers: Vec::new(&env),
            total_milestones: 1,
            milestones_paid_out: 1,
            escrow_balance: 0,
            funders: Vec::new(&env),
            reason: None,
            timestamp: env.ledger().timestamp(),
        };

        env.as_contract(&contract_id, || {
            Storage::set_grant(&env, grant_id, &grant);
        });

        let reason = String::from_str(&env, "test");
        let result = client.try_grant_cancel(&grant_id, &owner, &reason);

        assert_eq!(result, Err(Ok(ContractError::InvalidState.into())));
    }

    #[test]
    fn test_grant_complete_success_with_refunds() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, admin, contract_id) = setup_test(&env);
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_id = token_contract.address();
        let token_admin = token::StellarAssetClient::new(&env, &token_id);

        let owner = Address::generate(&env);
        let funder1 = Address::generate(&env);
        let funder2 = Address::generate(&env);
        let grant_id = 1u64;

        let total_funded = 1000i128; // milestone 1=300, 2=300 (total paid=600). remaining=400.
        let milestone_amount = 300i128;
        let fund1 = 600i128;
        let fund2 = 400i128;
        
        token_admin.mint(&contract_id, &total_funded);

        let mut funders = Vec::new(&env);
        funders.push_back(GrantFund {
            funder: funder1.clone(),
            amount: fund1,
        });
        funders.push_back(GrantFund {
            funder: funder2.clone(),
            amount: fund2,
        });

        // initial grant state
        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            token: token_id.clone(),
            status: GrantStatus::Active,
            total_amount: total_funded,
            reviewers: Vec::new(&env),
            total_milestones: 2,
            milestones_paid_out: 0,
            escrow_balance: total_funded,
            funders,
            reason: None,
            timestamp: env.ledger().timestamp(),
        };

        env.as_contract(&contract_id, || {
            Storage::set_grant(&env, grant_id, &grant);

            // create two approved milestones
            for i in 0..2 {
                let milestone = Milestone {
                    idx: i,
                    description: String::from_str(&env, "Desc"),
                    amount: milestone_amount,
                    state: MilestoneState::Approved, // Already approved
                    votes: Map::new(&env),
                    approvals: 1,
                    rejections: 0,
                    reasons: Map::new(&env),
                    status_updated_at: 0,
                    proof_url: None,
                    submission_timestamp: 0,
                };
                Storage::set_milestone(&env, grant_id, i, &milestone);
            }
        });

        client.grant_complete(&grant_id);

        // check refund totals
        let token_client = token::Client::new(&env, &token_id);

        // remaining = 1000 - 600 = 400
        // funder1 gets 60% of 400 = 240
        // funder2 gets 40% of 400 = 160
        assert_eq!(token_client.balance(&funder1), 240);
        assert_eq!(token_client.balance(&funder2), 160);

        // check grant state
        env.as_contract(&contract_id, || {
            let updated_grant = Storage::get_grant(&env, grant_id).unwrap();
            assert_eq!(updated_grant.status, GrantStatus::Completed);
            assert_eq!(updated_grant.escrow_balance, 0); // should be cleared
        });
    }

    #[test]
    fn test_grant_complete_pending_milestones() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, contract_id) = setup_test(&env);
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let grant_id = 1u64;

        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            token: token.clone(),
            status: GrantStatus::Active,
            total_amount: 1000,
            reviewers: Vec::new(&env),
            total_milestones: 2,
            milestones_paid_out: 0,
            escrow_balance: 1000,
            funders: Vec::new(&env),
            reason: None,
            timestamp: 0,
        };

        env.as_contract(&contract_id, || {
            Storage::set_grant(&env, grant_id, &grant);

            let m1 = Milestone {
                idx: 0,
                description: String::from_str(&env, "M1"),
                amount: 500,
                state: MilestoneState::Approved, // approved
                votes: Map::new(&env),
                approvals: 1,
                rejections: 0,
                reasons: Map::new(&env),
                status_updated_at: 0,
                proof_url: None,
                submission_timestamp: 0,
            };
            Storage::set_milestone(&env, grant_id, 0, &m1);

            let m2 = Milestone {
                idx: 1,
                description: String::from_str(&env, "M2"),
                amount: 500,
                state: MilestoneState::Pending, // PENDING!
                votes: Map::new(&env),
                approvals: 0,
                rejections: 0,
                reasons: Map::new(&env),
                status_updated_at: 0,
                proof_url: None,
                submission_timestamp: 0,
            };
            Storage::set_milestone(&env, grant_id, 1, &m2);
        });

        let result = client.try_grant_complete(&grant_id);
        assert_eq!(
            result,
            Err(Ok(ContractError::NotAllMilestonesApproved.into()))
        );
    }

    #[test]
    fn test_grant_complete_exact_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _, contract_id) = setup_test(&env);
        let owner = Address::generate(&env);
        let token_id = Address::generate(&env); // don't need real token if 0 refund
        let grant_id = 1u64;

        let total_funded = 500i128; // milestone 1=500 -> remaining=0

        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            token: token_id.clone(),
            status: GrantStatus::Active,
            total_amount: total_funded,
            reviewers: Vec::new(&env),
            total_milestones: 1,
            milestones_paid_out: 0,
            escrow_balance: total_funded, // exact match
            funders: Vec::new(&env),
            reason: None,
            timestamp: 0,
        };

        env.as_contract(&contract_id, || {
            Storage::set_grant(&env, grant_id, &grant);

            let m1 = Milestone {
                idx: 0,
                description: String::from_str(&env, "M1"),
                amount: 500,
                state: MilestoneState::Approved, // approved
                votes: Map::new(&env),
                approvals: 1,
                rejections: 0,
                reasons: Map::new(&env),
                status_updated_at: 0,
                proof_url: None,
                submission_timestamp: 0,
            };
            Storage::set_milestone(&env, grant_id, 0, &m1);
        });

        client.grant_complete(&grant_id);

        env.as_contract(&contract_id, || {
            let updated_grant = Storage::get_grant(&env, grant_id).unwrap();
            assert_eq!(updated_grant.status, GrantStatus::Completed);
            assert_eq!(updated_grant.escrow_balance, 0);
        });
    }
}
