use soroban_sdk::{Address, Env, Symbol};
use crate::storage::Storage;
use crate::types::{ContractError, Role};
use crate::events::Events;

pub struct AccessControl;

impl AccessControl {
    pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
        Storage::has_role(env, address, role)
    }

    pub fn require_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
        if !Self::has_role(env, address, role) {
            return Err(ContractError::Unauthorized);
        }
        Ok(())
    }

    pub fn grant_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
        if Self::has_role(env, address, role.clone()) {
            return Err(ContractError::RoleAlreadyAssigned);
        }
        Storage::set_role(env, address, role.clone());
        Events::emit_role_granted(env, address.clone(), role);
        Ok(())
    }

    pub fn revoke_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
        if !Self::has_role(env, address, role.clone()) {
            return Err(ContractError::RoleNotFound);
        }
        Storage::remove_role(env, address, role.clone());
        Events::emit_role_revoked(env, address.clone(), role);
        Ok(())
    }

    pub fn require_unpaused(env: &Env) -> Result<(), ContractError> {
        if Storage::is_paused(env) {
            return Err(ContractError::Unauthorized); // Or a specific ContractError::Paused
        }
        Ok(())
    }

    pub fn set_paused(env: &Env, address: &Address, paused: bool) -> Result<(), ContractError> {
        if paused {
            Self::require_role(env, address, Role::Pauser)?;
        } else {
            Self::require_role(env, address, Role::Admin)?; // Only Admin can unpause
        }
        Storage::set_paused(env, paused);
        
        let topics = (Symbol::new(env, "PauseStateChanged"), address.clone());
        env.events().publish(topics, paused);
        
        Ok(())
    }
}
