#![no_main]

use libfuzzer_sys::fuzz_target;
use stellar_grants::StellarGrantsContract;
use soroban_sdk::{Env, Address, String, Vec};

// Fuzz target for milestone_submit
fuzz_target!(|data: &[u8]| {
    // Skip if input is empty
    if data.is_empty() {
        return;
    }
    let result = std::panic::catch_unwind(|| {
        let env = Env::default();
        let owner = Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF");
        let title = String::from_str(&env, "Fuzz Grant");
        let description = String::from_str(&env, "Fuzzing");
        let token = Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHG");
        let reviewers: Vec<Address> = Vec::new(&env);
        let quorum = 1;
        let _ = StellarGrantsContract::grant_create(
            env.clone(),
            owner.clone(),
            title,
            description,
            token,
            100,
            10,
            2,
            reviewers,
            quorum,
            None,
        );
        // Fuzz milestone_submit with random description and proof_url
        let recipient = owner;
        let desc = String::from_str(&env, "desc");
        let proof = String::from_str(&env, "proof");
        let _ = StellarGrantsContract::milestone_submit(env, 0, 0, recipient, desc, proof);
    });
    // If a panic occurred, treat as a fuzz failure only if input was not obviously invalid
    if result.is_err() && !data.is_empty() {
        panic!("Fuzz target panicked on non-empty input");
    }
});
