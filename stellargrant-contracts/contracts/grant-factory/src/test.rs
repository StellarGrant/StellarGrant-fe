#[cfg(test)]
mod tests {
    use crate::{StellarGrantsFactory, StellarGrantsFactoryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec, BytesN};

    fn setup_test(env: &Env) -> (StellarGrantsFactoryClient<'_>, Address) {
        let contract_id = env.register(StellarGrantsFactory, ());
        let client = StellarGrantsFactoryClient::new(env, &contract_id);
        let admin = Address::generate(env);
        (client, admin)
    }

    #[test]
    fn test_initialize_factory() {
        let env = Env::default();
        let (client, _) = setup_test(&env);
        let dummy_hash = BytesN::from_array(&env, &[0; 32]);
        
        client.initialize(&dummy_hash);

        let result = client.try_initialize(&dummy_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_contributor_register_success() {
        let env = Env::default();
        env.mock_all_auths();

        let (client, _) = setup_test(&env);
        let contributor = Address::generate(&env);

        let name = String::from_str(&env, "Alice");
        let bio = String::from_str(&env, "Rust Developer");
        let mut skills = Vec::new(&env);
        skills.push_back(String::from_str(&env, "Rust"));
        skills.push_back(String::from_str(&env, "Soroban"));
        let github_url = String::from_str(&env, "https://github.com/alice");

        client.contributor_register(&contributor, &name, &bio, &skills, &github_url);

        let result =
            client.try_contributor_register(&contributor, &name, &bio, &skills, &github_url);
        assert!(result.is_err());
    }
}
