/// Tests for Issue #151 (reputation auto-scaling) and Issue #152 (dispute fee).
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};
use stellar_grants::{
    MilestoneState, StellarGrantsContractClient, CHALLENGE_PERIOD, COMMUNITY_REVIEW_PERIOD,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_env_client_token() -> (
    Env,
    StellarGrantsContractClient<'static>,
    Address, // admin
    Address, // council
    Address, // owner
    Address, // reviewer
    Address, // funder
    Address, // token
    token::StellarAssetClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let council = Address::generate(&env);
    let owner = Address::generate(&env);
    let reviewer = Address::generate(&env);
    let funder = Address::generate(&env);
    let tok_adm = Address::generate(&env);
    let tok = env.register_stellar_asset_contract_v2(tok_adm).address();

    let cid = env.register_contract(None, stellar_grants::StellarGrantsContract);
    let env_ref: &'static Env = unsafe { &*(&env as *const Env) };
    let client = StellarGrantsContractClient::new(env_ref, &cid);
    let tok_client = token::StellarAssetClient::new(env_ref, &tok);

    client.initialize(&admin, &council);
    (
        env, client, admin, council, owner, reviewer, funder, tok, tok_client,
    )
}

fn create_funded_submitted_voted(
    env: &Env,
    client: &StellarGrantsContractClient,
    owner: &Address,
    reviewer: &Address,
    funder: &Address,
    token: &Address,
    tok_admin: &token::StellarAssetClient,
) -> u64 {
    let mut revs = Vec::new(env);
    revs.push_back(reviewer.clone());

    let gid = client.grant_create(
        owner,
        &String::from_str(env, "G"),
        &String::from_str(env, "D"),
        token,
        &1000,
        &1000,
        &1,
        &revs,
        &1,
        &None,
        &0i128,
        &0i128,
        &Vec::<String>::new(env),
    );
    client.grant_accept(&gid, owner);
    tok_admin.mint(funder, &2000);
    client.grant_fund(&gid, funder, &1000, token, &None);
    tok_admin.mint(reviewer, &1);
    client.stake_to_review(reviewer, &gid, &1);
    client.milestone_submit(
        &gid,
        &0,
        owner,
        &String::from_str(env, "MS"),
        &String::from_str(env, "proof"),
        &None,
    );
    let now = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(now + COMMUNITY_REVIEW_PERIOD + 1);
    client.milestone_vote(&gid, &0, reviewer, &true, &None);
    gid
}

fn payout_voted_milestone(
    env: &Env,
    client: &StellarGrantsContractClient,
    gid: u64,
    owner: &Address,
) {
    let now = env.ledger().timestamp();
    env.ledger().set_timestamp(now + CHALLENGE_PERIOD + 1);
    client.milestone_payout(&gid, &0, owner);
}

// ── Issue #151 — Reputation auto-scaling ─────────────────────────────────────

#[test]
fn test_reputation_increases_after_milestone_approve() {
    let (env, client, _admin, _council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    client.contributor_register(
        &owner,
        &String::from_str(&env, "Alice"),
        &String::from_str(&env, "Bio"),
        &Vec::<String>::new(&env),
        &String::from_str(&env, "https://github.com/alice"),
    );

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);

    let profile_before = client.get_contributor_profile(&owner).unwrap();
    let rep_before = profile_before.reputation_score;
    let earned_before = profile_before.total_earned;

    payout_voted_milestone(&env, &client, gid, &owner);

    let profile_after = client.get_contributor_profile(&owner).unwrap();
    assert_eq!(
        profile_after.reputation_score,
        rep_before + 10,
        "reputation_score must increase by 10 after a successful payout"
    );
    assert_eq!(
        profile_after.total_earned,
        earned_before + 1000,
        "total_earned must increase by the milestone payout amount"
    );
}

#[test]
fn test_reputation_idempotent_per_milestone() {
    let (env, client, _admin, _council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    client.contributor_register(
        &owner,
        &String::from_str(&env, "Bob"),
        &String::from_str(&env, "Bio"),
        &Vec::<String>::new(&env),
        &String::from_str(&env, "https://github.com/bob"),
    );

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);
    payout_voted_milestone(&env, &client, gid, &owner);

    let rep_after_first = client
        .get_contributor_profile(&owner)
        .unwrap()
        .reputation_score;

    let result = client.try_milestone_payout(&gid, &0, &owner);
    assert!(result.is_err(), "second approve must fail");

    let rep_unchanged = client
        .get_contributor_profile(&owner)
        .unwrap()
        .reputation_score;
    assert_eq!(
        rep_after_first, rep_unchanged,
        "reputation must not double-count"
    );
}

#[test]
fn test_reputation_skipped_gracefully_without_profile() {
    let (env, client, _admin, _council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);
    payout_voted_milestone(&env, &client, gid, &owner);
}

// ── Issue #152 — Dispute fee ──────────────────────────────────────────────────

#[test]
fn test_zero_fee_dispute_requires_no_transfer() {
    let (env, client, _admin, _council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();
    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);

    client.dispute_milestone(&gid, &0, &owner);

    let m = client.get_milestone(&gid, &0);
    assert_eq!(m.state(), MilestoneState::Disputed);
}

#[test]
fn test_dispute_fee_deducted_from_caller() {
    let (env, client, admin, _council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    client.set_dispute_fee(&admin, &50i128);

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);

    tok_adm.mint(&owner, &100);

    let tok_client = token::Client::new(&env, &tok);
    let bal_before = tok_client.balance(&owner);

    client.dispute_milestone(&gid, &0, &owner);

    let bal_after = tok_client.balance(&owner);
    assert_eq!(
        bal_before - bal_after,
        50,
        "dispute fee (50) must be deducted from the caller"
    );
}

#[test]
fn test_dispute_fee_refunded_when_upheld() {
    let (env, client, admin, council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    client.set_dispute_fee(&admin, &50i128);

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);
    tok_adm.mint(&owner, &100);

    client.dispute_milestone(&gid, &0, &owner);

    let tok_client = token::Client::new(&env, &tok);
    let bal_before_resolve = tok_client.balance(&owner);

    client.resolve_dispute(&council, &gid, &0, &true);

    let bal_after_resolve = tok_client.balance(&owner);
    assert_eq!(
        bal_after_resolve - bal_before_resolve,
        1050,
        "resolve_dispute(true) pays the milestone amount and refunds the dispute fee"
    );
}

#[test]
fn test_dispute_fee_sent_to_treasury_when_dismissed() {
    let (env, client, admin, council, owner, reviewer, funder, tok, tok_adm) =
        make_env_client_token();

    let treasury = Address::generate(&env);
    client.set_staking_config(&admin, &1i128, &treasury);

    client.set_dispute_fee(&admin, &50i128);

    let gid =
        create_funded_submitted_voted(&env, &client, &owner, &reviewer, &funder, &tok, &tok_adm);
    tok_adm.mint(&owner, &100);

    client.dispute_milestone(&gid, &0, &owner);

    let tok_client = token::Client::new(&env, &tok);
    let treasury_before = tok_client.balance(&treasury);

    client.resolve_dispute(&council, &gid, &0, &false);

    let treasury_after = tok_client.balance(&treasury);
    assert_eq!(
        treasury_after - treasury_before,
        50,
        "dispute fee must be sent to treasury when dispute is dismissed"
    );
}
