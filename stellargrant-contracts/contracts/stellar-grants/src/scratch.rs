use soroban_sdk::{Env, String, Symbol, TryFromVal};

pub fn test_conv(env: Env, s: String) -> Symbol {
    Symbol::try_from_val(&env, &s).unwrap()
}
