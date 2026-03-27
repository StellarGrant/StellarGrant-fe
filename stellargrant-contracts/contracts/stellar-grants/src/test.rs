#![allow(
    unused_variables,
    clippy::needless_borrow,
    clippy::bool_assert_comparison,
    clippy::useless_conversion,
    clippy::needless_range_loop
)]

#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use crate::types::{ContractError, Grant, GrantFund, GrantStatus, Milestone, MilestoneState};
    use crate::StellarGrantsContract;
    use crate::StellarGrantsContractClient;
    use soroban_sdk::{testutils::Address as _, token, Address, Env, Map, String, Vec};

        // Reentrant token callback attempts nested grant_fund call and must panic via lock.
        client.grant_fund(&grant_id, &attacker, &100i128);
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
            Storage::set_grant(&env, grant_id, &grant);
        });

        let mut batch = Vec::new(&env);
        batch.push_back((grant_id, 100i128));
        client.fund_batch(&attacker, &batch);
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
            Storage::set_grant(&env, grant_id, &grant);
        });

        client.stake_to_review(&attacker, &grant_id, &100i128);
    }

    #[test]
    fn test_reentrancy_guard_allows_sequential_grant_funds() {
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
    fn test_grant_create_invalid_num_milestones() {
        let env = Env::default();
        let (client, _, _) = setup_test(&env);
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewers = Vec::new(&env);
        let title = String::from_str(&env, "New Grant");
        let description = String::from_str(&env, "Some desc");

        env.mock_all_auths();

        // 0 milestones
        let res1 = client.try_grant_create(
            &owner,
            &title,
            &description,
            &token,
            &1000i128,
            &500i128,
            &0u32,
            &reviewers,
        );
        assert_eq!(res1, Err(Ok(ContractError::InvalidInput.into())));

        // > 100 milestones
        let res2 = client.try_grant_create(
            &owner,
            &title,
            &description,
            &token,
            &100000i128,
            &100i128,
            &101u32,
            &reviewers,
        );
        assert_eq!(res2, Err(Ok(ContractError::InvalidInput.into())));
    }

    #[test]
    fn test_grant_create_amount_mismatch() {
        let env = Env::default();
        let (client, _, _) = setup_test(&env);
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewers = Vec::new(&env);
        let title = String::from_str(&env, "New Grant");
        let description = String::from_str(&env, "Some desc");

        env.mock_all_auths();

        // total < milestone_amount * num_milestones
        // 800 < 500 * 2
        let res = client.try_grant_create(
            &owner,
            &title,
            &description,
            &token,
            &800i128,
            &500i128,
            &2u32,
            &reviewers,
        );
        assert_eq!(res, Err(Ok(ContractError::InvalidInput.into())));
    }

    #[test]
    fn test_grant_create_unauthorized() {
        let env = Env::default();
        let (client, _, _) = setup_test(&env);
        let owner = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewers = Vec::new(&env);
        let title = String::from_str(&env, "New Grant");
        let description = String::from_str(&env, "Some desc");

        // No mock_all_auths()

        let res = client.try_grant_create(
            &owner,
            &title,
            &description,
            &token,
            &1000i128,
            &500i128,
            &2u32,
            &reviewers,
        );
        assert!(res.is_err());
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

        // Give high_rep_reviewer a reputation of 3, low_rep_reviewer a reputation of 1
        env.as_contract(&contract_id, || {
            Storage::set_reviewer_reputation(&env, high_rep_reviewer.clone(), 3);
            Storage::set_reviewer_reputation(&env, low_rep_reviewer.clone(), 1);
        });

        env.mock_all_auths();

        // Total weight = 3 + 1 = 4. Quorum margin = (4 / 2) + 1 = 3.
        // high_rep_reviewer's vote (3 weight) should pass it alone.
        let result =
            client.milestone_vote(&grant_id, &milestone_idx, &high_rep_reviewer, &true, &None);
        assert_eq!(result, true);

        env.as_contract(&contract_id, || {
            let updated_milestone = Storage::get_milestone(&env, grant_id, milestone_idx).unwrap();
            assert_eq!(updated_milestone.state, MilestoneState::Approved);
            // After consensus, high_rep_reviewer should have 4 (3 + 1)
            assert_eq!(
                Storage::get_reviewer_reputation(&env, high_rep_reviewer.clone()),
                4
            );
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
}
