use crate::types::ContributorProfile;
use soroban_sdk::{contracttype, Env, BytesN, Address};

#[contracttype]
pub enum DataKey {
    WasmHash,
    GrantCounter,
    Grant(u64),
    Contributor(Address),
}

pub struct Storage;

impl Storage {
    pub fn get_wasm_hash(env: &Env) -> Option<BytesN<32>> {
        env.storage().persistent().get(&DataKey::WasmHash)
    }

    pub fn set_wasm_hash(env: &Env, hash: &BytesN<32>) {
        env.storage().persistent().set(&DataKey::WasmHash, hash);
    }

    pub fn get_grant(env: &Env, grant_id: u64) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Grant(grant_id))
    }

    pub fn set_grant(env: &Env, grant_id: u64, grant_address: &Address) {
        env.storage().persistent().set(&DataKey::Grant(grant_id), grant_address);
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
        contributor: Address,
    ) -> Option<ContributorProfile> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributor(contributor))
    }

    pub fn set_contributor(
        env: &Env,
        contributor: Address,
        profile: &ContributorProfile,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::Contributor(contributor), profile);
    }
}
