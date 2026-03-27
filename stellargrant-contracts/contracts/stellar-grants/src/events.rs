use crate::types::MilestoneState;
use soroban_sdk::{symbol_short, Address, Env, String};

pub fn milestone_voted(
    env: &Env,
    grant_id: u64,
    milestone_idx: u32,
    reviewer: Address,
    approve: bool,
) {
    let topics = (symbol_short!("voted"), grant_id, milestone_idx);
    let data = (reviewer, approve, env.ledger().timestamp());
    #[allow(deprecated)]
    env.events().publish(topics, data);
}

pub fn milestone_rejected(
    env: &Env,
    grant_id: u64,
    milestone_idx: u32,
    reviewer: Address,
    reason: String,
) {
    let topics = (symbol_short!("rejected"), grant_id, milestone_idx);
    let data = (reviewer, reason, env.ledger().timestamp());
    #[allow(deprecated)]
    env.events().publish(topics, data);
}

pub fn milestone_status_changed(
    env: &Env,
    grant_id: u64,
    milestone_idx: u32,
    new_state: MilestoneState,
) {
    let topics = (symbol_short!("status"), grant_id, milestone_idx);
    let data = (new_state, env.ledger().timestamp());
    #[allow(deprecated)]
    env.events().publish(topics, data);
}
