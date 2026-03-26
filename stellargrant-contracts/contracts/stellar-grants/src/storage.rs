use crate::types::{Grant, Milestone};
use soroban_sdk::{contracttype, Env};

#[contracttype]
pub enum DataKey {
    Grant(u64),
    Milestone(u64, u32),
    GrantCounter,
    Contributor(soroban_sdk::Address),
    ExtensionRequest(u64, u32),
}

pub struct Storage;

impl Storage {
    pub fn get_grant(env: &Env, grant_id: u64) -> Option<Grant> {
        env.storage().persistent().get(&DataKey::Grant(grant_id))
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

    pub fn set_milestone(env: &Env, grant_id: u64, milestone_idx: u32, milestone: &Milestone) {
        env.storage()
            .persistent()
            .set(&DataKey::Milestone(grant_id, milestone_idx), milestone);
    }

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

    pub fn get_contributor(
        env: &Env,
        contributor: soroban_sdk::Address,
    ) -> Option<crate::types::ContributorProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributor(contributor))
    }

    pub fn set_contributor(
        env: &Env,
        contributor: soroban_sdk::Address,
        profile: &crate::types::ContributorProfile,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::Contributor(contributor), profile);
    }

    pub fn get_extension_request(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Option<crate::types::ExtensionRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::ExtensionRequest(grant_id, milestone_idx))
    }

    pub fn set_extension_request(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        request: &crate::types::ExtensionRequest,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::ExtensionRequest(grant_id, milestone_idx), request);
    }
}
