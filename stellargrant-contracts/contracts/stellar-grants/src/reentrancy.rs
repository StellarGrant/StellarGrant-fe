//! Reentrancy guard for finance paths that invoke external token contracts.
//!
//! Uses Soroban **temporary** storage as a binary lock (`reentlk`). Temporary keys
//! are cheap and scoped to the current ledger semantics used by the host for this
//! contract invocation, which matches the usual “per–top-level call” guard pattern.
//!
//! Prefer [`with_non_reentrant`] so the lock is always released on the success and
//! error return paths from the wrapped closure.

use crate::ContractError;
use soroban_sdk::{panic_with_error, symbol_short, Env, Symbol};

const REENTRANCY_LOCK_KEY: Symbol = symbol_short!("reentlk");

/// Acquire the reentrancy lock. Panics with [`ContractError::ReentrancyDetected`] if already held.
pub fn lock(env: &Env) {
    if env.storage().temporary().has(&REENTRANCY_LOCK_KEY) {
        panic_with_error!(env, ContractError::ReentrancyDetected);
    }
    env.storage().temporary().set(&REENTRANCY_LOCK_KEY, &true);
}

/// Release the lock (idempotent: removing a missing key is fine).
pub fn unlock(env: &Env) {
    env.storage().temporary().remove(&REENTRANCY_LOCK_KEY);
}

/// Runs `f` while holding the reentrancy lock, then releases it.
pub fn with_non_reentrant<T, F>(env: &Env, f: F) -> Result<T, ContractError>
where
    F: FnOnce() -> Result<T, ContractError>,
{
    lock(env);
    let result = f();
    unlock(env);
    result
}
