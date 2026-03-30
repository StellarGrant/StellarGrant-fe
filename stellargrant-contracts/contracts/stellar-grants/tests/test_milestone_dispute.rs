use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};
use stellar_grants::{MilestoneState, StellarGrantsContractClient, COMMUNITY_REVIEW_PERIOD};

#[test]
fn test_dispute_and_resolve_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let council = Address::generate(&env);
    let owner = Address::generate(&env);
    let reviewer = Address::generate(&env);
    let token_admin_addr = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &council);
    let mut reviewers: Vec<Address> = Vec::new(&env);
    reviewers.push_back(reviewer.clone());
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Desc"),
        &token,
        &1000,
        &1000,
        &1,
        &reviewers,
        &1,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    client.grant_accept(&grant_id, &owner);
    let funder = Address::generate(&env);
    token_admin.mint(&funder, &1000);
    client.grant_fund(&grant_id, &funder, &1000, &token, &None);
    client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "Milestone 1"),
        &String::from_str(&env, "proof"),
        &None,
    );
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);
    client.milestone_vote(&grant_id, &0, &reviewer, &true, &None);
    client.dispute_milestone(&grant_id, &0, &owner);
    client.resolve_dispute(&council, &grant_id, &0, &true);
    let milestone = client.get_milestone(&grant_id, &0);
    assert_eq!(milestone.state, MilestoneState::Resolved);
}

#[test]
#[should_panic]
fn test_vote_blocked_during_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let council = Address::generate(&env);
    let owner = Address::generate(&env);
    let reviewer = Address::generate(&env);
    let token_admin_addr = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &council);
    let mut reviewers: Vec<Address> = Vec::new(&env);
    reviewers.push_back(reviewer.clone());
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Desc"),
        &token,
        &1000,
        &1000,
        &1,
        &reviewers,
        &1,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    client.grant_accept(&grant_id, &owner);
    let funder = Address::generate(&env);
    token_admin.mint(&funder, &1000);
    client.grant_fund(&grant_id, &funder, &1000, &token, &None);
    client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "Milestone 1"),
        &String::from_str(&env, "proof"),
        &None,
    );
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);
    client.milestone_vote(&grant_id, &0, &reviewer, &true, &None);
    client.dispute_milestone(&grant_id, &0, &owner);
    // This should panic
    client.milestone_vote(&grant_id, &0, &reviewer, &true, &None);
}

#[test]
#[should_panic]
fn test_only_council_can_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let council = Address::generate(&env);
    let owner = Address::generate(&env);
    let reviewer = Address::generate(&env);
    let token_admin_addr = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &council);
    let mut reviewers: Vec<Address> = Vec::new(&env);
    reviewers.push_back(reviewer.clone());
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Desc"),
        &token,
        &1000,
        &1000,
        &1,
        &reviewers,
        &1,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    client.grant_accept(&grant_id, &owner);
    let funder = Address::generate(&env);
    token_admin.mint(&funder, &1000);
    client.grant_fund(&grant_id, &funder, &1000, &token, &None);
    client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "Milestone 1"),
        &String::from_str(&env, "proof"),
        &None,
    );
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);
    client.milestone_vote(&grant_id, &0, &reviewer, &true, &None);
    client.dispute_milestone(&grant_id, &0, &owner);
    // This should panic (not council)
    client.resolve_dispute(&owner, &grant_id, &0, &true);
}
