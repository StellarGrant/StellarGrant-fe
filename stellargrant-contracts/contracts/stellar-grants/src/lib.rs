#![no_std]
#![allow(clippy::too_many_arguments)]
pub fn get_milestone(env: Env, grant_id: u64, milestone_idx: u32) -> Option<Milestone> {
    Storage::get_milestone(&env, grant_id, milestone_idx)
}
mod access;
mod events;
mod reentrancy;
mod storage;
mod types;

pub use events::Events;
pub use storage::Storage;
pub use types::{
    AccessControl, ContractError, DisputeInfo, EscrowLifecycleState, EscrowMode, EscrowState,
    ExtensionRequest, Grant, GrantFund, GrantStatus, Milestone, MilestoneState,
    MilestoneSubmission, Role,
};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String, Vec};
pub const COMMUNITY_REVIEW_PERIOD: u64 = 3 * 24 * 60 * 60;
pub const CHALLENGE_PERIOD: u64 = 48 * 60 * 60;
pub const CANCEL_GRACE_PERIOD: u64 = 7 * 24 * 60 * 60;

#[contract]
pub struct StellarGrantsContract;

#[contractimpl]
impl StellarGrantsContract {
    pub fn dispute_milestone(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        let is_reviewer = grant.reviewers.contains(caller.clone());
        let is_owner = grant.owner == caller;
        if !(is_owner || is_reviewer) {
            return Err(ContractError::Unauthorized);
        }
        if milestone.state() != MilestoneState::Submitted
            && milestone.state() != MilestoneState::Approved
            && milestone.state() != MilestoneState::Paid
            && milestone.state() != MilestoneState::AwaitingPayout
        {
            return Err(ContractError::InvalidState);
        }

        // Issue #152: collect dispute fee if configured
        let fee_amount = Storage::get_dispute_fee_amount(&env);
        if fee_amount > 0 {
            if Storage::get_milestone_dispute_info(&env, grant_id, milestone_idx).is_some() {
                return Err(ContractError::DisputeAlreadyCharged);
            }
            let fee_token = milestone.payout_token.clone();
            let token_client = token::Client::new(&env, &fee_token);
            token_client.transfer(&caller, env.current_contract_address(), &fee_amount);

            Storage::set_milestone_dispute_info(
                &env,
                grant_id,
                milestone_idx,
                &DisputeInfo {
                    payer: caller.clone(),
                    fee_amount,
                    fee_token: fee_token.clone(),
                },
            );
            Events::emit_dispute_fee_charged(
                &env,
                grant_id,
                milestone_idx,
                caller.clone(),
                fee_amount,
            );
        }

        milestone.set_state(MilestoneState::Disputed);
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Disputed);
        Ok(())
    }
    pub fn milestone_approve(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<(), ContractError> {
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        Self::internal_milestone_approve(&env, &mut grant, grant_id, milestone_idx)?;
        Storage::set_grant(&env, grant_id, &grant);
        Ok(())
    }
    fn internal_milestone_approve(
        env: &Env,
        grant: &mut Grant,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<(), ContractError> {
        let mut milestone = Storage::get_milestone(env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        if milestone.state() == MilestoneState::Approved {
            return Err(ContractError::MilestoneAlreadyApproved);
        }

        if milestone.state() != MilestoneState::Submitted {
            return Err(ContractError::InvalidState);
        }

        if milestone.approvals() < grant.quorum() {
            return Err(ContractError::QuorumNotReached);
        }

        let amount = milestone.amount;
        let payout_token = milestone.payout_token.clone();
        let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);

        if current_balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        // State Update
        milestone.set_state(MilestoneState::Approved);
        milestone.status_updated_at = env.ledger().timestamp();

        let recipient = grant.owner.clone();

        // Payout Execution & balance deduction
        let new_balance = current_balance
            .checked_sub(amount)
            .ok_or(ContractError::InsufficientBalance)?;
        grant.escrow_balances.set(payout_token.clone(), new_balance);
        grant.set_milestones_paid_out(grant.milestones_paid_out() + 1);

        Storage::set_milestone(env, grant_id, milestone_idx, &milestone);

        let token_client = token::Client::new(env, &payout_token);
        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        // Issue #151: credit reputation after payout
        Self::update_contributor_reputation(env, grant_id, milestone_idx, &recipient, amount);

        // Events
        Events::emit_milestone_approved(
            env,
            grant_id,
            milestone_idx,
            amount,
            payout_token.clone(),
            recipient.clone(),
        );
        Events::emit_payout_executed(env, grant_id, recipient, amount, payout_token);

        Ok(())
    }
    pub fn batch_milestone_approve(
        env: Env,
        grant_id: u64,
        milestone_indices: Vec<u32>,
        reviewer: Address,
    ) -> Result<(), ContractError> {
        reviewer.require_auth();
        assert_not_paused(&env)?;

        let batch_len = milestone_indices.len();
        if batch_len == 0 {
            return Err(ContractError::BatchEmpty);
        }
        if batch_len > 20 {
            return Err(ContractError::BatchTooLarge);
        }

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        // Ensure the caller is an authorized reviewer for this grant
        if !grant.reviewers.contains(reviewer) {
            return Err(ContractError::Unauthorized);
        }

        for milestone_idx in milestone_indices.iter() {
            Self::internal_milestone_approve(&env, &mut grant, grant_id, milestone_idx)?;
        }

        Storage::set_grant(&env, grant_id, &grant);
        Ok(())
    }
    pub fn resolve_dispute(
        env: Env,
        council: Address,
        grant_id: u64,
        milestone_idx: u32,
        approve: bool,
    ) -> Result<(), ContractError> {
        council.require_auth();
        let council_addr = Storage::get_council(&env).ok_or(ContractError::InvalidInput)?;
        if council_addr != council {
            return Err(ContractError::Unauthorized);
        }
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        if milestone.state() != MilestoneState::Disputed {
            return Err(ContractError::InvalidState);
        }
        milestone.set_state(MilestoneState::Resolved);
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Resolved);

        // Was the milestone already paid out before being disputed?
        // Track this from milestone dispute state transition tracking (use a storage key or inline check).
        // For simplicity, if approve=true and escrow_balance < milestone_amount,
        // we assume it was auto-paid at quorum, so no additional transfer is needed.
        // Fetch grant for payout/refund
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        let payout_token = milestone.payout_token.clone();
        let current_payout_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);
        let milestone_already_paid = current_payout_balance < milestone.amount;
        let token_client = token::Client::new(&env, &payout_token);

        if approve {
            if !milestone_already_paid {
                // Approve: payout milestone amount to grant owner (contributor)
                let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);
                if current_balance < milestone.amount {
                    return Err(ContractError::InvalidInput);
                }
                token_client.transfer(
                    &env.current_contract_address(),
                    &grant.owner,
                    &milestone.amount,
                );

                grant
                    .escrow_balances
                    .set(payout_token.clone(), current_balance - milestone.amount);
                grant.set_milestones_paid_out(grant.milestones_paid_out() + 1);
                Storage::set_grant(&env, grant_id, &grant);

                // Issue #151: credit reputation for the payout
                Self::update_contributor_reputation(
                    &env,
                    grant_id,
                    milestone_idx,
                    &grant.owner,
                    milestone.amount,
                );
            }
            Events::emit_milestone_paid(
                &env,
                grant_id,
                milestone_idx,
                milestone.amount,
                payout_token.clone(),
            );

            // Issue #152: refund dispute fee to caller (dispute was upheld)
            if let Some(dispute_info) =
                Storage::get_milestone_dispute_info(&env, grant_id, milestone_idx)
            {
                if dispute_info.fee_amount > 0 {
                    let fee_token_client = token::Client::new(&env, &dispute_info.fee_token);
                    fee_token_client.transfer(
                        &env.current_contract_address(),
                        &dispute_info.payer,
                        &dispute_info.fee_amount,
                    );
                    Events::emit_dispute_fee_refunded(
                        &env,
                        grant_id,
                        milestone_idx,
                        dispute_info.payer.clone(),
                        dispute_info.fee_amount,
                    );
                }
                Storage::remove_milestone_dispute_info(&env, grant_id, milestone_idx);
            }
        } else {
            // Reject: refund milestone amount to funders (pro-rata)
            let total_refundable = milestone.amount;
            let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);
            if current_balance < total_refundable {
                return Err(ContractError::InvalidInput);
            }
            refund_token_to_funders(
                &env,
                grant_id,
                &grant.funders,
                &payout_token,
                total_refundable,
            )?;
            grant
                .escrow_balances
                .set(payout_token.clone(), current_balance - total_refundable);
            Storage::set_grant(&env, grant_id, &grant);

            // Issue #152: slash dispute fee → treasury (dispute dismissed)
            if let Some(dispute_info) =
                Storage::get_milestone_dispute_info(&env, grant_id, milestone_idx)
            {
                if dispute_info.fee_amount > 0 {
                    let fee_token_client = token::Client::new(&env, &dispute_info.fee_token);
                    if let Some(treasury) = Storage::get_treasury(&env) {
                        fee_token_client.transfer(
                            &env.current_contract_address(),
                            &treasury,
                            &dispute_info.fee_amount,
                        );
                        Events::emit_dispute_fee_slashed(
                            &env,
                            grant_id,
                            milestone_idx,
                            treasury,
                            dispute_info.fee_amount,
                        );
                    }
                }
                Storage::remove_milestone_dispute_info(&env, grant_id, milestone_idx);
            }
        }
        Ok(())
    }
    pub fn grant_clawback(env: Env, council: Address, grant_id: u64) -> Result<(), ContractError> {
        council.require_auth();

        let council_addr = Storage::get_council(&env).ok_or(ContractError::InvalidInput)?;
        if council_addr != council {
            return Err(ContractError::Unauthorized);
        }

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        // Only clawback grants that are still active (not already cancelled/completed).
        if grant.status() == GrantStatus::Cancelled || grant.status() == GrantStatus::Completed {
            return Err(ContractError::InvalidState);
        }

        let mut total_clawed_back: i128 = 0;

        // Collect all token addresses from escrow_balances to avoid borrow issues.
        let mut token_list: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(&env);
        for (token, _) in grant.escrow_balances.iter() {
            token_list.push_back(token);
        }

        for token in token_list.iter() {
            let balance = grant.escrow_balances.get(token.clone()).unwrap_or(0);
            if balance <= 0 {
                continue;
            }

            let token_client = token::Client::new(&env, &token);

            // Pro-rata refund to funders who contributed in this token.
            if has_token_funders(&grant.funders, &token) {
                refund_token_to_funders(&env, grant_id, &grant.funders, &token, balance)?;
            } else {
                // No funders recorded for this token; send the entire balance to the council
                // as a fallback to avoid permanently locking funds.
                token_client.transfer(&env.current_contract_address(), &council, &balance);
            }

            total_clawed_back += balance;
            grant.escrow_balances.set(token.clone(), 0);
        }

        grant.set_status(GrantStatus::Cancelled);
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_grant_clawbacked(&env, grant_id, council, total_clawed_back);

        Ok(())
    }
    pub fn grant_withdraw(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<(), ContractError> {
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        grant.owner.require_auth();

        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        if milestone.state() != MilestoneState::Approved {
            return Err(ContractError::InvalidState);
        }

        let payout_token = milestone.payout_token.clone();
        let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);

        if current_balance < milestone.amount {
            return Err(ContractError::InvalidInput);
        }

        let token_client = token::Client::new(&env, &payout_token);
        token_client.transfer(
            &env.current_contract_address(),
            &grant.owner,
            &milestone.amount,
        );

        grant
            .escrow_balances
            .set(payout_token.clone(), current_balance - milestone.amount);
        grant.set_milestones_paid_out(grant.milestones_paid_out() + 1);
        milestone.state = MilestoneState::Paid;
        milestone.status_updated_at = env.ledger().timestamp();

        Storage::set_grant(&env, grant_id, &grant);
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        // Issue #151: credit reputation after manual withdrawal payout
        let owner_clone = grant.owner.clone();
        let amount_clone = milestone.amount;
        Self::update_contributor_reputation(
            &env,
            grant_id,
            milestone_idx,
            &owner_clone,
            amount_clone,
        );

        Events::emit_milestone_paid(
            &env,
            grant_id,
            milestone_idx,
            milestone.amount,
            payout_token,
        );
        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Paid);

        Ok(())
    }
    pub fn request_milestone_extension(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        new_deadline: u64,
    ) -> Result<(), ContractError> {
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        grant.owner.require_auth();
        assert_not_paused(&env)?;

        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        let milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        if !milestone_allows_extension(&milestone) {
            return Err(ContractError::ExtensionDenied);
        }
        if is_milestone_deadline_elapsed(&env, &milestone) {
            return Err(ContractError::DeadlinePassed);
        }
        if new_deadline <= env.ledger().timestamp() {
            return Err(ContractError::ExtensionDenied);
        }
        if milestone.deadline_timestamp != 0 && new_deadline <= milestone.deadline_timestamp {
            return Err(ContractError::ExtensionDenied);
        }
        if Storage::get_extension_request(&env, grant_id, milestone_idx).is_some() {
            return Err(ContractError::ExtensionDenied);
        }

        let extension_request = ExtensionRequest::new(&env, grant.owner.clone(), new_deadline);
        Storage::set_extension_request(&env, grant_id, milestone_idx, &extension_request);

        Events::emit_milestone_extension_requested(
            &env,
            grant_id,
            milestone_idx,
            extension_request.new_deadline,
        );

        Ok(())
    }
    pub fn approve_milestone_extension(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
    ) -> Result<(), ContractError> {
        reviewer.require_auth();
        assert_not_paused(&env)?;

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }
        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }
        access::require_optional_role(&env, &reviewer, Role::Reviewer)?;

        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        if !milestone_allows_extension(&milestone) {
            return Err(ContractError::ExtensionDenied);
        }
        if is_milestone_deadline_elapsed(&env, &milestone) {
            return Err(ContractError::DeadlinePassed);
        }

        let mut extension_request = Storage::get_extension_request(&env, grant_id, milestone_idx)
            .ok_or(ContractError::ExtensionDenied)?;
        if extension_request.approvals.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        extension_request.approvals.set(reviewer.clone(), true);
        extension_request.approvals_count = extension_request.approvals_count.saturating_add(1);

        Events::emit_milestone_extension_approved(
            &env,
            grant_id,
            milestone_idx,
            reviewer,
            extension_request.approvals_count,
            grant.quorum(),
        );

        if extension_request.approvals_count >= grant.quorum() {
            milestone.deadline_timestamp = extension_request.new_deadline;
            Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
            Storage::remove_extension_request(&env, grant_id, milestone_idx);
            return Ok(());
        }

        Storage::set_extension_request(&env, grant_id, milestone_idx, &extension_request);

        Ok(())
    }

    pub fn check_expiry(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<bool, ContractError> {
        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        Ok(is_milestone_expired(&env, &milestone))
    }

    pub fn claim_expired_funds(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        reentrancy::with_non_reentrant(&env, || {
            let mut grant =
                Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status() == GrantStatus::Cancelled || grant.status() == GrantStatus::Completed
            {
                return Err(ContractError::InvalidState);
            }
            if grant.owner != caller
                && !grant_has_funder(&grant, &caller)?
                && !is_admin_actor(&env, &caller)
            {
                return Err(ContractError::Unauthorized);
            }

            let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
                .ok_or(ContractError::MilestoneNotFound)?;
            if milestone.deadline_timestamp == 0 {
                return Err(ContractError::DeadlineNotSet);
            }

            let expired =
                mark_milestone_expired_if_needed(&env, grant_id, milestone_idx, &mut milestone)?;
            if !expired || milestone.state() != MilestoneState::Expired {
                return Err(ContractError::ExpiryNotReached);
            }

            let payout_token = milestone.payout_token.clone();
            let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);
            if current_balance < milestone.amount {
                return Err(ContractError::InsufficientBalance);
            }

            refund_token_to_funders(
                &env,
                grant_id,
                &grant.funders,
                &payout_token,
                milestone.amount,
            )?;

            grant
                .escrow_balances
                .set(payout_token.clone(), current_balance - milestone.amount);
            milestone.set_state(MilestoneState::ExpiredClaimed);
            milestone.status_updated_at = env.ledger().timestamp();

            Storage::set_grant(&env, grant_id, &grant);
            Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

            Events::milestone_status_changed(
                &env,
                grant_id,
                milestone_idx,
                MilestoneState::ExpiredClaimed,
            );
            Events::emit_expired_funds_claimed(
                &env,
                grant_id,
                milestone_idx,
                caller,
                milestone.amount,
                payout_token,
            );
            Ok(())
        })
    }
    pub fn set_dispute_fee(
        env: Env,
        admin: Address,
        fee_amount: i128,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        Storage::set_dispute_fee_amount(&env, fee_amount);
        Ok(())
    }
    pub fn get_dispute_fee(env: Env) -> i128 {
        Storage::get_dispute_fee_amount(&env)
    }

    pub fn grant_role(
        env: Env,
        admin: Address,
        account: Address,
        role: Role,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        access::grant_role(&env, &account, role)?;
        Events::emit_role_granted(&env, admin, account, role);
        Ok(())
    }

    pub fn revoke_role(
        env: Env,
        admin: Address,
        account: Address,
        role: Role,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        if role == Role::Admin && Storage::get_global_admin(&env) == Some(account.clone()) {
            return Err(ContractError::InvalidInput);
        }
        access::revoke_role(&env, &account, role)?;
        Events::emit_role_revoked(&env, admin, account, role);
        Ok(())
    }

    pub fn renounce_role(env: Env, account: Address, role: Role) -> Result<(), ContractError> {
        account.require_auth();
        if role == Role::Admin && Storage::get_global_admin(&env) == Some(account.clone()) {
            return Err(ContractError::InvalidInput);
        }
        access::renounce_role(&env, &account, role)?;
        Events::emit_role_renounced(&env, account, role);
        Ok(())
    }

    pub fn has_role(env: Env, account: Address, role: Role) -> bool {
        if role == Role::Admin && Storage::get_global_admin(&env) == Some(account.clone()) {
            return true;
        }
        access::has_role(&env, &account, role)
    }

    pub fn get_access_control(env: Env, account: Address) -> AccessControl {
        let mut access_control = Storage::get_access_control(&env, &account);
        if Storage::get_global_admin(&env) == Some(account) {
            access_control.grant(Role::Admin);
        }
        access_control
    }
    pub fn initialize(env: Env, admin: Address, council: Address) -> Result<(), ContractError> {
        if Storage::get_global_admin(&env).is_some() {
            return Err(ContractError::InvalidInput);
        }
        admin.require_auth();
        Storage::set_global_admin(&env, &admin);
        Storage::set_council(&env, &council);
        Storage::set_storage_version(&env, 1);
        access::grant_role(&env, &admin, Role::Admin)?;
        access::grant_role(&env, &admin, Role::Pauser)?;
        Events::emit_contract_initialized(&env, council);
        // Enhanced event emission: include all relevant data, standardize topics
        Ok(())
    }
    pub fn admin_change(
        env: Env,
        old_admin: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        old_admin.require_auth();
        let current = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if current != old_admin || !is_admin_actor(&env, &old_admin) {
            return Err(ContractError::NotContractAdmin);
        }
        Storage::set_global_admin(&env, &new_admin);
        if !access::has_role(&env, &new_admin, Role::Admin) {
            access::grant_role(&env, &new_admin, Role::Admin)?;
        }
        if !access::has_role(&env, &new_admin, Role::Pauser) {
            access::grant_role(&env, &new_admin, Role::Pauser)?;
        }
        if old_admin != new_admin {
            if access::has_role(&env, &old_admin, Role::Admin) {
                access::revoke_role(&env, &old_admin, Role::Admin)?;
            }
            if access::has_role(&env, &old_admin, Role::Pauser) {
                access::revoke_role(&env, &old_admin, Role::Pauser)?;
            }
        }
        Events::emit_contract_upgraded(&env, old_admin, String::from_str(&env, "admin_changed"));
        Ok(())
    }
    pub fn admin_upgrade(
        env: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        let next = Storage::get_storage_version(&env).saturating_add(1);
        Storage::set_storage_version(&env, next);
        Events::emit_contract_wasm_upgraded(&env, admin.clone(), new_wasm_hash.clone(), next);
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
    pub fn get_contract_storage_version(env: Env) -> u32 {
        Storage::get_storage_version(&env)
    }
    pub fn set_council(env: Env, caller: Address, council: Address) -> Result<(), ContractError> {
        caller.require_auth();
        require_admin_actor(&env, &caller)?;
        Storage::set_council(&env, &council);
        Events::emit_contract_upgraded(&env, caller, String::from_str(&env, "council_updated"));
        // Enhanced event emission: include all relevant data, standardize topics
        Ok(())
    }

    // ── Pausable module ──────────────────────────────────────────────
    pub fn pause(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        if !is_admin_actor(&env, &caller) && !access::has_role(&env, &caller, Role::Pauser) {
            return Err(ContractError::Unauthorized);
        }
        Storage::set_paused(&env, true);
        Events::emit_contract_upgraded(&env, caller, String::from_str(&env, "paused"));
        Ok(())
    }
    pub fn unpause(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        if !is_admin_actor(&env, &caller) && !access::has_role(&env, &caller, Role::Pauser) {
            return Err(ContractError::Unauthorized);
        }
        Storage::set_paused(&env, false);
        Events::emit_contract_upgraded(&env, caller, String::from_str(&env, "unpaused"));
        Ok(())
    }
    pub fn is_paused(env: Env) -> bool {
        Storage::is_paused(&env)
    }
    pub fn grant_update_metadata(
        env: Env,
        grant_id: u64,
        owner: Address,
        new_title: String,
        new_description: String,
    ) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }
        if grant.status() == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        grant.title = new_title.clone();
        grant.description = new_description.clone();
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_grant_metadata_updated(&env, grant_id, owner, new_title, new_description);
        // Enhanced event emission: include all relevant data, standardize topics
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn grant_create(
        env: Env,
        owner: Address,
        title: String,
        description: String,
        token: Address,
        total_amount: i128,
        milestone_amount: i128,
        num_milestones: u32,
        reviewers: soroban_sdk::Vec<Address>,
        quorum: u32,
        milestone_deadlines: Option<soroban_sdk::Vec<u64>>,
        min_funding: i128,
        hard_cap: i128,
        tags: soroban_sdk::Vec<String>,
    ) -> Result<u64, ContractError> {
        owner.require_auth();
        assert_not_paused(&env)?;
        access::require_optional_role(&env, &owner, Role::GrantCreator)?;

        if Storage::is_blacklisted(&env, &owner) {
            return Err(ContractError::Blacklisted);
        }

        if let Some(ref deadlines) = milestone_deadlines {
            if deadlines.len() != num_milestones {
                return Err(ContractError::InvalidInput);
            }
        }

        if total_amount <= 0 || milestone_amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        if num_milestones == 0 || num_milestones > 100 {
            return Err(ContractError::InvalidInput);
        }
        let total_reviewers = reviewers.len();
        if quorum == 0 || quorum > total_reviewers {
            return Err(ContractError::InvalidInput);
        }

        let total_required = milestone_amount
            .checked_mul(num_milestones as i128)
            .ok_or(ContractError::InvalidInput)?;

        if total_amount < total_required {
            return Err(ContractError::InvalidInput);
        }

        // Validate tags: max 5 tags, each max 20 chars
        if tags.len() > 5 {
            return Err(ContractError::TooManyTags);
        }
        for tag in tags.iter() {
            if tag.len() > 20 {
                return Err(ContractError::TagTooLong);
            }
        }
        for reviewer in reviewers.iter() {
            access::require_optional_role(&env, &reviewer, Role::Reviewer)?;
        }

        let grant_id = Storage::increment_grant_counter(&env);

        // All grants start in PendingAcceptance; the recipient (owner) must explicitly
        // call grant_accept before any funding or milestone activity can begin.
        let initial_status = GrantStatus::PendingAcceptance;

        let mut grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            title: title.clone(),
            description,
            primary_token: token.clone(),
            total_amount,
            milestone_amount,
            reviewers,
            escrow_balances: soroban_sdk::Map::new(&env), // Initialize empty map
            funders: soroban_sdk::Vec::new(&env),
            reason: None,
            timestamp: env.ledger().timestamp(),
            last_heartbeat: env.ledger().timestamp(),
            cancellation_requested_at: None,
            min_funding,
            hard_cap,
            tags: tags.clone(),
            packed_config: 0,
        };
        grant.set_status(initial_status);
        grant.set_quorum(quorum);
        grant.set_total_milestones(num_milestones);
        grant.set_milestones_paid_out(0);

        Storage::set_grant(&env, grant_id, &grant);
        Storage::index_add(&env, initial_status as u32, grant_id);
        Storage::set_grant_min_reputation(&env, grant_id, 0);
        Storage::set_escrow_state(
            &env,
            grant_id,
            &EscrowState::new(
                EscrowMode::Standard,
                EscrowLifecycleState::Funding,
                false,
                0,
            ),
        );
        Storage::set_multisig_signers(&env, grant_id, &soroban_sdk::Vec::new(&env));

        for i in 0..num_milestones {
            let deadline = if let Some(ref deadlines) = milestone_deadlines {
                deadlines.get(i).unwrap_or(0)
            } else {
                0
            };

            let mut milestone = Milestone {
                description: String::from_str(&env, ""),
                amount: milestone_amount,
                payout_token: token.clone(), // Default to primary token
                state: MilestoneState::Pending,
                votes: soroban_sdk::Map::new(&env),
                reasons: soroban_sdk::Map::new(&env),
                status_updated_at: 0,
                proof_url: None,
                submission_timestamp: 0,
                deadline_timestamp: deadline,
                community_comments: soroban_sdk::Map::new(&env),
                packed_stats: 0,
            };
            milestone.set_idx(i);
            milestone.set_approvals(0);
            milestone.set_rejections(0);
            milestone.set_community_upvotes(0);
            Storage::set_milestone(&env, grant_id, i, &milestone);
        }
        // Enhanced event emission: include all relevant data, standardize topics
        Events::emit_grant_created(
            &env,
            grant_id,
            owner.clone(),
            title.clone(),
            total_amount,
            tags,
        );

        Ok(grant_id)
    }
    pub fn grant_accept(env: Env, grant_id: u64, recipient: Address) -> Result<(), ContractError> {
        recipient.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.owner != recipient {
            return Err(ContractError::Unauthorized);
        }

        if grant.status() != GrantStatus::PendingAcceptance {
            return Err(ContractError::InvalidState);
        }

        let new_status = if grant.min_funding > 0 {
            GrantStatus::PendingFunding
        } else {
            GrantStatus::Active
        };

        grant.set_status(new_status);
        Storage::set_grant(&env, grant_id, &grant);
        Storage::index_transition(
            &env,
            GrantStatus::PendingAcceptance as u32,
            new_status as u32,
            grant_id,
        );

        Events::emit_grant_accepted(&env, grant_id, recipient);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn grant_create_with_rep_req(
        env: Env,
        owner: Address,
        title: String,
        description: String,
        token: Address,
        total_amount: i128,
        milestone_amount: i128,
        num_milestones: u32,
        reviewers: soroban_sdk::Vec<Address>,
        min_reputation_score: u64,
    ) -> Result<u64, ContractError> {
        let quorum = (reviewers.len() / 2) + 1;
        let grant_id = Self::grant_create(
            env.clone(),
            owner,
            title,
            description,
            token,
            total_amount,
            milestone_amount,
            num_milestones,
            reviewers,
            quorum,
            None,
            0,
            0,
            soroban_sdk::Vec::new(&env),
        )?;
        Storage::set_grant_min_reputation(&env, grant_id, min_reputation_score);
        Ok(grant_id)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn grant_create_high_security(
        env: Env,
        owner: Address,
        title: String,
        description: String,
        token: Address,
        total_amount: i128,
        milestone_amount: i128,
        num_milestones: u32,
        reviewers: soroban_sdk::Vec<Address>,
        multisig_signers: soroban_sdk::Vec<Address>,
    ) -> Result<u64, ContractError> {
        if multisig_signers.is_empty() {
            return Err(ContractError::InvalidInput);
        }
        let quorum = (reviewers.len() / 2) + 1;

        let grant_id = Self::grant_create(
            env.clone(),
            owner,
            title,
            description,
            token,
            total_amount,
            milestone_amount,
            num_milestones,
            reviewers,
            quorum,
            None,
            0,
            0,
            soroban_sdk::Vec::new(&env),
        )?;

        Storage::set_escrow_state(
            &env,
            grant_id,
            &EscrowState::new(
                EscrowMode::HighSecurity,
                EscrowLifecycleState::Funding,
                false,
                0,
            ),
        );
        Storage::set_multisig_signers(&env, grant_id, &multisig_signers);

        Ok(grant_id)
    }
    pub fn contributor_register(
        env: Env,
        contributor: Address,
        name: String,
        bio: String,
        skills: soroban_sdk::Vec<String>,
        github_url: String,
    ) -> Result<(), ContractError> {
        contributor.require_auth();

        if Storage::is_blacklisted(&env, &contributor) {
            return Err(ContractError::Blacklisted);
        }

        if name.is_empty() || name.len() > 100 {
            return Err(ContractError::InvalidInput);
        }
        if bio.len() > 500 {
            return Err(ContractError::InvalidInput);
        }

        if Storage::get_contributor(&env, contributor.clone()).is_some() {
            return Err(ContractError::AlreadyRegistered);
        }

        let profile = crate::types::ContributorProfile {
            contributor: contributor.clone(),
            name: name.clone(),
            bio,
            skills,
            github_url,
            registration_timestamp: env.ledger().timestamp(),
            reputation_score: 0,
            grants_count: 0,
            total_earned: 0,
        };

        Storage::set_contributor(&env, contributor.clone(), &profile);

        Events::emit_contributor_registered(&env, 0, contributor, name);
        // Enhanced event emission: include all relevant data, standardize topics

        Ok(())
    }
    pub fn grant_cancel(
        env: Env,
        grant_id: u64,
        owner: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        Self::cancel_grant(env, grant_id, owner, reason)
    }
    pub fn cancel_grant(
        env: Env,
        grant_id: u64,
        caller: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        reentrancy::with_non_reentrant(&env, || {
            let mut grant =
                Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            let caller_is_owner = grant.owner == caller;
            let caller_is_admin = is_admin_actor(&env, &caller);
            let grant_is_inactive = grant.status() == GrantStatus::Inactive;
            let caller_is_funder = grant_has_funder(&grant, &caller)?;

            let now = env.ledger().timestamp();
            let heartbeat_age = now.saturating_sub(grant.last_heartbeat);
            let heartbeat_timeout_60d = heartbeat_age > 60 * 24 * 60 * 60;

            if !(caller_is_admin
                || caller_is_owner
                || heartbeat_timeout_60d
                || (grant_is_inactive && caller_is_funder))
            {
                return Err(ContractError::Unauthorized);
            }

            let current_status = grant.status();
            match current_status {
                GrantStatus::Active | GrantStatus::Paused => {
                    // Check whether any milestone is still actively under review.
                    let mut has_active_submission = false;
                    for milestone_idx in 0..grant.total_milestones() {
                        if let Some(m) = Storage::get_milestone(&env, grant_id, milestone_idx) {
                            if m.state() == MilestoneState::Submitted
                                || m.state() == MilestoneState::CommunityReview
                            {
                                has_active_submission = true;
                                break;
                            }
                        }
                    }

                    if has_active_submission {
                        // Deferred cancellation — start grace period.
                        let executable_after = env.ledger().timestamp() + CANCEL_GRACE_PERIOD;
                        grant.set_status(GrantStatus::CancellationPending);
                        grant.cancellation_requested_at = Some(env.ledger().timestamp());
                        grant.reason = Some(reason.clone());
                        Storage::set_grant(&env, grant_id, &grant);
                        Storage::index_transition(
                            &env,
                            current_status as u32,
                            GrantStatus::CancellationPending as u32,
                            grant_id,
                        );
                        Events::emit_grant_cancellation_requested(
                            // Enhanced event emission: include all relevant data, standardize topics
                            &env,
                            grant_id,
                            caller,
                            reason,
                            executable_after,
                        );
                        return Ok(());
                    }
                    // No submitted milestones — fall through to immediate cancellation.
                }
                GrantStatus::CancellationPending => {
                    // Second call: check that the grace period has elapsed.
                    let requested_at = grant
                        .cancellation_requested_at
                        .unwrap_or(env.ledger().timestamp());
                    if env.ledger().timestamp() < requested_at + CANCEL_GRACE_PERIOD {
                        return Err(ContractError::CancellationGracePeriod);
                    }
                    // Grace period has elapsed — fall through to execute the refund.
                }
                _ => return Err(ContractError::InvalidState),
            }

            // Cannot cancel if all milestones are approved/paid out
            if grant.milestones_paid_out() >= grant.total_milestones() {
                return Err(ContractError::InvalidState);
            }

            for (token, balance) in grant.escrow_balances.iter() {
                if balance > 0 {
                    refund_token_to_funders(&env, grant_id, &grant.funders, &token, balance)?;
                }
            }

            // Update state
            grant.set_status(GrantStatus::Cancelled);
            grant.escrow_balances = soroban_sdk::Map::new(&env);
            grant.reason = Some(reason.clone());
            grant.timestamp = env.ledger().timestamp();

            Storage::set_grant(&env, grant_id, &grant);
            Storage::index_remove(&env, current_status as u32, grant_id);
            Storage::index_remove(&env, GrantStatus::CancellationPending as u32, grant_id);
            Storage::index_add(&env, GrantStatus::Cancelled as u32, grant_id);

            // Enhanced event emission: include all relevant data, standardize topics
            Events::emit_grant_cancelled(
                &env,
                grant_id,
                caller.clone(),
                reason.clone(),
                0, // Total refund amount is now per-token, so we use 0 as placeholder here or could aggregate
            );

            Ok(())
        })
    }
    pub fn grant_complete(env: Env, grant_id: u64) -> Result<(), ContractError> {
        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }

            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status() != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let mut escrow_state = Storage::get_escrow_state(&env, grant_id);
            if escrow_state.lifecycle() == EscrowLifecycleState::Released {
                return Err(ContractError::GrantAlreadyReleased);
            }

            // Quorum is interpreted as all milestones approved in current contract design.
            let _ =
                Self::compute_total_paid_if_quorum_ready(&env, grant_id, grant.total_milestones())?;
            escrow_state.set_quorum_ready(true);

            if escrow_state.mode() == EscrowMode::Standard {
                Self::finalize_grant_release(&env, grant_id)?;
                return Ok(());
            }

            // High-security grants remain locked until every multisig signer calls sign_release.
            escrow_state.set_lifecycle(EscrowLifecycleState::AwaitingMultisig);
            Storage::set_escrow_state(&env, grant_id, &escrow_state);
            Ok(())
        })
    }
    pub fn sign_release(env: Env, grant_id: u64, signer: Address) -> Result<(), ContractError> {
        signer.require_auth();
        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }

            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status() != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let mut escrow_state = Storage::get_escrow_state(&env, grant_id);
            if escrow_state.mode() != EscrowMode::HighSecurity {
                return Err(ContractError::InvalidState);
            }
            if escrow_state.lifecycle() == EscrowLifecycleState::Released {
                return Err(ContractError::GrantAlreadyReleased);
            }

            let signers = Storage::get_multisig_signers(&env, grant_id);
            if !signers.contains(signer.clone()) {
                return Err(ContractError::NotMultisigSigner);
            }
            if Storage::has_release_approval(&env, grant_id, &signer) {
                return Err(ContractError::AlreadySignedRelease);
            }

            Storage::set_release_approval(&env, grant_id, &signer, true);
            escrow_state.set_approvals_count(escrow_state.approvals_count() + 1);
            Storage::set_escrow_state(&env, grant_id, &escrow_state);

            let approvals_complete = escrow_state.approvals_count() >= signers.len();
            if approvals_complete && escrow_state.quorum_ready() {
                Self::finalize_grant_release(&env, grant_id)?;
            } else if approvals_complete {
                escrow_state.set_lifecycle(EscrowLifecycleState::AwaitingMultisig);
                Storage::set_escrow_state(&env, grant_id, &escrow_state);
            }

            Ok(())
        })
    }

    fn compute_total_paid_if_quorum_ready(
        env: &Env,
        grant_id: u64,
        total_milestones: u32,
    ) -> Result<i128, ContractError> {
        let mut total_paid: i128 = 0;
        let mut approved_count = 0;
        for milestone_idx in 0..total_milestones {
            if let Some(milestone) = Storage::get_milestone(env, grant_id, milestone_idx) {
                if milestone.state() != MilestoneState::Approved
                    && milestone.state() != MilestoneState::AwaitingPayout
                    && milestone.state() != MilestoneState::Paid
                {
                    return Err(ContractError::NotAllMilestonesApproved);
                }
                total_paid += milestone.amount;
                approved_count += 1;
            } else {
                return Err(ContractError::NotAllMilestonesApproved);
            }
        }
        if approved_count != total_milestones {
            return Err(ContractError::NotAllMilestonesApproved);
        }
        Ok(total_paid)
    }

    fn finalize_grant_release(env: &Env, grant_id: u64) -> Result<(), ContractError> {
        let mut grant = Storage::get_grant(env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.status() == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        let total_paid =
            Self::compute_total_paid_if_quorum_ready(env, grant_id, grant.total_milestones())?;
        let escrow_bal = grant
            .escrow_balances
            .get(grant.primary_token.clone())
            .unwrap_or(0);
        if escrow_bal < total_paid {
            return Err(ContractError::InvalidInput);
        }
        let remaining_balance = escrow_bal - total_paid;
        let token_client = token::Client::new(env, &grant.primary_token);

        if total_paid > 0 {
            token_client.transfer(&env.current_contract_address(), &grant.owner, &total_paid);
        }

        if remaining_balance > 0 {
            let mut total_contributions: i128 = 0;
            for fund_entry in grant.funders.iter() {
                total_contributions += fund_entry.amount;
            }

            if total_contributions > 0 {
                let funders_len = grant.funders.len();
                let mut distributed = 0i128;
                for i in 0..funders_len {
                    let fund_entry = funder_entry_at(&grant.funders, i)?;
                    let is_last = i + 1 == funders_len;
                    let refund_amount = if is_last {
                        remaining_balance - distributed
                    } else {
                        let amount = fund_entry
                            .amount
                            .checked_mul(remaining_balance)
                            .ok_or(ContractError::InvalidInput)?
                            .checked_div(total_contributions)
                            .ok_or(ContractError::InvalidInput)?;
                        distributed += amount;
                        amount
                    };

                    if refund_amount > 0 {
                        token_client.transfer(
                            &env.current_contract_address(),
                            &fund_entry.funder,
                            &refund_amount,
                        );
                        Events::emit_final_refund(
                            // Enhanced event emission: include all relevant data, standardize topics
                            env,
                            grant_id,
                            fund_entry.funder.clone(),
                            refund_amount,
                        );
                    }
                }
            }
        }

        // Mark all approved or awaiting payout milestones as paid
        for milestone_idx in 0..grant.total_milestones() {
            if let Some(mut milestone) = Storage::get_milestone(env, grant_id, milestone_idx) {
                if milestone.state() == MilestoneState::Approved
                    || milestone.state() == MilestoneState::AwaitingPayout
                {
                    if milestone.state() == MilestoneState::AwaitingPayout
                        && env.ledger().timestamp() < milestone.status_updated_at + CHALLENGE_PERIOD
                    {
                        return Err(ContractError::DeadlinePassed);
                    }
                    milestone.set_state(MilestoneState::Paid);
                    milestone.status_updated_at = env.ledger().timestamp();
                    Storage::set_milestone(env, grant_id, milestone_idx, &milestone);

                    Events::milestone_status_changed(
                        env,
                        grant_id,
                        milestone_idx,
                        MilestoneState::Paid,
                    );
                    Events::emit_milestone_paid(
                        env,
                        grant_id,
                        milestone_idx,
                        milestone.amount,
                        milestone.payout_token.clone(),
                    );
                }
            }
        }

        grant.set_status(GrantStatus::Completed);
        grant.escrow_balances = soroban_sdk::Map::new(env);
        grant.set_milestones_paid_out(grant.total_milestones());
        grant.timestamp = env.ledger().timestamp();
        Storage::set_grant(env, grant_id, &grant);
        Storage::index_transition(
            env,
            GrantStatus::Active as u32,
            GrantStatus::Completed as u32,
            grant_id,
        );

        if total_paid > 0 {
            if let Some(mut profile) = Storage::get_contributor(env, grant.owner.clone()) {
                profile.total_earned = profile
                    .total_earned
                    .checked_add(total_paid)
                    .ok_or(ContractError::InvalidInput)?;
                profile.reputation_score = profile
                    .reputation_score
                    .checked_add(grant.total_milestones() as u64)
                    .ok_or(ContractError::InvalidInput)?;
                Storage::set_contributor(env, grant.owner.clone(), &profile);
                Events::emit_reputation_increased(
                    env,
                    grant_id,
                    grant.owner.clone(),
                    profile.reputation_score,
                    profile.total_earned,
                );
            }
        }

        let mut escrow_state = Storage::get_escrow_state(env, grant_id);
        escrow_state.set_lifecycle(EscrowLifecycleState::Released);
        escrow_state.set_quorum_ready(true);
        Storage::set_escrow_state(env, grant_id, &escrow_state);

        // Emit a completion receipt snapshot for indexers.
        Events::emit_payee_receipt(
            env,
            grant_id,
            grant.owner.clone(),
            grant.primary_token.clone(),
            total_paid,
            None,
        );

        Events::emit_grant_completed(env, grant_id, total_paid, remaining_balance);
        Ok(())
    }
    pub fn milestone_vote(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approve: bool,
        feedback: Option<String>,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();
        assert_not_paused(&env)?;

        if Storage::is_blacklisted(&env, &reviewer) {
            return Err(ContractError::Blacklisted);
        }

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;

        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }
        if mark_milestone_expired_if_needed(&env, grant_id, milestone_idx, &mut milestone)? {
            return Err(ContractError::DeadlinePassed);
        }

        if milestone.state() == MilestoneState::CommunityReview {
            if env.ledger().timestamp() < milestone.submission_timestamp + COMMUNITY_REVIEW_PERIOD {
                return Err(ContractError::CommunityReviewPeriod);
            }
            // Community period has elapsed — transition to Submitted so voting proceeds.
            milestone.set_state(MilestoneState::Submitted);
        } else if milestone.state() != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }
        if milestone.state() == MilestoneState::Disputed {
            return Err(ContractError::InvalidState);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }
        access::require_optional_role(&env, &reviewer, Role::Reviewer)?;

        // Issue #164: enforce MinReviewerStake — reviewer must have staked at least
        // the global minimum before they can cast a vote on any milestone.
        let min_stake = Storage::get_min_reviewer_stake(&env);
        if min_stake > 0 {
            let reviewer_stake = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            if reviewer_stake < min_stake {
                return Err(ContractError::InsufficientStake);
            }
        }

        // Duplicate-vote guard: return error if reviewer already voted
        if milestone.votes.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        if let Some(ref fb) = feedback {
            if fb.len() > 256 {
                return Err(ContractError::InvalidInput);
            }
            milestone.reasons.set(reviewer.clone(), fb.clone());
        }

        let reputation = Storage::get_reviewer_reputation(&env, reviewer.clone());
        milestone.votes.set(reviewer.clone(), approve);

        if approve {
            milestone.set_approvals(milestone.approvals() + reputation);
        } else {
            milestone.set_rejections(milestone.rejections() + reputation);
        }

        let quorum_reached = milestone.approvals() >= grant.quorum();
        if quorum_reached {
            // Emit QuorumReached event
            Events::emit_quorum_reached(
                &env,
                grant_id,
                milestone_idx,
                milestone.approvals(),
                grant.quorum(),
            );

            // Reward harmonious voters who voted approve
            for (voter, voted_approve) in milestone.votes.iter() {
                if voted_approve {
                    let mut rep = Storage::get_reviewer_reputation(&env, voter.clone());
                    rep += 1;
                    Storage::set_reviewer_reputation(&env, voter.clone(), rep);
                }
            }

            // ----- Milestone approved, awaiting challenge period -----
            milestone.set_state(MilestoneState::AwaitingPayout);
            milestone.status_updated_at = env.ledger().timestamp();
            Events::milestone_status_changed(
                &env,
                grant_id,
                milestone_idx,
                MilestoneState::AwaitingPayout,
            );
        }

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        // Enhanced event emission: include all relevant data, standardize topics
        Events::milestone_voted(
            &env,
            grant_id,
            milestone_idx,
            reviewer.clone(),
            approve,
            feedback.clone(),
        );

        Ok(quorum_reached)
    }
    pub fn milestone_reject(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        reason: String,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();
        assert_not_paused(&env)?;

        if reason.len() > 256 {
            return Err(ContractError::InvalidInput);
        }

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }
        if mark_milestone_expired_if_needed(&env, grant_id, milestone_idx, &mut milestone)? {
            return Err(ContractError::DeadlinePassed);
        }

        if milestone.state() == MilestoneState::CommunityReview {
            if env.ledger().timestamp() < milestone.submission_timestamp + COMMUNITY_REVIEW_PERIOD {
                return Err(ContractError::CommunityReviewPeriod);
            }
            milestone.set_state(MilestoneState::Submitted);
        } else if milestone.state() != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }
        access::require_optional_role(&env, &reviewer, Role::Reviewer)?;

        if milestone.votes.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        let reputation = Storage::get_reviewer_reputation(&env, reviewer.clone());
        milestone.votes.set(reviewer.clone(), false);
        milestone.set_rejections(milestone.rejections() + reputation);
        milestone.reasons.set(reviewer.clone(), reason.clone());

        let majority_rejected = milestone.rejections() >= grant.quorum();

        if majority_rejected {
            milestone.set_state(MilestoneState::Rejected);
            milestone.status_updated_at = env.ledger().timestamp();

            // Reward harmonious voters who voted reject
            for (voter, voted_approve) in milestone.votes.iter() {
                if !voted_approve {
                    let mut rep = Storage::get_reviewer_reputation(&env, voter.clone());
                    rep += 1;
                    Storage::set_reviewer_reputation(&env, voter.clone(), rep);
                }
            }

            Events::milestone_status_changed(
                &env,
                grant_id,
                milestone_idx,
                MilestoneState::Rejected,
            );
        }

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_rejected(&env, grant_id, milestone_idx, reviewer, reason);

        Ok(majority_rejected)
    }
    pub fn milestone_dispute(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        recipient: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        let _reason = reason;
        recipient.require_auth();

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.owner != recipient {
            return Err(ContractError::Unauthorized);
        }

        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        if milestone.state() != MilestoneState::Rejected {
            return Err(ContractError::InvalidState);
        }

        milestone.set_state(MilestoneState::Disputed);
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Disputed);
        Ok(())
    }
    pub fn milestone_resolve_dispute(
        env: Env,
        council: Address,
        grant_id: u64,
        milestone_idx: u32,
        approve: bool,
    ) -> Result<(), ContractError> {
        council.require_auth();

        let council_addr = Storage::get_council(&env).ok_or(ContractError::InvalidInput)?;
        if council_addr != council {
            return Err(ContractError::Unauthorized);
        }

        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        if milestone.state() != MilestoneState::Disputed
            && milestone.state() != MilestoneState::Challenged
        {
            return Err(ContractError::InvalidState);
        }

        if milestone.state() == MilestoneState::Disputed {
            milestone.set_state(if approve {
                MilestoneState::Approved
            } else {
                MilestoneState::Rejected
            });
        } else {
            // Milestone is Challenged
            milestone.set_state(if approve {
                MilestoneState::AwaitingPayout // Owner wins, resume to AwaitingPayout
            } else {
                MilestoneState::Rejected // Funder wins, reject the milestone
            });
        }
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::milestone_status_changed(&env, grant_id, milestone_idx, milestone.state());

        Ok(())
    }
    pub fn milestone_submit(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        recipient: Address,
        description: String,
        proof_url: String,
        payout_token: Option<Address>, // New parameter
    ) -> Result<(), ContractError> {
        recipient.require_auth();
        assert_not_paused(&env)?;

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        check_heartbeat(&env, &mut grant);

        if grant.status() == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        if grant.owner != recipient {
            return Err(ContractError::Unauthorized);
        }

        ensure_min_reputation_for_grant(&env, grant_id, recipient.clone())?;

        apply_milestone_submission(
            &env,
            grant_id,
            &grant,
            milestone_idx,
            description,
            proof_url,
            payout_token,
        )
    }
    pub fn milestone_submit_batch(
        env: Env,
        grant_id: u64,
        recipient: Address,
        submissions: Vec<MilestoneSubmission>,
    ) -> Result<(), ContractError> {
        recipient.require_auth();

        let batch_len = submissions.len();
        if batch_len == 0 {
            return Err(ContractError::BatchEmpty);
        }
        if batch_len > 20 {
            return Err(ContractError::BatchTooLarge);
        }

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.status() == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        if grant.owner != recipient {
            return Err(ContractError::Unauthorized);
        }

        ensure_min_reputation_for_grant(&env, grant_id, recipient.clone())?;

        for sub in submissions.iter() {
            apply_milestone_submission(
                &env,
                grant_id,
                &grant,
                sub.idx,
                sub.description.clone(),
                sub.proof.clone(),
                sub.payout_token.clone(),
            )?;
        }

        Ok(())
    }
    pub fn grant_fund(
        env: Env,
        grant_id: u64,
        funder: Address,
        amount: i128,
        token: Address, // New parameter
        memo: Option<String>,
    ) -> Result<(), ContractError> {
        funder.require_auth();
        assert_not_paused(&env)?;
        reentrancy::with_non_reentrant(&env, || {
            if amount <= 0 {
                return Err(ContractError::InvalidInput);
            }

            let mut grant =
                Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            check_heartbeat(&env, &mut grant);

            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status() != GrantStatus::Active
                && grant.status() != GrantStatus::PendingFunding
            {
                return Err(ContractError::InvalidState);
            }

            // Perform the token transfer from the funder to the contract
            let token_client = token::Client::new(&env, &token);
            let contract_address = env.current_contract_address();
            token_client.transfer(&funder, &contract_address, &amount);

            let current_balance = grant.escrow_balances.get(token.clone()).unwrap_or(0);
            let new_balance = current_balance
                .checked_add(amount)
                .ok_or(ContractError::InvalidInput)?;
            if grant.hard_cap > 0 && token == grant.primary_token && new_balance > grant.hard_cap {
                return Err(ContractError::CapReached);
            }

            grant.escrow_balances.set(token.clone(), new_balance);

            // Update funds tracking (per token)
            let mut fund_entry_found = false;
            for i in 0..grant.funders.len() {
                let mut fund_entry = funder_entry_at(&grant.funders, i)?;
                if fund_entry.funder == funder && fund_entry.token == token {
                    fund_entry.amount = fund_entry
                        .amount
                        .checked_add(amount)
                        .ok_or(ContractError::InvalidInput)?;
                    grant.funders.set(i, fund_entry);
                    fund_entry_found = true;
                    break;
                }
            }

            if !fund_entry_found {
                grant.funders.push_back(GrantFund {
                    funder: funder.clone(),
                    amount,
                    token: token.clone(),
                });
            }

            // Auto-transition PendingFunding → Active once threshold is met (based on primary token)
            let primary_balance = grant
                .escrow_balances
                .get(grant.primary_token.clone())
                .unwrap_or(0);
            if grant.status() == GrantStatus::PendingFunding && primary_balance >= grant.min_funding
            {
                grant.set_status(GrantStatus::Active);
                Storage::index_transition(
                    &env,
                    GrantStatus::PendingFunding as u32,
                    GrantStatus::Active as u32,
                    grant_id,
                );
                Events::emit_grant_activated(&env, grant.id);
            }

            Storage::set_grant(&env, grant_id, &grant);

            // Enhanced event emission: include all relevant data, standardize topics
            Events::emit_grant_funded(
                &env,
                grant_id,
                funder.clone(),
                amount,
                token.clone(),
                new_balance,
            );
            Events::emit_payer_receipt(&env, grant_id, funder, amount, token, None, memo);

            Ok(())
        })
    }
    pub fn milestone_upvote(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: Address,
    ) -> Result<(), ContractError> {
        voter.require_auth();

        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        if mark_milestone_expired_if_needed(&env, grant_id, milestone_idx, &mut milestone)? {
            return Err(ContractError::DeadlinePassed);
        }

        if milestone.state() != MilestoneState::CommunityReview {
            return Err(ContractError::InvalidState);
        }
        if Storage::has_milestone_upvote(&env, grant_id, milestone_idx, &voter) {
            return Err(ContractError::AlreadyUpvoted);
        }

        Storage::set_milestone_upvote(&env, grant_id, milestone_idx, &voter);
        milestone.set_community_upvotes(milestone.community_upvotes() + 1);
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        // Enhanced event emission: include all relevant data, standardize topics
        Events::emit_milestone_upvoted(
            &env,
            grant_id,
            milestone_idx,
            voter.clone(),
            milestone.community_upvotes(),
        );
        Ok(())
    }
    pub fn milestone_comment(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: Address,
        comment: String,
    ) -> Result<(), ContractError> {
        voter.require_auth();

        if comment.len() > 512 {
            return Err(ContractError::InvalidInput);
        }

        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;
        if mark_milestone_expired_if_needed(&env, grant_id, milestone_idx, &mut milestone)? {
            return Err(ContractError::DeadlinePassed);
        }

        if milestone.state() != MilestoneState::CommunityReview {
            return Err(ContractError::InvalidState);
        }

        milestone
            .community_comments
            .set(voter.clone(), comment.clone());
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::emit_milestone_commented(&env, grant_id, milestone_idx, voter, comment);
        Ok(())
    }
    pub fn grant_add_reviewer(
        env: Env,
        grant_id: u64,
        owner: Address,
        new_reviewer: Address,
    ) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }
        if grant.reviewers.contains(new_reviewer.clone()) {
            return Err(ContractError::InvalidInput);
        }
        access::require_optional_role(&env, &new_reviewer, Role::Reviewer)?;

        grant.reviewers.push_back(new_reviewer.clone());
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_reviewer_added(&env, grant_id, owner, new_reviewer);
        Ok(())
    }
    pub fn grant_remove_reviewer(
        env: Env,
        grant_id: u64,
        owner: Address,
        old_reviewer: Address,
    ) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        // Must have more than 1 reviewer to allow removal
        if grant.reviewers.len() <= 1 {
            return Err(ContractError::InvalidInput);
        }

        // Find and remove the reviewer
        let mut new_reviewers = soroban_sdk::Vec::new(&env);
        let mut found = false;
        for r in grant.reviewers.iter() {
            if r == old_reviewer {
                found = true;
            } else {
                new_reviewers.push_back(r);
            }
        }

        if !found {
            return Err(ContractError::Unauthorized);
        }

        // Ensure quorum does not exceed the new reviewer count
        if grant.quorum() > new_reviewers.len() {
            return Err(ContractError::InvalidInput);
        }

        grant.reviewers = new_reviewers;
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_reviewer_removed(&env, grant_id, owner, old_reviewer);
        Ok(())
    }
    pub fn grant_pause(env: Env, grant_id: u64, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        let is_owner = grant.owner == caller;
        let is_admin = is_admin_actor(&env, &caller);
        if !is_owner && !is_admin {
            return Err(ContractError::Unauthorized);
        }
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        grant.set_status(GrantStatus::Paused);
        Storage::set_grant(&env, grant_id, &grant);
        Storage::index_transition(
            &env,
            GrantStatus::Active as u32,
            GrantStatus::Paused as u32,
            grant_id,
        );
        Events::emit_grant_paused(&env, grant_id, caller);
        Ok(())
    }
    pub fn grant_resume(env: Env, grant_id: u64, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        let is_owner = grant.owner == caller;
        let is_admin = is_admin_actor(&env, &caller);
        if !is_owner && !is_admin {
            return Err(ContractError::Unauthorized);
        }
        if grant.status() != GrantStatus::Paused {
            return Err(ContractError::InvalidState);
        }

        grant.set_status(GrantStatus::Active);
        Storage::set_grant(&env, grant_id, &grant);
        Storage::index_transition(
            &env,
            GrantStatus::Paused as u32,
            GrantStatus::Active as u32,
            grant_id,
        );
        Events::emit_grant_resumed(&env, grant_id, caller);
        Ok(())
    }
    pub fn get_grant(env: Env, grant_id: u64) -> Result<Grant, ContractError> {
        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)
    }
    pub fn get_contributor_profile(
        env: Env,
        contributor: Address,
    ) -> Option<crate::types::ContributorProfile> {
        Storage::get_contributor(&env, contributor)
    }
    pub fn get_grants_by_status(
        env: Env,
        status: GrantStatus,
        page: u32,
        page_size: u32,
    ) -> Vec<u64> {
        let page_size = if page_size == 0 || page_size > 50 {
            50
        } else {
            page_size
        };
        let ids = Storage::get_status_index(&env, status as u32);
        let total = ids.len();
        let start = page * page_size;
        if start >= total {
            return Vec::new(&env);
        }
        let end = (start + page_size).min(total);
        let mut result = Vec::new(&env);
        for i in start..end {
            if let Some(grant_id) = ids.get(i) {
                result.push_back(grant_id);
            }
        }
        result
    }

    pub fn get_milestone(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<Milestone, ContractError> {
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if milestone_idx >= grant.total_milestones() {
            return Err(ContractError::InvalidInput);
        }

        Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)
    }
    pub fn get_milestone_feedback(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<soroban_sdk::Map<Address, String>, ContractError> {
        let milestone = Self::get_milestone(env, grant_id, milestone_idx)?;
        Ok(milestone.reasons)
    }

    // ── Reviewer Staking (#42) ──────────────────────────────────────
    pub fn set_staking_config(
        env: Env,
        admin: Address,
        min_stake: i128,
        treasury: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        if min_stake <= 0 {
            return Err(ContractError::InvalidInput);
        }
        env.storage()
            .persistent()
            .set(&storage::DataKey::MinReviewerStake, &min_stake);
        env.storage()
            .persistent()
            .set(&storage::DataKey::Treasury, &treasury);
        Ok(())
    }
    pub fn stake_to_review(
        env: Env,
        reviewer: Address,
        grant_id: u64,
        amount: i128,
    ) -> Result<(), ContractError> {
        reviewer.require_auth();

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status() == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status() != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let min_stake = Storage::get_min_reviewer_stake(&env);
            if amount < min_stake {
                return Err(ContractError::InsufficientStake);
            }

            let contract_addr = env.current_contract_address();
            let client = token::Client::new(&env, &grant.primary_token);
            client.transfer(&reviewer, &contract_addr, &amount);

            let current = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            Storage::set_reviewer_stake(&env, grant_id, &reviewer, current + amount);

            Ok(())
        })
    }
    pub fn slash_reviewer(
        env: Env,
        admin: Address,
        grant_id: u64,
        reviewer: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            let stake = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            if stake <= 0 {
                return Err(ContractError::StakeNotFound);
            }

            let treasury = Storage::get_treasury(&env).ok_or(ContractError::InvalidInput)?;
            let client = token::Client::new(&env, &grant.primary_token);
            client.transfer(&env.current_contract_address(), &treasury, &stake);

            Storage::set_reviewer_stake(&env, grant_id, &reviewer, 0);

            Ok(())
        })
    }
    pub fn unstake(env: Env, reviewer: Address, grant_id: u64) -> Result<(), ContractError> {
        reviewer.require_auth();

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status() == GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let stake = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            if stake <= 0 {
                return Err(ContractError::StakeNotFound);
            }

            let client = token::Client::new(&env, &grant.primary_token);
            client.transfer(&env.current_contract_address(), &reviewer, &stake);

            Storage::set_reviewer_stake(&env, grant_id, &reviewer, 0);

            Ok(())
        })
    }

    // ── KYC Integration (#43) ───────────────────────────────────────
    pub fn set_identity_oracle(
        env: Env,
        admin: Address,
        oracle: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        require_admin_actor(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&storage::DataKey::IdentityOracle, &oracle);
        Ok(())
    }

    // ── Bulk Funding (#44) ──────────────────────────────────────────
    pub fn fund_batch(
        env: Env,
        funder: Address,
        grants: Vec<(u64, i128, Address)>,
    ) -> Result<(), ContractError> {
        funder.require_auth();

        reentrancy::with_non_reentrant(&env, || {
            let batch_len = grants.len();
            if batch_len == 0 {
                return Err(ContractError::BatchEmpty);
            }
            if batch_len > 20 {
                return Err(ContractError::BatchTooLarge);
            }

            for item in grants.iter() {
                let (grant_id, amount, token) = item;
                if amount <= 0 {
                    return Err(ContractError::InvalidInput);
                }

                let mut grant =
                    Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

                check_heartbeat(&env, &mut grant);

                if grant.status() == GrantStatus::Inactive {
                    return Err(ContractError::HeartbeatMissed);
                }
                if grant.status() != GrantStatus::Active
                    && grant.status() != GrantStatus::PendingFunding
                {
                    return Err(ContractError::InvalidState);
                }

                let contract_addr = env.current_contract_address();
                let client = token::Client::new(&env, &token);
                client.transfer(&funder, &contract_addr, &amount);

                let current_balance = grant.escrow_balances.get(token.clone()).unwrap_or(0);
                let new_balance = current_balance
                    .checked_add(amount)
                    .ok_or(ContractError::InvalidInput)?;
                if grant.hard_cap > 0
                    && token == grant.primary_token
                    && new_balance > grant.hard_cap
                {
                    return Err(ContractError::CapReached);
                }
                grant.escrow_balances.set(token.clone(), new_balance);

                let mut found = false;
                for i in 0..grant.funders.len() {
                    let mut fund_entry = funder_entry_at(&grant.funders, i)?;
                    if fund_entry.funder == funder && fund_entry.token == token {
                        fund_entry.amount += amount;
                        grant.funders.set(i, fund_entry);
                        found = true;
                        break;
                    }
                }
                if !found {
                    grant.funders.push_back(GrantFund {
                        funder: funder.clone(),
                        amount,
                        token: token.clone(),
                    });
                }

                // Auto-activate if threshold met
                let primary_balance = grant
                    .escrow_balances
                    .get(grant.primary_token.clone())
                    .unwrap_or(0);
                if grant.status() == GrantStatus::PendingFunding
                    && primary_balance >= grant.min_funding
                {
                    grant.set_status(GrantStatus::Active);
                    Storage::index_transition(
                        &env,
                        GrantStatus::PendingFunding as u32,
                        GrantStatus::Active as u32,
                        grant_id,
                    );
                    Events::emit_grant_activated(&env, grant.id);
                }

                Storage::set_grant(&env, grant_id, &grant);

                Events::emit_grant_funded(
                    &env,
                    grant_id,
                    funder.clone(),
                    amount,
                    token.clone(),
                    new_balance,
                );
                Events::emit_payer_receipt(
                    &env,
                    grant_id,
                    funder.clone(),
                    amount,
                    token,
                    None,
                    None,
                );
            }

            Ok(())
        })
    }
    pub fn grant_ping(env: Env, grant_id: u64, owner: Address) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }

        // Grant must be in a state where pinging makes sense
        if grant.status() != GrantStatus::Active && grant.status() != GrantStatus::Inactive {
            return Err(ContractError::InvalidState);
        }

        let now = env.ledger().timestamp();
        grant.last_heartbeat = now;

        // If it was inactive, restore it to active
        if grant.status() == GrantStatus::Inactive {
            grant.set_status(GrantStatus::Active);
            Storage::index_transition(
                &env,
                GrantStatus::Inactive as u32,
                GrantStatus::Active as u32,
                grant_id,
            );
        }

        Storage::set_grant(&env, grant_id, &grant);
        Events::emit_heartbeat_updated(&env, grant_id, now);

        Ok(())
    }
    pub fn admin_blacklist_add(
        env: Env,
        admin: Address,
        target: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        if require_admin_actor(&env, &admin).is_err() {
            return Err(ContractError::Unauthorized);
        }

        Storage::set_blacklisted(&env, &target);
        Ok(())
    }
    pub fn admin_blacklist_remove(
        env: Env,
        admin: Address,
        target: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        if require_admin_actor(&env, &admin).is_err() {
            return Err(ContractError::Unauthorized);
        }

        Storage::remove_blacklisted(&env, &target);
        Ok(())
    }
    pub fn milestone_payout(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        caller: Address,
    ) -> Result<(), ContractError> {
        caller.require_auth();
        assert_not_paused(&env)?;
        reentrancy::with_non_reentrant(&env, || {
            let mut grant =
                Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status() != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
                .ok_or(ContractError::MilestoneNotFound)?;

            // In older flows resolving dispute might set state to Approved, accept both
            if milestone.state() != MilestoneState::AwaitingPayout
                && milestone.state() != MilestoneState::Approved
            {
                return Err(ContractError::InvalidState);
            }

            if milestone.state() == MilestoneState::AwaitingPayout
                && env.ledger().timestamp() < milestone.status_updated_at + CHALLENGE_PERIOD
            {
                return Err(ContractError::DeadlinePassed);
            }

            let payout_amount = milestone.amount;
            let payout_token = milestone.payout_token.clone();

            // Transfer milestone amount from contract escrow to grant owner
            let token_client = token::Client::new(&env, &payout_token);
            token_client.transfer(
                &env.current_contract_address(),
                &grant.owner,
                &payout_amount,
            );

            // Update escrow accounting
            let current_balance = grant.escrow_balances.get(payout_token.clone()).unwrap_or(0);
            grant.escrow_balances.set(
                payout_token.clone(),
                current_balance
                    .checked_sub(payout_amount)
                    .ok_or(ContractError::InvalidInput)?,
            );
            grant.set_milestones_paid_out(
                grant
                    .milestones_paid_out()
                    .checked_add(1)
                    .ok_or(ContractError::InvalidInput)?,
            );
            Storage::set_grant(&env, grant_id, &grant);

            milestone.set_state(MilestoneState::Paid);
            milestone.status_updated_at = env.ledger().timestamp();

            Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

            Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Paid);
            Events::emit_milestone_paid(
                &env,
                grant_id,
                milestone_idx,
                payout_amount,
                payout_token.clone(),
            );
            Events::emit_payout_executed(
                &env,
                grant_id,
                grant.owner.clone(),
                payout_amount,
                payout_token.clone(),
            );
            Events::emit_payee_receipt(
                &env,
                grant_id,
                grant.owner.clone(),
                payout_token.clone(),
                payout_amount,
                Some(milestone_idx),
            );

            // Reuse the shared idempotent reputation path for direct payouts too.
            Self::update_contributor_reputation(
                &env,
                grant_id,
                milestone_idx,
                &grant.owner,
                payout_amount,
            );

            Ok(())
        })
    }
    pub fn milestone_challenge(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        funder: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        funder.require_auth();
        assert_not_paused(&env)?;

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.status() != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        if !grant_has_funder(&grant, &funder)? {
            return Err(ContractError::Unauthorized);
        }

        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)?;

        if milestone.state() != MilestoneState::AwaitingPayout {
            return Err(ContractError::InvalidState);
        }

        milestone.set_state(MilestoneState::Challenged);
        milestone.status_updated_at = env.ledger().timestamp();

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Challenged);
        Events::milestone_challenged(&env, grant_id, milestone_idx, funder, reason);

        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────
    fn update_contributor_reputation(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        contributor: &Address,
        payout_amount: i128,
    ) {
        // Guard: apply at most once per milestone
        if Storage::has_milestone_reputation_applied(env, grant_id, milestone_idx) {
            return;
        }
        Storage::mark_milestone_reputation_applied(env, grant_id, milestone_idx);

        let reputation_gain: u64 = 10;

        let mut profile = match Storage::get_contributor(env, contributor.clone()) {
            Some(p) => p,
            None => {
                // Contributor has not registered a profile — skip silently as
                // the issue specifies this as an acceptable fallback.
                return;
            }
        };

        profile.reputation_score = profile.reputation_score.saturating_add(reputation_gain);
        profile.total_earned = profile.total_earned.saturating_add(payout_amount);

        Storage::set_contributor(env, contributor.clone(), &profile);

        Events::emit_reputation_updated(
            env,
            grant_id,
            milestone_idx,
            contributor.clone(),
            profile.reputation_score,
            profile.total_earned,
        );
    }
}

