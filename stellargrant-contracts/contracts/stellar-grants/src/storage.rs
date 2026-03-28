use crate::types::{EscrowLifecycleState, EscrowMode, EscrowState, Grant, Milestone};
use soroban_sdk::{contracttype, Env};

#[contracttype]
pub enum DataKey {
    Grant(u64),
    Milestone(u64, u32),
    GrantCounter,
    Contributor(soroban_sdk::Address),
    /// Reviewer stake amount for a grant: (grant_id, reviewer) -> i128
    ReviewerStake(u64, soroban_sdk::Address),
    /// Minimum stake required to review a grant
    MinReviewerStake,
    /// Treasury address for slashed stakes
    Treasury,
    /// Identity oracle contract address for KYC verification
    IdentityOracle,
    ReviewerReputation(soroban_sdk::Address),
    GlobalAdmin,
    Council,
    EscrowState(u64),
    MultisigSigners(u64),
    ReleaseSignerApproval(u64, soroban_sdk::Address),
    GrantMinReputation(u64),
    /// Tracks whether a voter has already upvoted a specific milestone.
    MilestoneUpvoter(u64, u32, soroban_sdk::Address),
}

pub struct Storage;

// Soroban TTL values are expressed in ledgers. At roughly 5 seconds per ledger,
// this refreshes entries when they have less than about 6 days left and extends
// them out to roughly 58 days, which keeps long-lived grants active without
// needing constant writes.
const PERSISTENT_TTL_THRESHOLD: u32 = 100_000;
const PERSISTENT_TTL_EXTEND_TO: u32 = 1_000_000;

impl Storage {
    fn bump_persistent_ttl(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
    }

    // --- Staking helpers ---

