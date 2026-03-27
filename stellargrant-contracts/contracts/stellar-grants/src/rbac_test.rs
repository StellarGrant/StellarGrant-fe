#[cfg(test)]
mod tests {
    use crate::types::{ContractError, Role};
    use crate::StellarGrantsContract;
    use crate::StellarGrantsContractClient;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn setup_test(env: &Env) -> (StellarGrantsContractClient<'_>, Address) {
        let contract_id = env.register(StellarGrantsContract, ());
        let client = StellarGrantsContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(&admin);
        (client, admin)
    }

    #[test]
    fn test_rbac_initialization() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        assert!(client.has_role(&admin, &Role::Admin));
    }

    #[test]
    fn test_grant_role_success() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        let user = Address::generate(&env);
        
        env.mock_all_auths();
        client.grant_role(&admin, &user, &Role::GrantCreator);
        
        assert!(client.has_role(&user, &Role::GrantCreator));
    }

    #[test]
    fn test_grant_role_unauthorized() {
        let env = Env::default();
        let (client, _admin) = setup_test(&env);
        let user = Address::generate(&env);
        let target = Address::generate(&env);
        
        env.mock_all_auths();
        let result = client.try_grant_role(&user, &target, &Role::Admin);
        
        assert_eq!(result, Err(Ok(ContractError::Unauthorized.into())));
    }

    #[test]
    fn test_revoke_role_success() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        let user = Address::generate(&env);
        
        env.mock_all_auths();
        client.grant_role(&admin, &user, &Role::Pauser);
        assert!(client.has_role(&user, &Role::Pauser));
        
        client.revoke_role(&admin, &user, &Role::Pauser);
        assert!(!client.has_role(&user, &Role::Pauser));
    }

    #[test]
    fn test_renounce_role_success() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        let user = Address::generate(&env);
        
        env.mock_all_auths();
        client.grant_role(&admin, &user, &Role::Reviewer);
        
        client.renounce_role(&user, &Role::Reviewer);
        assert!(!client.has_role(&user, &Role::Reviewer));
    }

    #[test]
    fn test_grant_create_rbac() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        let creator = Address::generate(&env);
        let token = Address::generate(&env);
        
        env.mock_all_auths();
        
        // Fails without role
        let result = client.try_grant_create(
            &creator,
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Desc"),
            &token,
            &1000,
            &500,
            &2,
            &Vec::new(&env),
        );
        assert_eq!(result, Err(Ok(ContractError::Unauthorized.into())));
        
        // Succeeds with role
        client.grant_role(&admin, &creator, &Role::GrantCreator);
        let grant_id = client.grant_create(
            &creator,
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Desc"),
            &token,
            &1000,
            &500,
            &2,
            &Vec::new(&env),
        );
        assert_eq!(grant_id, 1);
    }

    #[test]
    fn test_emergency_pause_rbac() {
        let env = Env::default();
        let (client, admin) = setup_test(&env);
        let pauser = Address::generate(&env);
        
        env.mock_all_auths();
        
        // User cannot pause without role
        let result = client.try_set_paused(&pauser, &true);
        assert_eq!(result, Err(Ok(ContractError::Unauthorized.into())));
        
        // Pauser can pause
        client.grant_role(&admin, &pauser, &Role::Pauser);
        client.set_paused(&pauser, &true);
        
        // Actions are blocked while paused
        let creator = Address::generate(&env);
        client.grant_role(&admin, &creator, &Role::GrantCreator);
        let result_create = client.try_grant_create(
            &creator,
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Desc"),
            &Address::generate(&env),
            &1000,
            &500,
            &2,
            &Vec::new(&env),
        );
        assert_eq!(result_create, Err(Ok(ContractError::Unauthorized.into())));
        
        // Only Admin can unpause
        let result_unpause = client.try_set_paused(&pauser, &false);
        assert_eq!(result_unpause, Err(Ok(ContractError::Unauthorized.into())));
        
        client.set_paused(&admin, &false);
        
        // Actions work again
        client.grant_create(
            &creator,
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Desc"),
            &Address::generate(&env),
            &1000,
            &500,
            &2,
            &Vec::new(&env),
        );
    }
}
