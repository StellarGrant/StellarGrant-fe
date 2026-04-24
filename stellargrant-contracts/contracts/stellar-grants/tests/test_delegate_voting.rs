use soroban_sdk::{
    testutils::{Address as TestAddress, Events, Ledger as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String, Vec,
};
use stellar_grants::{
    ContractError, StellarGrantsContract, StellarGrantsContractClient, Storage,
    COMMUNITY_REVIEW_PERIOD,
};

fn setup_active_grant<'a>(
    env: &'a Env,
    reviewers: &Vec<Address>,
) -> (StellarGrantsContractClient<'a>, Address, u64) {
    let contract_id = env.register(StellarGrantsContract, ());
    let client = StellarGrantsContractClient::new(env, &contract_id);
    let owner = <Address as TestAddress>::generate(env);
    let admin = <Address as TestAddress>::generate(env);

    let token_contract = env.register_stellar_asset_contract_v2(admin);
    let token_id = token_contract.address();
    let quorum = (reviewers.len() / 2) + 1;
    let title = String::from_str(env, "Delegation Test");
    let description = String::from_str(env, "Reviewer delegation");
    let milestone_deadlines: Option<Vec<u64>> = None;
    let tags = Vec::<String>::new(env);

    let grant_id = client.mock_all_auths().grant_create(
        &owner,
        &title,
        &description,
        &token_id,
        &100,
        &10,
        &1,
        reviewers,
        &quorum,
        &milestone_deadlines,
        &0i128,
        &0i128,
        &tags,
        &false,
    );
    client.mock_all_auths().grant_accept(&grant_id, &owner);
    client.mock_all_auths().milestone_submit(
        &grant_id,
        &0,
        &owner,
        &String::from_str(env, "Milestone"),
        &String::from_str(env, "proof"),
        &None,
    );

    let ts = env.ledger().timestamp();
    env.ledger()
        .set_timestamp(ts.saturating_add(COMMUNITY_REVIEW_PERIOD).saturating_add(1));

    (client, contract_id, grant_id)
}

#[test]
fn test_delegatee_vote_counts_for_reviewer_and_emits_event() {
    let env = Env::default();
    let reviewer = <Address as TestAddress>::generate(&env);
    let delegatee = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(reviewer.clone());

    let (client, contract_id, grant_id) = setup_active_grant(&env, &reviewers);
    let feedback: Option<String> = None;

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &delegatee, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &delegatee, &grant_id);

    let mut found_delegate_event = false;
    extern crate std;
    for event in env.events().all().events() {
        let rendered = std::format!("{:?}", event);
        if rendered.contains("reviewer_delegated") {
            found_delegate_event = true;
        }
    }
    assert!(
        found_delegate_event,
        "ReviewerDelegated event was not emitted"
    );

    let approved = client
        .mock_auths(&[MockAuth {
            address: &delegatee,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "milestone_vote",
                args: (&grant_id, &0u32, &reviewer, &true, &feedback, &None::<u32>).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .milestone_vote(&grant_id, &0, &reviewer, &true, &feedback, &None);
    assert!(approved);

    env.as_contract(&contract_id, || {
        let milestone = Storage::get_milestone(&env, grant_id, 0).unwrap();
        assert_eq!(milestone.votes.get(reviewer.clone()), Some(true));
        assert_eq!(milestone.votes.get(delegatee.clone()), None);
    });
}

#[test]
fn test_delegate_update_and_revocation_work_for_same_grant() {
    let env = Env::default();
    let reviewer = <Address as TestAddress>::generate(&env);
    let first_delegatee = <Address as TestAddress>::generate(&env);
    let second_delegatee = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(reviewer.clone());

    let (client, contract_id, grant_id) = setup_active_grant(&env, &reviewers);
    let feedback: Option<String> = None;

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &first_delegatee, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &first_delegatee, &grant_id);

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &second_delegatee, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &second_delegatee, &grant_id);

    env.as_contract(&contract_id, || {
        assert_eq!(
            Storage::get_delegation(&env, grant_id, &reviewer),
            Some(second_delegatee.clone())
        );
    });

    let approved = client
        .mock_auths(&[MockAuth {
            address: &second_delegatee,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "milestone_vote",
                args: (&grant_id, &0u32, &reviewer, &true, &feedback, &None::<u32>).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .milestone_vote(&grant_id, &0, &reviewer, &true, &feedback, &None);
    assert!(approved);

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &reviewer, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &reviewer, &grant_id);

    env.as_contract(&contract_id, || {
        assert_eq!(Storage::get_delegation(&env, grant_id, &reviewer), None);
    });
}

#[test]
fn test_delegated_vote_uses_reviewer_slot_for_double_vote_protection() {
    let env = Env::default();
    let reviewer = <Address as TestAddress>::generate(&env);
    let other_reviewer = <Address as TestAddress>::generate(&env);
    let delegatee = <Address as TestAddress>::generate(&env);
    let mut reviewers = Vec::new(&env);
    reviewers.push_back(reviewer.clone());
    reviewers.push_back(other_reviewer);

    let (client, contract_id, grant_id) = setup_active_grant(&env, &reviewers);
    let feedback: Option<String> = None;

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &delegatee, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &delegatee, &grant_id);

    let delegated_vote = client
        .mock_auths(&[MockAuth {
            address: &delegatee,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "milestone_vote",
                args: (&grant_id, &0u32, &reviewer, &true, &feedback, &None::<u32>).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .milestone_vote(&grant_id, &0, &reviewer, &true, &feedback, &None);
    assert!(
        !delegated_vote,
        "single delegated vote should not reach quorum"
    );

    client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "grant_delegate",
                args: (&reviewer, &reviewer, &grant_id).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .grant_delegate(&reviewer, &reviewer, &grant_id);

    let second_vote = client
        .mock_auths(&[MockAuth {
            address: &reviewer,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "milestone_vote",
                args: (&grant_id, &0u32, &reviewer, &true, &feedback, &None::<u32>).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_milestone_vote(&grant_id, &0, &reviewer, &true, &feedback, &None);
    assert_eq!(second_vote, Err(Ok(ContractError::AlreadyVoted.into())));
}
