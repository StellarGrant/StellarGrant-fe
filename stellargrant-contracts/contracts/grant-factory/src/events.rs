use soroban_sdk::{contractevent, Address, Env, String};

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCreated {
    pub grant_id: u64,
    pub instance: Address,
    pub owner: Address,
    pub title: String,
    pub total_amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributorRegistered {
    pub contributor: Address,
    pub name: String,
    pub timestamp: u64,
}

pub struct Events;

impl Events {
    pub fn emit_grant_created(
        env: &Env,
        grant_id: u64,
        instance: Address,
        owner: Address,
        title: String,
        total_amount: i128,
    ) {
        let event = GrantCreated {
            grant_id,
            instance,
            owner,
            title,
            total_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_contributor_registered(env: &Env, contributor: Address, name: String) {
        let event = ContributorRegistered {
            contributor,
            name,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }
}
