use crate::types::{Grant, Milestone};
use soroban_sdk::{contracttype, Env};

#[contracttype]
pub enum DataKey {
    Grant,
    Milestone(u32),
}

pub struct Storage;

impl Storage {
    pub fn get_grant(env: &Env) -> Option<Grant> {
        env.storage().persistent().get(&DataKey::Grant)
    }

    pub fn set_grant(env: &Env, grant: &Grant) {
        env.storage().persistent().set(&DataKey::Grant, grant);
    }

    pub fn get_milestone(env: &Env, milestone_idx: u32) -> Option<Milestone> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_idx))
    }

    pub fn set_milestone(env: &Env, milestone_idx: u32, milestone: &Milestone) {
        env.storage()
            .persistent()
            .set(&DataKey::Milestone(milestone_idx), milestone);
    }
}
