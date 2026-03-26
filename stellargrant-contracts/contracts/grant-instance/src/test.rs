#[cfg(test)]
mod tests {
    use crate::{GrantInstance, GrantInstanceClient, GrantStatus, MilestoneState};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec, token};

    fn setup_test(env: &Env) -> (GrantInstanceClient<'_>, Address, Address) {
        let contract_id = env.register(GrantInstance, ());
        let client = GrantInstanceClient::new(env, &contract_id);
        let admin = Address::generate(env);
        
        let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
        let token_id = token_contract.address();
        
        (client, admin, token_id)
    }

    #[test]
    fn test_initialize_grant_instance() {
        let env = Env::default();
        let (client, _, token_id) = setup_test(&env);
        let owner = Address::generate(&env);
        
        let reviewers = Vec::new(&env);
        
        client.initialize(
            &owner,
            &String::from_str(&env, "Test Grant"),
            &String::from_str(&env, "Description"),
            &token_id,
            &1000i128,
            &500i128,
            &2u32,
            &reviewers
        );

        let grant = client.get_grant();
        
        assert_eq!(grant.owner, owner);
        assert_eq!(grant.status, GrantStatus::Active);
        assert_eq!(grant.total_amount, 1000);
        assert_eq!(grant.milestone_amount, 500);
    }
}
