use crate::types::ContractError;
use soroban_sdk::Env;

pub fn assert_not_paused(env: &Env) -> Result<(), ContractError> {
    if crate::storage::Storage::get_is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}
