use crate::ContractError;
use soroban_sdk::{panic_with_error, symbol_short, Env, Symbol};

const REENTRANCY_LOCK_KEY: Symbol = symbol_short!("reentlk");

pub fn with_non_reentrant<T, F>(env: &Env, f: F) -> Result<T, ContractError>
where
    F: FnOnce() -> Result<T, ContractError>,
{
    if env.storage().temporary().has(&REENTRANCY_LOCK_KEY) {
        panic_with_error!(env, ContractError::ReentrancyDetected);
    }

    env.storage().temporary().set(&REENTRANCY_LOCK_KEY, &true);
    let result = f();
    env.storage().temporary().remove(&REENTRANCY_LOCK_KEY);
    result
}
