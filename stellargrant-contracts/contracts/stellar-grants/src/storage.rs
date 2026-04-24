use crate::types::{
    AccessControl, BountySubmissionEntry, DisputeInfo, EscrowLifecycleState, EscrowMode,
    EscrowState, ExtensionRequest, Grant, Milestone, Role,
};
use soroban_sdk::{contracttype, Address, Env, Map, Vec};

#[contracttype]
pub enum DataKey {
    Grant(u64),
    Milestone(u64, u32),
    GrantCounter,
    Contributor(soroban_sdk::Address),
    ReviewerStake(u64, soroban_sdk::Address),
    MinReviewerStake,
    Treasury,
    IdentityOracle,
    ReviewerReputation(soroban_sdk::Address),
    GlobalAdmin,
    Council,
    EscrowState(u64),
    MultisigSigners(u64),
    ReleaseSignerApproval(u64, soroban_sdk::Address),
    GrantMinReputation(u64),
    MilestoneUpvoter(u64, u32, soroban_sdk::Address),
    Blacklist(soroban_sdk::Address),
    GrantStatusIndex(u32),
    StorageVersion,
    IsPaused,
    /// Per-grant reviewer delegation map: grant_id -> (delegator -> delegatee).
    Delegation,
    /// Tracks whether reputation was already credited for a milestone payout (issue #151).
    MilestoneReputationApplied(u64, u32),
    DisputeFeeAmount,
    MilestoneDisputeInfo(u64, u32),
    AccessControl(soroban_sdk::Address),
    RoleMemberCount(u32),
    ExtensionRequest(u64, u32),
    BountySubmissions(u64, u32),
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

    pub fn get_access_control(env: &Env, address: &soroban_sdk::Address) -> AccessControl {
        let key = DataKey::AccessControl(address.clone());
        let access: AccessControl = env.storage().persistent().get(&key).unwrap_or_default();
        if access.role_flags != 0 {
            Self::bump_persistent_ttl(env, &key);
        }
        access
    }

