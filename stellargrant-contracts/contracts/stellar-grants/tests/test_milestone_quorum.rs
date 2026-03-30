use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger as _},
    Address, Env, String, Vec,
};
use stellar_grants::{MilestoneState, StellarGrantsContractClient, COMMUNITY_REVIEW_PERIOD};

#[test]
fn test_milestone_voting_quorum_and_events() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let admin = <Address as TestAddress>::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&contract_id, &1000);

    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 2u32;

    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Testing"),
        &token_id,
        &100,
        &10,
        &3,
        &reviewers,
        &quorum,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );

    client.grant_accept(&grant_id, &owner);

    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
        &None,
    );

    // Advance past the community review period so reviewer voting is allowed
    let ts = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(ts.saturating_add(COMMUNITY_REVIEW_PERIOD).saturating_add(1));

    let res1 = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
    assert_eq!(res1, false); // Quorum not reached yet
    let res2 = client.milestone_vote(&grant_id, &0, &reviewers.get(1).unwrap(), &true, &None);
    assert_eq!(res2, true);

    let milestone = client.get_milestone(&grant_id, &0);
    // Awaiting payout after quorum
    assert_eq!(milestone.state, MilestoneState::AwaitingPayout);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + stellar_grants::CHALLENGE_PERIOD + 1);
    client.milestone_payout(&grant_id, &0, &owner);

    let paid_milestone = client.get_milestone(&grant_id, &0);
    assert_eq!(paid_milestone.state, MilestoneState::Paid);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_milestone_vote_after_quorum_panics() {
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, Vec};
    use stellar_grants::{StellarGrantsContractClient, COMMUNITY_REVIEW_PERIOD};
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);
    let owner = <Address as TestAddress>::generate(&env);
    let admin = <Address as TestAddress>::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin.mint(&contract_id, &1000);

    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    reviewers.push_back(<Address as TestAddress>::generate(&env));
    let quorum = 2u32;

    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Test Grant"),
        &String::from_str(&env, "Testing"),
        &token_id,
        &100,
        &10,
        &3,
        &reviewers,
        &quorum,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );
    client.grant_accept(&grant_id, &owner);
    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
        &None,
    );
    let ts = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(ts.saturating_add(COMMUNITY_REVIEW_PERIOD).saturating_add(1));
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(1).unwrap(), &true, &None);
    // This vote should panic (milestone already approved — MilestoneNotSubmitted #7)
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(2).unwrap(), &true, &None);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_milestone_double_voting_panics() {
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, Vec};
    use stellar_grants::{StellarGrantsContractClient, COMMUNITY_REVIEW_PERIOD};
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
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
    );

    client.grant_accept(&grant_id, &owner);

    let _ = client.milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(&env, "desc"),
        &String::from_str(&env, "proof"),
        &None,
    );

    let ts = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(ts.saturating_add(COMMUNITY_REVIEW_PERIOD).saturating_add(1));

    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
    let _ = client.milestone_vote(&grant_id, &0, &reviewers.get(0).unwrap(), &true, &None);
}
