use crate::types::MilestoneState;
use soroban_sdk::{contractevent, Address, Env, String};

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneVoted {
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub reviewer: Address,
    pub approve: bool,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneRejected {
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub reviewer: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneStatusChanged {
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub new_state: MilestoneState,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCancelled {
    pub grant_id: u64,
    pub owner: Address,
    pub reason: String,
    pub refund_amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefundExecuted {
    pub grant_id: u64,
    pub funder: Address,
    pub amount: i128,
}

pub struct Events;

impl Events {
    pub fn emit_grant_cancelled(
        env: &Env,
        grant_id: u64,
        owner: Address,
        reason: String,
        refund_amount: i128,
    ) {
        let event = GrantCancelled {
            grant_id,
            owner,
            reason,
            refund_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_refund_executed(env: &Env, grant_id: u64, funder: Address, amount: i128) {
        let event = RefundExecuted {
            grant_id,
            funder,
            amount,
        };
        event.publish(env);
    }

    pub fn milestone_voted(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approve: bool,
    ) {
        let event = MilestoneVoted {
            grant_id,
            milestone_idx,
            reviewer,
            approve,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_rejected(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        reason: String,
    ) {
        let event = MilestoneRejected {
            grant_id,
            milestone_idx,
            reviewer,
            reason,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_status_changed(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        new_state: MilestoneState,
    ) {
        let event = MilestoneStatusChanged {
            grant_id,
            milestone_idx,
            new_state,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }
}