    pub fn get_reviewer_stake(env: &Env, grant_id: u64, reviewer: &soroban_sdk::Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::ReviewerStake(grant_id, reviewer.clone()))
            .unwrap_or(0)
    }

    pub fn set_reviewer_stake(
        env: &Env,
        grant_id: u64,
        reviewer: &soroban_sdk::Address,
        amount: i128,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::ReviewerStake(grant_id, reviewer.clone()), &amount);
    }

    pub fn get_min_reviewer_stake(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::MinReviewerStake)
            .unwrap_or(0)
    }

    pub fn get_treasury(env: &Env) -> Option<soroban_sdk::Address> {
        env.storage().persistent().get(&DataKey::Treasury)
    }

    pub fn get_identity_oracle(env: &Env) -> Option<soroban_sdk::Address> {
        env.storage().persistent().get(&DataKey::IdentityOracle)
    }

    pub fn get_global_admin(env: &Env) -> Option<soroban_sdk::Address> {
        env.storage().persistent().get(&DataKey::GlobalAdmin)
    }

    pub fn set_global_admin(env: &Env, admin: &soroban_sdk::Address) {
        env.storage().persistent().set(&DataKey::GlobalAdmin, admin);
    }

    pub fn get_council(env: &Env) -> Option<soroban_sdk::Address> {
        env.storage().persistent().get(&DataKey::Council)
    }

    pub fn set_council(env: &Env, council: &soroban_sdk::Address) {
        env.storage().persistent().set(&DataKey::Council, council);
    }

    pub fn get_grant(env: &Env, grant_id: u64) -> Option<Grant> {
        let key = DataKey::Grant(grant_id);
        let grant = env.storage().persistent().get(&key);
        if grant.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        grant
    }

    pub fn set_grant(env: &Env, grant_id: u64, grant: &Grant) {
        let key = DataKey::Grant(grant_id);
        env.storage().persistent().set(&key, grant);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn has_grant(env: &Env, grant_id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Grant(grant_id))
    }

    pub fn get_milestone(env: &Env, grant_id: u64, milestone_idx: u32) -> Option<Milestone> {
        let key = DataKey::Milestone(grant_id, milestone_idx);
        let milestone = env.storage().persistent().get(&key);
        if milestone.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        milestone
    }

    pub fn set_milestone(env: &Env, grant_id: u64, milestone_idx: u32, milestone: &Milestone) {
        let key = DataKey::Milestone(grant_id, milestone_idx);
        env.storage().persistent().set(&key, milestone);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn increment_grant_counter(env: &Env) -> u64 {
        let key = DataKey::GrantCounter;
        let mut counter: u64 = env.storage().persistent().get(&key).unwrap_or(0);
        counter += 1;
        env.storage().persistent().set(&key, &counter);
        Self::bump_persistent_ttl(env, &key);
        counter
    }

    pub fn get_contributor(
        env: &Env,
        contributor: soroban_sdk::Address,
    ) -> Option<crate::types::ContributorProfile> {
        let key = DataKey::Contributor(contributor);
        let profile = env.storage().persistent().get(&key);
        if profile.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        profile
    }

    pub fn set_contributor(
        env: &Env,
        contributor: soroban_sdk::Address,
        profile: &crate::types::ContributorProfile,
    ) {
        let key = DataKey::Contributor(contributor);
        env.storage().persistent().set(&key, profile);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn get_reviewer_reputation(env: &Env, reviewer: soroban_sdk::Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ReviewerReputation(reviewer))
            .unwrap_or(1) // Default basic reputation
    }

    pub fn set_reviewer_reputation(env: &Env, reviewer: soroban_sdk::Address, score: u32) {
        env.storage()
            .persistent()
            .set(&DataKey::ReviewerReputation(reviewer), &score);
    }

    pub fn get_escrow_state(env: &Env, grant_id: u64) -> EscrowState {
        env.storage()
            .persistent()
            .get(&DataKey::EscrowState(grant_id))
            .unwrap_or(EscrowState {
                mode: EscrowMode::Standard,
                lifecycle: EscrowLifecycleState::Funding,
                quorum_ready: false,
                approvals_count: 0,
            })
    }

    pub fn set_escrow_state(env: &Env, grant_id: u64, state: &EscrowState) {
        env.storage()
            .persistent()
            .set(&DataKey::EscrowState(grant_id), state);
    }

    pub fn get_multisig_signers(
        env: &Env,
        grant_id: u64,
    ) -> soroban_sdk::Vec<soroban_sdk::Address> {
        env.storage()
            .persistent()
            .get(&DataKey::MultisigSigners(grant_id))
            .unwrap_or(soroban_sdk::Vec::new(env))
    }

    pub fn set_multisig_signers(
        env: &Env,
        grant_id: u64,
        signers: &soroban_sdk::Vec<soroban_sdk::Address>,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::MultisigSigners(grant_id), signers);
    }

    pub fn has_release_approval(env: &Env, grant_id: u64, signer: &soroban_sdk::Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::ReleaseSignerApproval(grant_id, signer.clone()))
            .unwrap_or(false)
    }

    pub fn set_release_approval(
        env: &Env,
        grant_id: u64,
        signer: &soroban_sdk::Address,
        approved: bool,
    ) {
        env.storage().persistent().set(
            &DataKey::ReleaseSignerApproval(grant_id, signer.clone()),
            &approved,
        );
    }

    pub fn get_grant_min_reputation(env: &Env, grant_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::GrantMinReputation(grant_id))
            .unwrap_or(0)
    }

    pub fn set_grant_min_reputation(env: &Env, grant_id: u64, min_reputation: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::GrantMinReputation(grant_id), &min_reputation);
    }

    pub fn has_milestone_upvote(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: &soroban_sdk::Address,
    ) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::MilestoneUpvoter(
                grant_id,
                milestone_idx,
                voter.clone(),
            ))
            .unwrap_or(false)
    }

    pub fn set_milestone_upvote(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: &soroban_sdk::Address,
    ) {
        env.storage().persistent().set(
            &DataKey::MilestoneUpvoter(grant_id, milestone_idx, voter.clone()),
            &true,
        );
    }
}