fn assert_not_paused(env: &Env) -> Result<(), ContractError> {
    if Storage::is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

fn check_heartbeat(env: &Env, grant: &mut Grant) {
    if grant.status() != GrantStatus::Active {
        return;
    }

    let now = env.ledger().timestamp();
    let seconds_since_heartbeat = now.saturating_sub(grant.last_heartbeat);

    // 30 days = 30 * 24 * 60 * 60 = 2,592,000 seconds
    if seconds_since_heartbeat > 30 * 24 * 60 * 60 {
        grant.set_status(GrantStatus::Inactive);
        Storage::set_grant(env, grant.id, grant);
        Storage::index_transition(
            env,
            GrantStatus::Active as u32,
            GrantStatus::Inactive as u32,
            grant.id,
        );
        Events::emit_grant_gone_inactive(env, grant.id, now);
    }
}

fn is_admin_actor(env: &Env, address: &Address) -> bool {
    Storage::get_global_admin(env) == Some(address.clone())
        || access::has_role(env, address, Role::Admin)
}

fn require_admin_actor(env: &Env, address: &Address) -> Result<(), ContractError> {
    if is_admin_actor(env, address) {
        return Ok(());
    }
    Err(ContractError::NotContractAdmin)
}

fn funder_entry_at(funders: &Vec<GrantFund>, index: u32) -> Result<GrantFund, ContractError> {
    funders.get(index).ok_or(ContractError::InvalidInput)
}

fn grant_has_funder(grant: &Grant, address: &Address) -> Result<bool, ContractError> {
    for i in 0..grant.funders.len() {
        let fund_entry = funder_entry_at(&grant.funders, i)?;
        if fund_entry.funder == *address {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_token_funders(funders: &Vec<GrantFund>, token: &Address) -> bool {
    for fund_entry in funders.iter() {
        if fund_entry.token == *token {
            return true;
        }
    }
    false
}

fn refund_token_to_funders(
    env: &Env,
    grant_id: u64,
    funders: &Vec<GrantFund>,
    token: &Address,
    refundable_amount: i128,
) -> Result<(), ContractError> {
    let mut total_token_contributions: i128 = 0;
    let mut token_funders = soroban_sdk::Vec::new(env);
    for fund_entry in funders.iter() {
        if fund_entry.token == *token {
            total_token_contributions += fund_entry.amount;
            token_funders.push_back(fund_entry);
        }
    }

    if total_token_contributions == 0 {
        return Ok(());
    }

    let token_client = token::Client::new(env, token);
    let token_funders_len = token_funders.len();
    let mut distributed = 0i128;

    for i in 0..token_funders_len {
        let fund_entry = funder_entry_at(&token_funders, i)?;
        let is_last = i + 1 == token_funders_len;
        let refund_amount = if is_last {
            refundable_amount - distributed
        } else {
            let amount = fund_entry
                .amount
                .checked_mul(refundable_amount)
                .ok_or(ContractError::InvalidInput)?
                .checked_div(total_token_contributions)
                .ok_or(ContractError::InvalidInput)?;
            distributed += amount;
            amount
        };

        if refund_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &fund_entry.funder,
                &refund_amount,
            );
            Events::emit_refund_issued(
                env,
                grant_id,
                fund_entry.funder.clone(),
                refund_amount,
                token.clone(),
            );
        }
    }

    Ok(())
}

fn is_milestone_deadline_elapsed(env: &Env, milestone: &Milestone) -> bool {
    milestone.deadline_timestamp > 0 && env.ledger().timestamp() > milestone.deadline_timestamp
}

fn milestone_can_expire(milestone: &Milestone) -> bool {
    matches!(
        milestone.state(),
        MilestoneState::Pending
            | MilestoneState::CommunityReview
            | MilestoneState::Submitted
            | MilestoneState::Rejected
    )
}

fn is_milestone_expired(env: &Env, milestone: &Milestone) -> bool {
    milestone.state() == MilestoneState::Expired
        || milestone.state() == MilestoneState::ExpiredClaimed
        || (milestone_can_expire(milestone) && is_milestone_deadline_elapsed(env, milestone))
}

fn milestone_allows_extension(milestone: &Milestone) -> bool {
    matches!(
        milestone.state(),
        MilestoneState::Pending
            | MilestoneState::CommunityReview
            | MilestoneState::Submitted
            | MilestoneState::Rejected
    )
}

fn mark_milestone_expired_if_needed(
    env: &Env,
    grant_id: u64,
    milestone_idx: u32,
    milestone: &mut Milestone,
) -> Result<bool, ContractError> {
    if milestone.state() == MilestoneState::Expired
        || milestone.state() == MilestoneState::ExpiredClaimed
    {
        return Ok(true);
    }
    if !milestone_can_expire(milestone) || !is_milestone_deadline_elapsed(env, milestone) {
        return Ok(false);
    }

    milestone.set_state(MilestoneState::Expired);
    milestone.status_updated_at = env.ledger().timestamp();
    Storage::set_milestone(env, grant_id, milestone_idx, milestone);
    Storage::remove_extension_request(env, grant_id, milestone_idx);

    Events::milestone_status_changed(env, grant_id, milestone_idx, MilestoneState::Expired);
    Events::emit_milestone_expired(env, grant_id, milestone_idx);
    Ok(true)
}

fn apply_milestone_submission(
    env: &Env,
    grant_id: u64,
    grant: &Grant,
    milestone_idx: u32,
    description: String,
    proof_url: String,
    payout_token: Option<Address>,
) -> Result<(), ContractError> {
    if milestone_idx >= grant.total_milestones() {
        return Err(ContractError::InvalidInput);
    }

    let mut milestone = Storage::get_milestone(env, grant_id, milestone_idx)
        .ok_or(ContractError::MilestoneNotFound)?;
    if mark_milestone_expired_if_needed(env, grant_id, milestone_idx, &mut milestone)? {
        return Err(ContractError::DeadlinePassed);
    }

    if milestone.state() == MilestoneState::CommunityReview
        || milestone.state() == MilestoneState::Submitted
        || milestone.state() == MilestoneState::Approved
        || milestone.state() == MilestoneState::Paid
        || milestone.state() == MilestoneState::AwaitingPayout
    {
        return Err(ContractError::MilestoneAlreadySubmitted);
    }
    if milestone.state() == MilestoneState::Expired
        || milestone.state() == MilestoneState::ExpiredClaimed
    {
        return Err(ContractError::DeadlinePassed);
    }

    milestone.description = description.clone();
    // Milestone enters the community review window before official voting opens.
    milestone.set_state(MilestoneState::CommunityReview);
    milestone.proof_url = Some(proof_url);
    if let Some(token) = payout_token {
        milestone.payout_token = token;
    }
    milestone.submission_timestamp = env.ledger().timestamp();

    Storage::set_milestone(env, grant_id, milestone_idx, &milestone);
    Events::emit_milestone_submitted(env, grant_id, milestone_idx, description);

    Ok(())
}

fn ensure_min_reputation_for_grant(
    env: &Env,
    grant_id: u64,
    contributor: Address,
) -> Result<(), ContractError> {
    let min_reputation = Storage::get_grant_min_reputation(env, grant_id);
    if min_reputation == 0 {
        return Ok(());
    }

    let profile = Storage::get_contributor(env, contributor).ok_or(ContractError::Unauthorized)?;
    if profile.reputation_score < min_reputation {
        return Err(ContractError::InsufficientReputation);
    }

    Ok(())
}

#[cfg(test)]
mod test;
