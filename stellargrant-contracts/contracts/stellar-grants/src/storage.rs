use crate::types::{ContractError, Grant, Milestone};
use soroban_sdk::{contracttype, Env};

#[contracttype]
pub enum DataKey {
    Grant(u64),
    Milestone(u64, u32),
}

pub struct Storage;

impl Storage {
    pub fn get_grant(env: &Env, grant_id: u64) -> Option<Grant> {
        env.storage().persistent().get(&DataKey::Grant(grant_id))
    }

    pub fn get_grant_v(env: &Env, grant_id: u64) -> Grant {
        Self::get_grant(env, grant_id).unwrap_or_else(|| {
            env.panic_with_error(ContractError::GrantNotFound);
        })
    }

    pub fn set_grant(env: &Env, grant_id: u64, grant: &Grant) {
        env.storage()
            .persistent()
            .set(&DataKey::Grant(grant_id), grant);
    }

    pub fn has_grant(env: &Env, grant_id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Grant(grant_id))
    }

    pub fn get_milestone(env: &Env, grant_id: u64, milestone_idx: u32) -> Option<Milestone> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(grant_id, milestone_idx))
    }

    pub fn get_milestone_v(env: &Env, grant_id: u64, milestone_idx: u32) -> Milestone {
        Self::get_milestone(env, grant_id, milestone_idx).unwrap_or_else(|| {
            env.panic_with_error(ContractError::MilestoneNotFound);
        })
    }

    pub fn set_milestone(env: &Env, grant_id: u64, milestone_idx: u32, milestone: &Milestone) {
        env.storage()
            .persistent()
            .set(&DataKey::Milestone(grant_id, milestone_idx), milestone);
    }
<<<<<<< HEAD
=======

    pub fn increment_grant_counter(env: &Env) -> u64 {
        let mut counter: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::GrantCounter)
            .unwrap_or(0);
        counter += 1;
        env.storage()
            .persistent()
            .set(&DataKey::GrantCounter, &counter);
        counter
    }
>>>>>>> 8a269c2 (chore: resolve upstream merge conflicts and update test schemas)
}
