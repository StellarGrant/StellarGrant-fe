#![no_std]
#![allow(clippy::too_many_arguments)]
mod events;
mod storage;
mod types;

pub use events::Events;
pub use storage::Storage;
pub use types::{ContractError, ContributorProfile};

use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Symbol, Val, Vec, IntoVal
};

#[contract]
pub struct StellarGrantsFactory;

#[contractimpl]
impl StellarGrantsFactory {
    /// Initialize the factory with the WASM hash of the GrantInstance contract
    pub fn initialize(env: Env, wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        if Storage::get_wasm_hash(&env).is_some() {
            return Err(ContractError::AlreadyInitialized);
        }
        Storage::set_wasm_hash(&env, &wasm_hash);
        Ok(())
    }

    /// Allows a grant developer/owner to create a new milestone-based grant.
    pub fn create_grant(
        env: Env,
        owner: Address,
        title: String,
        description: String,
        token: Address,
        total_amount: i128,
        milestone_amount: i128,
        num_milestones: u32,
        reviewers: Vec<Address>,
    ) -> Result<Address, ContractError> {
        owner.require_auth();

        let wasm_hash = Storage::get_wasm_hash(&env).ok_or(ContractError::NotInitialized)?;

        let grant_id = Storage::increment_grant_counter(&env);

        // Generate a salt based on grant_id
        let mut salt_bytes = [0u8; 32];
        let id_bytes = grant_id.to_be_bytes();
        salt_bytes[24..32].copy_from_slice(&id_bytes);
        let salt = BytesN::from_array(&env, &salt_bytes);

        // Deploy the grant instance
        let grant_instance_addr = env
            .deployer()
            .with_current_contract(salt)
            .deploy(wasm_hash);

        // Initialize the new grant instance
        let init_fn = Symbol::new(&env, "initialize");
        let mut init_args = Vec::new(&env);
        init_args.push_back(owner.into_val(&env));
        init_args.push_back(title.into_val(&env));
        init_args.push_back(description.into_val(&env));
        init_args.push_back(token.into_val(&env));
        init_args.push_back(total_amount.into_val(&env));
        init_args.push_back(milestone_amount.into_val(&env));
        init_args.push_back(num_milestones.into_val(&env));
        init_args.push_back(reviewers.into_val(&env));

        let _ = env.invoke_contract::<Val>(&grant_instance_addr, &init_fn, init_args);

        // Store the grant address in factory mapping
        Storage::set_grant(&env, grant_id, &grant_instance_addr);

        Events::emit_grant_created(&env, grant_id, grant_instance_addr.clone(), owner, title, total_amount);

        Ok(grant_instance_addr)
    }

    /// Register a contributor profile on-chain
    pub fn contributor_register(
        env: Env,
        contributor: Address,
        name: String,
        bio: String,
        skills: Vec<String>,
        github_url: String,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        if name.is_empty() || name.len() > 100 {
            return Err(ContractError::InvalidInput);
        }
        if bio.len() > 500 {
            return Err(ContractError::InvalidInput);
        }

        if Storage::get_contributor(&env, contributor.clone()).is_some() {
            return Err(ContractError::AlreadyRegistered);
        }

        let profile = ContributorProfile {
            contributor: contributor.clone(),
            name: name.clone(),
            bio,
            skills,
            github_url,
            registration_timestamp: env.ledger().timestamp(),
            grants_count: 0,
            total_earned: 0,
        };

        Storage::set_contributor(&env, contributor.clone(), &profile);

        Events::emit_contributor_registered(&env, contributor, name);

        Ok(())
    }

    /// Fetch a grant's address by its ID
    pub fn get_grant(env: Env, grant_id: u64) -> Option<Address> {
        Storage::get_grant(&env, grant_id)
    }

    /// Helper view to get current amount of grants
    pub fn get_total_grants(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&crate::storage::DataKey::GrantCounter)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
