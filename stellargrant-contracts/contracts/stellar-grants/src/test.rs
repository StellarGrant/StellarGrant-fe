#![cfg(test)]

use crate::{
    ContractError, StellarGrantsContract, StellarGrantsContractClient, HEARTBEAT_CANCEL_SECS,
    HEARTBEAT_INACTIVE_SECS,
};
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger},
    token, Address, Env, String, Vec,
};

#[test]
fn test_heartbeat_miss() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarGrantsContract);
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

    // Create grant
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Heartbeat"),
        &String::from_str(&env, "Testing heartbeat"),
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
        &false,
        &false,
    );

    let funder = <Address as TestAddress>::generate(&env);
    client.grant_accept(&grant_id, &owner);
    token_admin.mint(&funder, &100);
    client.grant_fund(&grant_id, &funder, &100, &token, &None);

    let mut now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + 10);

    // Ping should succeed
    client.grant_ping(&grant_id, &owner);

    // Advance time to exactly inactive boundary
    now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + HEARTBEAT_INACTIVE_SECS);

    // Attempting to mark inactive should fail because it's not strictly greater
    let res_stale = client.try_mark_grant_inactive(&grant_id);
    assert!(
        res_stale.is_err(),
        "Should not be able to mark inactive yet"
    );

    // Advance time by 30 days + 1 sec
    env.ledger()
        .set_timestamp(now + HEARTBEAT_INACTIVE_SECS + 1);

    // Call mark_grant_inactive, should succeed
    client.mark_grant_inactive(&grant_id);

    // Advance 60 days
    env.ledger().set_timestamp(now + HEARTBEAT_CANCEL_SECS + 1);

    // Anyone can cancel
    let random_caller = <Address as TestAddress>::generate(&env);
    client.cancel_grant(&grant_id, &random_caller, &String::from_str(&env, "stale"));
}

#[test]
fn test_public_good_cancellation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarGrantsContract);
    let client = StellarGrantsContractClient::new(&env, &contract_id);

    let admin = <Address as TestAddress>::generate(&env);
    let council = <Address as TestAddress>::generate(&env);
    let treasury = <Address as TestAddress>::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin, &council, &treasury);

    let owner = <Address as TestAddress>::generate(&env);
    let token_admin_addr = <Address as TestAddress>::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin_addr.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token);

    let mut reviewers = Vec::new(&env);
    reviewers.push_back(<Address as TestAddress>::generate(&env));

    // Create public good grant
    let grant_id = client.grant_create(
        &owner,
        &String::from_str(&env, "Public Good"),
        &String::from_str(&env, "Testing public good"),
        &token,
        &100,
        &10,
        &1,
        &reviewers,
        &1,
        &None,
        &0i128,
        &0i128,
        &soroban_sdk::Vec::<soroban_sdk::String>::new(&env),
        &false,
        &true, // is_public_good = true
    );

    let funder = <Address as TestAddress>::generate(&env);
    client.grant_accept(&grant_id, &owner);
    token_admin.mint(&funder, &100);
    client.grant_fund(&grant_id, &funder, &100, &token, &None);

    // Cancel grant
    client.cancel_grant(
        &grant_id,
        &owner,
        &String::from_str(&env, "donating to treasury"),
    );

    // Check balance of treasury
    let treasury_balance = token::Client::new(&env, &token).balance(&treasury);
    assert_eq!(treasury_balance, 100);

    // Check balance of funder (should be 0 because it went to treasury)
    let funder_balance = token::Client::new(&env, &token).balance(&funder);
    assert_eq!(funder_balance, 0);
}
