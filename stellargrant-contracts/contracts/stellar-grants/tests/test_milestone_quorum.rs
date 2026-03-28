use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger},
    Address, Env, String, Vec,
    testutils::Address as TestAddress, testutils::Ledger, Address, Env, String, Vec,
};
use stellar_grants::{MilestoneState, StellarGrantsContractClient, Storage};

#[test]

fn test_milestone_voting_quorum_and_events() {
    let env = Env::default();
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let token = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 2u32;

    // Allow all require_auth to pass in test
    env.mock_all_auths();
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Testing"),
        &token,
        &100,
        &10,
        &3,
        &reviewers,
        &quorum,
        &None,
    );

    // Submit milestone
    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
    );
    // Advance ledger timestamp by COMMUNITY_REVIEW_PERIOD to allow voting
    const COMMUNITY_REVIEW_PERIOD: u64 = 3 * 24 * 60 * 60;
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);

    // Advance past community review period (3 days)
    env.ledger().set_timestamp(3 * 24 * 60 * 60 + 1);

    // Reviewer 1 votes approve
    let res1 = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
    assert_eq!(res1, false); // Quorum not reached yet
                             // Reviewer 2 votes approve (should reach quorum)
    let res2 = client.milestone_vote(&grant_id, &0, &reviewers.get(1).unwrap(), &true, &None);
    assert_eq!(res2, true); // Quorum reached

    // Check milestone state is Approved using contract view method
    let milestone = client.get_milestone(&grant_id, &0);

    assert_eq!(milestone.state, MilestoneState::Approved);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_milestone_vote_after_quorum_panics() {
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, Vec};
    use stellar_grants::StellarGrantsContractClient;
    let env = Env::default();
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let token = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 2u32;
    env.mock_all_auths();
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Testing"),
        &token,
        &100,
        &10,
        &3,
        &reviewers,
        &quorum,
        &None,
    );
    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
    );
    // Advance ledger timestamp by COMMUNITY_REVIEW_PERIOD to allow voting
    const COMMUNITY_REVIEW_PERIOD: u64 = 3 * 24 * 60 * 60;
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);

    // Advance past community review period (3 days)
    env.ledger().set_timestamp(3 * 24 * 60 * 60 + 1);

    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(1).unwrap(), &true, &None);
    // This vote should panic
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(2).unwrap(), &true, &None);
}
#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_milestone_double_voting_panics() {
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, Vec};
    use stellar_grants::StellarGrantsContractClient;
    let env = Env::default();
    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let token = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 2u32;

    // Create grant
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Testing"),
        &token,
        &100,
        &10,
        &3,
        &reviewers,
        &quorum,
        &None,
    );

    // Submit milestone
    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
    );

    // Advance past community review period (3 days)
    env.ledger().set_timestamp(3 * 24 * 60 * 60 + 1);

    // Reviewer 1 votes approve
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);

    // Reviewer 1 tries to vote again (should panic)
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
}