    pub fn set_access_control(
        env: &Env,
        address: &soroban_sdk::Address,
        access_control: &AccessControl,
    ) {
        let key = DataKey::AccessControl(address.clone());
        env.storage().persistent().set(&key, access_control);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn remove_access_control(env: &Env, address: &soroban_sdk::Address) {
        env.storage()
            .persistent()
            .remove(&DataKey::AccessControl(address.clone()));
    }

    pub fn get_role_member_count(env: &Env, role: Role) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::RoleMemberCount(role as u32))
            .unwrap_or(0)
    }

    pub fn set_role_member_count(env: &Env, role: Role, count: u32) {
        let key = DataKey::RoleMemberCount(role as u32);
        env.storage().persistent().set(&key, &count);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn get_storage_version(env: &Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::StorageVersion)
            .unwrap_or(1)
    }

    pub fn set_storage_version(env: &Env, version: u32) {
        env.storage()
            .persistent()
            .set(&DataKey::StorageVersion, &version);
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

    pub fn get_bounty_submissions(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Option<Vec<BountySubmissionEntry>> {
        let key = DataKey::BountySubmissions(grant_id, milestone_idx);
        let v = env.storage().persistent().get(&key);
        if v.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        v
    }

    pub fn set_bounty_submissions(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        entries: &Vec<BountySubmissionEntry>,
    ) {
        let key = DataKey::BountySubmissions(grant_id, milestone_idx);
        env.storage().persistent().set(&key, entries);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn remove_bounty_submissions(env: &Env, grant_id: u64, milestone_idx: u32) {
        env.storage()
            .persistent()
            .remove(&DataKey::BountySubmissions(grant_id, milestone_idx));
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
            .unwrap_or(EscrowState::new(
                EscrowMode::Standard,
                EscrowLifecycleState::Funding,
                false,
                0,
            ))
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

    pub fn is_blacklisted(env: &Env, address: &soroban_sdk::Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Blacklist(address.clone()))
    }

    pub fn set_blacklisted(env: &Env, address: &soroban_sdk::Address) {
        env.storage()
            .persistent()
            .set(&DataKey::Blacklist(address.clone()), &true);
    }

    pub fn remove_blacklisted(env: &Env, address: &soroban_sdk::Address) {
        env.storage()
            .persistent()
            .remove(&DataKey::Blacklist(address.clone()));
    }

    // --- Grant status index helpers ---
    pub const STATUS_INDEX_MAX: u32 = 500;

    pub fn get_status_index(env: &Env, status: u32) -> soroban_sdk::Vec<u64> {
        let key = DataKey::GrantStatusIndex(status);
        let vec = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(soroban_sdk::Vec::new(env));
        if !vec.is_empty() {
            Self::bump_persistent_ttl(env, &key);
        }
        vec
    }

    pub fn set_status_index(env: &Env, status: u32, ids: &soroban_sdk::Vec<u64>) {
        let key = DataKey::GrantStatusIndex(status);
        env.storage().persistent().set(&key, ids);
        Self::bump_persistent_ttl(env, &key);
    }
    pub fn index_add(env: &Env, status: u32, grant_id: u64) {
        let mut ids = Self::get_status_index(env, status);
        if ids.len() >= Self::STATUS_INDEX_MAX {
            return; // silently drop when cap is reached
        }
        if !ids.contains(grant_id) {
            ids.push_back(grant_id);
            Self::set_status_index(env, status, &ids);
        }
    }
    pub fn index_remove(env: &Env, status: u32, grant_id: u64) {
        let ids = Self::get_status_index(env, status);
        let mut new_ids = soroban_sdk::Vec::new(env);
        for id in ids.iter() {
            if id != grant_id {
                new_ids.push_back(id);
            }
        }
        Self::set_status_index(env, status, &new_ids);
    }
    pub fn index_transition(env: &Env, from: u32, to: u32, grant_id: u64) {
        Self::index_remove(env, from, grant_id);
        Self::index_add(env, to, grant_id);
    }

    // --- Global pause flag (instance storage) ---

    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::IsPaused)
            .unwrap_or(false)
    }

    pub fn set_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&DataKey::IsPaused, &paused);
    }

    // --- Reviewer delegation helpers ---

    pub fn get_delegation(env: &Env, grant_id: u64, delegator: &Address) -> Option<Address> {
        let key = DataKey::Delegation;
        let delegations: Map<u64, Map<Address, Address>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(env));
        if !delegations.is_empty() {
            Self::bump_persistent_ttl(env, &key);
        }

        delegations
            .get(grant_id)
            .and_then(|grant_delegations| grant_delegations.get(delegator.clone()))
    }

    pub fn set_delegation(env: &Env, grant_id: u64, delegator: &Address, delegatee: &Address) {
        let key = DataKey::Delegation;
        let mut delegations: Map<u64, Map<Address, Address>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(env));
        let mut grant_delegations = delegations.get(grant_id).unwrap_or(Map::new(env));

        grant_delegations.set(delegator.clone(), delegatee.clone());
        delegations.set(grant_id, grant_delegations);

        env.storage().persistent().set(&key, &delegations);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn remove_delegation(env: &Env, grant_id: u64, delegator: &Address) {
        let key = DataKey::Delegation;
        let mut delegations: Map<u64, Map<Address, Address>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(env));
        let Some(mut grant_delegations) = delegations.get(grant_id) else {
            return;
        };

        if !grant_delegations.contains_key(delegator.clone()) {
            return;
        }

        grant_delegations.remove(delegator.clone());
        if grant_delegations.is_empty() {
            delegations.remove(grant_id);
        } else {
            delegations.set(grant_id, grant_delegations);
        }

        env.storage().persistent().set(&key, &delegations);
        Self::bump_persistent_ttl(env, &key);
    }

    // --- Issue #151: milestone reputation tracking ---

    pub fn has_milestone_reputation_applied(env: &Env, grant_id: u64, milestone_idx: u32) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::MilestoneReputationApplied(
                grant_id,
                milestone_idx,
            ))
    }

    pub fn mark_milestone_reputation_applied(env: &Env, grant_id: u64, milestone_idx: u32) {
        let key = DataKey::MilestoneReputationApplied(grant_id, milestone_idx);
        env.storage().persistent().set(&key, &true);
        Self::bump_persistent_ttl(env, &key);
    }

    // --- Issue #152: dispute fee ---

    pub fn get_dispute_fee_amount(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::DisputeFeeAmount)
            .unwrap_or(0)
    }

    pub fn set_dispute_fee_amount(env: &Env, amount: i128) {
        let key = DataKey::DisputeFeeAmount;
        env.storage().persistent().set(&key, &amount);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn get_milestone_dispute_info(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Option<DisputeInfo> {
        let key = DataKey::MilestoneDisputeInfo(grant_id, milestone_idx);
        let info = env.storage().persistent().get(&key);
        if info.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        info
    }

    pub fn set_milestone_dispute_info(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        info: &DisputeInfo,
    ) {
        let key = DataKey::MilestoneDisputeInfo(grant_id, milestone_idx);
        env.storage().persistent().set(&key, info);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn remove_milestone_dispute_info(env: &Env, grant_id: u64, milestone_idx: u32) {
        env.storage()
            .persistent()
            .remove(&DataKey::MilestoneDisputeInfo(grant_id, milestone_idx));
    }

    pub fn get_extension_request(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Option<ExtensionRequest> {
        let key = DataKey::ExtensionRequest(grant_id, milestone_idx);
        let request = env.storage().persistent().get(&key);
        if request.is_some() {
            Self::bump_persistent_ttl(env, &key);
        }
        request
    }

    pub fn set_extension_request(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        request: &ExtensionRequest,
    ) {
        let key = DataKey::ExtensionRequest(grant_id, milestone_idx);
        env.storage().persistent().set(&key, request);
        Self::bump_persistent_ttl(env, &key);
    }

    pub fn remove_extension_request(env: &Env, grant_id: u64, milestone_idx: u32) {
        env.storage()
            .persistent()
            .remove(&DataKey::ExtensionRequest(grant_id, milestone_idx));
    }
}
