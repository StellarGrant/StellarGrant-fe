#![cfg(test)]
mod tests {
    use crate::StellarGrantsContract;
    use crate::StellarGrantsContractClient;
    use crate::types::{GrantStatus, ContractError};
    use soroban_sdk::{
        testutils::{Address as _},
        token, Address, Env, Vec, String,
    };

    fn setup_test(env: &Env) -> (StellarGrantsContractClient<'_>, Address) {
        let contract_id = env.register(StellarGrantsContract, ());
        let client = StellarGrantsContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        (client, admin)
    }

    #[test]
    fn test_pending_funding_basic_flow() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        client.initialize(&admin);

        let owner = Address::generate(&env);
        let funder = Address::generate(&env);
        let token = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let mut reviewers = Vec::new(&env);
        reviewers.push_back(reviewer);

        let total_amount = 1000;
        let min_funding = 500;
        let milestone_amount = 250;
        let num_milestones = 2;

        // Create token contract and mint tokens
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_client = token::StellarAssetClient::new(&env, &token_contract.address());
        
        env.mock_all_auths();
        token_client.mint(&funder, &1000);
        
        // Create grant - should start in PendingFunding
        let grant_id = client.grant_create(
            &owner,
            &String::from_str(&env, "Test Grant"),
            &String::from_str(&env, "Description"),
            &token_contract.address(),
            &total_amount,
            &min_funding,
            &milestone_amount,
            &num_milestones,
            &reviewers,
            &None,
        );

        let grant = client.get_grant(&grant_id);
        assert_eq!(grant.status, GrantStatus::PendingFunding);
        assert_eq!(grant.min_funding, min_funding);
        assert_eq!(grant.escrow_balance, 0);

        // Fund with less than min_funding - should remain PendingFunding
        client.grant_fund(&grant_id, &funder, &(min_funding - 100));
        let grant = client.get_grant(&grant_id);
        assert_eq!(grant.status, GrantStatus::PendingFunding);
        assert_eq!(grant.escrow_balance, min_funding - 100);

        // Fund to reach min_funding - should become Active
        client.grant_fund(&grant_id, &funder, &100);
        let grant = client.get_grant(&grant_id);
        assert_eq!(grant.status, GrantStatus::Active);
        assert_eq!(grant.escrow_balance, min_funding);
    }

    #[test]
    fn test_milestone_submit_rejected_pending_funding() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        client.initialize(&admin);

        let owner = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let reviewer = Address::generate(&env);
        let mut reviewers = Vec::new(&env);
        reviewers.push_back(reviewer);

        env.mock_all_auths();
        
        let grant_id = client.grant_create(
            &owner,
            &String::from_str(&env, "Test Grant"),
            &String::from_str(&env, "Description"),
            &token_contract.address(),
            &1000,
            &500,
            &250,
            &2,
            &reviewers,
            &None,
        );

        // Try to submit milestone while grant is in PendingFunding
        let result = client.try_milestone_submit(
            &grant_id,
            &0,
            &owner,
            &String::from_str(&env, "Milestone 1"),
            &String::from_str(&env, "https://proof.url"),
        );

        assert_eq!(result, Err(Ok(ContractError::InvalidState.into())));
    }
}
