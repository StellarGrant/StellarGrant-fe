use soroban_sdk::testutils::Events;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger},
    token, Address, Env, String, Vec,
};
use stellar_grants::StellarGrantsContractClient;

#[test]
fn test_event_emission_on_grant_create_and_fund() {
    let env = Env::default();
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let token_admin_addr = <Address as TestAddress>::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 1u32;
    env.mock_all_auths();
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Event Grant"),
        &String::from_str(&env, "Testing events"),
        &token,
        &100,
        &10,
        &1,
        &reviewers,
        &quorum,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    let funder = <Address as TestAddress>::generate(&env);
    client.grant_accept(&grant_id, &owner);
    token_admin.mint(&funder, &100);
    client.grant_fund(&grant_id, &funder, &100, &token, &None);
    let events = env.events().all();
    // Debug print all events
    println!("All events:");
    for e in events.events() {
        println!("{:?}", e);
    }
    // Check that grant_funded event is present and well-formed
    let mut found_grant_funded = false;
    for e in events.events() {
        let s = format!("{:?}", e);
        if s.contains("grant_funded") {
            found_grant_funded = true;
        }
    }
    assert!(found_grant_funded, "grant_funded event not found");
}

#[test]
fn test_event_emission_on_milestone_vote() {
    let env = Env::default();
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let token_admin_addr = <Address as TestAddress>::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);
    let mut reviewers = Vec::new(&env);
    let reviewer = <Address as TestAddress>::generate(&env);
    reviewers.push_back(reviewer.clone());
    let quorum = 1u32;
    env.mock_all_auths();
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Event Grant"),
        &String::from_str(&env, "Testing events"),
        &token,
        &100,
        &10,
        &1,
        &reviewers,
        &quorum,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    let funder = <Address as TestAddress>::generate(&env);
    client.grant_accept(&grant_id, &owner);
    token_admin.mint(&funder, &100);
    client.grant_fund(&grant_id, &funder, &100, &token, &None);
    client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
        &None,
    );
    // Advance ledger timestamp by COMMUNITY_REVIEW_PERIOD to allow voting
    const COMMUNITY_REVIEW_PERIOD: u64 = 3 * 24 * 60 * 60;
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);
    client.milestone_vote(&grant_id, &0, &reviewer, &true, &None);
    let events = env.events().all();
    // Debug print all events
    println!("All events:");
    for e in events.events() {
        println!("{:?}", e);
    }
    // Check that milestone_voted and quorum_reached events are present and well-formed
    let mut found_milestone_voted = false;
    let mut found_quorum_reached = false;
    for e in events.events() {
        let s = format!("{:?}", e);
        if s.contains("milestone_voted") {
            found_milestone_voted = true;
        }
        if s.contains("quorum_reached") {
            found_quorum_reached = true;
        }
    }
    assert!(found_milestone_voted, "milestone_voted event not found");
    assert!(found_quorum_reached, "quorum_reached event not found");
}
