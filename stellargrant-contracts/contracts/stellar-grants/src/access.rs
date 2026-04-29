use crate::types::{AccessControl, ContractError, Role};
use crate::Storage;
use soroban_sdk::{Address, Env};

pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
    Storage::get_access_control(env, address).has_role(role)
}

pub fn role_guard_enabled(env: &Env, role: Role) -> bool {
    Storage::get_role_member_count(env, role) > 0
}

pub fn has_role_or_admin(env: &Env, address: &Address, role: Role) -> bool {
    has_role(env, address, Role::Admin) || has_role(env, address, role)
}

pub fn require_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
    if has_role_or_admin(env, address, role) {
        return Ok(());
    }
    Err(ContractError::Unauthorized)
}

pub fn require_optional_role(
    env: &Env,
    address: &Address,
    role: Role,
) -> Result<(), ContractError> {
    if !role_guard_enabled(env, role) {
        return Ok(());
    }
    require_role(env, address, role)
}

pub fn grant_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
    let mut access_control = Storage::get_access_control(env, address);
    if access_control.has_role(role) {
        return Err(ContractError::RoleAlreadyAssigned);
    }

    access_control.grant(role);
    Storage::set_access_control(env, address, &access_control);

    let next_count = Storage::get_role_member_count(env, role).saturating_add(1);
    Storage::set_role_member_count(env, role, next_count);
    Ok(())
}

pub fn revoke_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
    let mut access_control = Storage::get_access_control(env, address);
    if !access_control.has_role(role) {
        return Err(ContractError::RoleNotAssigned);
    }

    access_control.revoke(role);
    persist_access_control(env, address, &access_control);

    let next_count = Storage::get_role_member_count(env, role).saturating_sub(1);
    Storage::set_role_member_count(env, role, next_count);
    Ok(())
}

pub fn renounce_role(env: &Env, address: &Address, role: Role) -> Result<(), ContractError> {
    revoke_role(env, address, role)
}

fn persist_access_control(env: &Env, address: &Address, access_control: &AccessControl) {
    if access_control.role_flags == 0 {
        Storage::remove_access_control(env, address);
        return;
    }
    Storage::set_access_control(env, address, access_control);
}
