#![no_std]
#![allow(clippy::too_many_arguments)]

/// View: Get milestone by grant_id and milestone_idx
pub fn get_milestone(env: Env, grant_id: u64, milestone_idx: u32) -> Option<Milestone> {
    Storage::get_milestone(&env, grant_id, milestone_idx)
}
mod events;
/// Token-transfer reentrancy guard (lock/unlock on transient storage). See `reentrancy` module.
mod reentrancy;
mod storage;
mod types;

pub use events::Events;
pub use storage::Storage;
pub use types::{
    ContractError, EscrowLifecycleState, EscrowMode, EscrowState, Grant, GrantFund, GrantStatus,
    Milestone, MilestoneState, MilestoneSubmission,
};

use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

/// Community review window (3 days in seconds) that must elapse after milestone
/// submission before official reviewer voting is allowed.
pub const COMMUNITY_REVIEW_PERIOD: u64 = 3 * 24 * 60 * 60;

/// Grace period (7 days in seconds) applied when a cancellation is requested
/// while one or more milestones are still in a submitted/review state.
pub const CANCEL_GRACE_PERIOD: u64 = 7 * 24 * 60 * 60;

#[contract]
pub struct StellarGrantsContract;

#[contractimpl]
impl StellarGrantsContract {
    /// Initiate a dispute on a milestone. Callable by grant owner, reviewers, or contributor.
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
        // Only grant owner, reviewers, or contributor can dispute
        let is_reviewer = grant.reviewers.contains(caller.clone());
        let is_owner = grant.owner == caller;
        // For now, assume contributor is grant.owner (can be extended)
        if !(is_owner || is_reviewer) {
            return Err(ContractError::Unauthorized);
        }
        if milestone.state != MilestoneState::Submitted
            && milestone.state != MilestoneState::Approved
        {
            return Err(ContractError::InvalidState);
        }
        milestone.state = MilestoneState::Disputed;
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Disputed);
        Ok(())
    }

    /// Committee resolves a disputed milestone. Only callable by council.
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
        if milestone.state != MilestoneState::Disputed {
            return Err(ContractError::InvalidState);
        }
        milestone.state = MilestoneState::Resolved;
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Resolved);

        // Fetch grant for payout/refund
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let token_client = token::Client::new(&env, &grant.token);
        if approve {
            // Approve: payout milestone amount to grant owner (contributor)
            if grant.escrow_balance < grant.milestone_amount {
                return Err(ContractError::InvalidInput);
            }
            token_client.transfer(
                &env.current_contract_address(),
                &grant.owner,
                &grant.milestone_amount,
            );
            grant.escrow_balance -= grant.milestone_amount;
            grant.milestones_paid_out += 1;
            Storage::set_grant(&env, grant_id, &grant);
            Events::emit_milestone_paid(&env, grant_id, milestone_idx, grant.milestone_amount);
        } else {
            // Reject: refund milestone amount to funders (pro-rata)
            let total_refundable = grant.milestone_amount;
            let mut total_contributions: i128 = 0;
            for fund_entry in grant.funders.iter() {
                total_contributions += fund_entry.amount;
            }
            if total_contributions <= 0 {
                return Err(ContractError::InvalidInput);
            }
            let funders_len = grant.funders.len();
            let mut distributed = 0i128;
            for i in 0..funders_len {
                let fund_entry = grant.funders.get(i).unwrap();
                let is_last = i + 1 == funders_len;
                let refund_amount = if is_last {
                    total_refundable - distributed
                } else {
                    let amount = fund_entry
                        .amount
                        .checked_mul(total_refundable)
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
                    Events::emit_refund_issued(
                        &env,
                        grant_id,
                        fund_entry.funder.clone(),
                        refund_amount,
                    );
                }
            }
            grant.escrow_balance -= total_refundable;
            Storage::set_grant(&env, grant_id, &grant);
        }
        Ok(())
    }

    /// Initialize the contract with a global admin and council for dispute resolution.
    ///
    /// # Arguments
    /// * `admin` - Contract-wide administrator (upgrades, council rotation, staking config, etc.).
    /// * `council` - Address of DAO Council or arbitration authority.
    ///
    /// # Errors
    /// * [`ContractError::InvalidInput`] if already initialized.
    pub fn initialize(env: Env, admin: Address, council: Address) -> Result<(), ContractError> {
        if Storage::get_global_admin(&env).is_some() {
            return Err(ContractError::InvalidInput);
        }
        admin.require_auth();
        Storage::set_global_admin(&env, &admin);
        Storage::set_council(&env, &council);
        Events::emit_contract_initialized(&env, council);
        Ok(())
    }

    /// Rotate the contract admin. Only `old_admin` may call; must match stored admin.
    pub fn admin_change(
        env: Env,
        old_admin: Address,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        old_admin.require_auth();
        let current = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if current != old_admin {
            return Err(ContractError::NotContractAdmin);
        }
        Storage::set_global_admin(&env, &new_admin);
        Events::emit_contract_upgraded(&env, old_admin, String::from_str(&env, "admin_changed"));
        Ok(())
    }

    /// Set or rotate the DAO Council address for milestone disputes.
    pub fn set_council(env: Env, caller: Address, council: Address) -> Result<(), ContractError> {
        caller.require_auth();
        let admin = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if admin != caller {
            return Err(ContractError::NotContractAdmin);
        }
        Storage::set_council(&env, &council);
        Events::emit_contract_upgraded(&env, caller, String::from_str(&env, "council_updated"));
        Ok(())
    }

    /// Allows a grant developer/owner to create a new milestone-based grant.
    ///
    /// # Arguments
    /// * `grant_id` - Grant identifier to update.
    /// * `owner` - Grant owner requesting update.
    /// * `new_title` - New grant title.
    /// * `new_description` - New grant description.
    ///
    /// # Returns
    /// * `Ok(())` on success.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`], [`ContractError::Unauthorized`], [`ContractError::InvalidState`].
    ///
    /// # Side Effects
    /// * Updates grant title and description in storage.
    /// * Emits `GrantMetadataUpdated` event.
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
        if grant.status == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        grant.title = new_title.clone();
        grant.description = new_description.clone();
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_grant_metadata_updated(&env, grant_id, owner, new_title, new_description);
        Ok(())
    }

    /// Allows a grant developer/owner to create a new milestone-based grant.
    ///
    /// # Arguments
    /// * `owner` - The address of the grant owner.
    /// * `title` - The title of the grant.
    /// * `description` - The description of the grant.
    /// * `token` - The underlying token for funding the grant.
    /// * `total_amount` - The total amount to be raised.
    /// * `milestone_amount` - The payout chunk for each milestone.
    /// * `num_milestones` - The number of milestones (up to 100).
    /// * `reviewers` - A list of addresses authorized to approve/reject milestones.
    ///
    /// # Errors
    /// * [`ContractError::InvalidInput`] – if validation of amounts or milestones fails.
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
    ) -> Result<u64, ContractError> {
        owner.require_auth();

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

        let grant_id = Storage::increment_grant_counter(&env);

        let grant = Grant {
            id: grant_id,
            owner: owner.clone(),
            title: title.clone(),
            description,
            token,
            status: GrantStatus::Active,
            total_amount,
            milestone_amount,
            reviewers,
            quorum,
            total_milestones: num_milestones,
            milestones_paid_out: 0,
            escrow_balance: 0,
            funders: soroban_sdk::Vec::new(&env),
            reason: None,
            timestamp: env.ledger().timestamp(),
            last_heartbeat: env.ledger().timestamp(),
            cancellation_requested_at: None,
        };

        Storage::set_grant(&env, grant_id, &grant);
        Storage::set_grant_min_reputation(&env, grant_id, 0);
        Storage::set_escrow_state(
            &env,
            grant_id,
            &EscrowState {
                mode: EscrowMode::Standard,
                lifecycle: EscrowLifecycleState::Funding,
                quorum_ready: false,
                approvals_count: 0,
            },
        );
        Storage::set_multisig_signers(&env, grant_id, &soroban_sdk::Vec::new(&env));

        for i in 0..num_milestones {
            let deadline = if let Some(ref deadlines) = milestone_deadlines {
                deadlines.get(i).unwrap_or(0)
            } else {
                0
            };

            let milestone = Milestone {
                idx: i,
                description: String::from_str(&env, ""),
                amount: milestone_amount,
                state: MilestoneState::Pending,
                votes: soroban_sdk::Map::new(&env),
                approvals: 0,
                rejections: 0,
                reasons: soroban_sdk::Map::new(&env),
                status_updated_at: 0,
                proof_url: None,
                submission_timestamp: 0,
                deadline,
                community_upvotes: 0,
                community_comments: soroban_sdk::Map::new(&env),
            };
            Storage::set_milestone(&env, grant_id, i, &milestone);
        }

        Events::emit_grant_created(&env, grant_id, owner, title, total_amount);

        Ok(grant_id)
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
        )?;
        Storage::set_grant_min_reputation(&env, grant_id, min_reputation_score);
        Ok(grant_id)
    }

    /// Create a high-security grant that requires multisig final release.
    ///
    /// # Arguments
    /// * `owner` - Grant owner address.
    /// * `title` - Grant title.
    /// * `description` - Grant description.
    /// * `token` - Token address used for funding and payouts.
    /// * `total_amount` - Total amount requested for the grant.
    /// * `milestone_amount` - Per-milestone payout amount.
    /// * `num_milestones` - Number of milestones to support.
    /// * `reviewers` - Reviewer addresses for milestone votes.
    /// * `multisig_signers` - Required addresses for release approval.
    ///
    /// # Returns
    /// * `Ok(grant_id)` on successful creation.
    ///
    /// # Errors
    /// * [`ContractError::InvalidInput`] when `multisig_signers` is empty or if underlying creation fails.
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
        )?;

        Storage::set_escrow_state(
            &env,
            grant_id,
            &EscrowState {
                mode: EscrowMode::HighSecurity,
                lifecycle: EscrowLifecycleState::Funding,
                quorum_ready: false,
                approvals_count: 0,
            },
        );
        Storage::set_multisig_signers(&env, grant_id, &multisig_signers);

        Ok(grant_id)
    }

    /// Register a contributor profile on-chain
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

        Ok(())
    }

    /// Cancel a grant and refund remaining balance to funders
    pub fn grant_cancel(
        env: Env,
        grant_id: u64,
        owner: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        Self::cancel_grant(env, grant_id, owner, reason)
    }

    /// Cancel a grant and refund escrowed funds. Callable by grant owner or global admin.
    ///
    /// If any milestone is currently in [`MilestoneState::CommunityReview`] or
    /// [`MilestoneState::Submitted`] the first call transitions the grant to
    /// [`GrantStatus::CancellationPending`] and starts a 7-day grace period so
    /// reviewers can finish their work. Call again after the grace period to
    /// execute the actual refund.
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
            let caller_is_admin = Storage::get_global_admin(&env) == Some(caller.clone());
            let grant_is_inactive = grant.status == GrantStatus::Inactive;
            let caller_is_funder = grant.funders.iter().any(|f| f.funder == caller);

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

            match grant.status {
                GrantStatus::Active => {
                    // Check whether any milestone is still actively under review.
                    let mut has_active_submission = false;
                    for milestone_idx in 0..grant.total_milestones {
                        if let Some(m) = Storage::get_milestone(&env, grant_id, milestone_idx) {
                            if m.state == MilestoneState::Submitted
                                || m.state == MilestoneState::CommunityReview
                            {
                                has_active_submission = true;
                                break;
                            }
                        }
                    }

                    if has_active_submission {
                        // Deferred cancellation — start grace period.
                        let executable_after = env.ledger().timestamp() + CANCEL_GRACE_PERIOD;
                        grant.status = GrantStatus::CancellationPending;
                        grant.cancellation_requested_at = Some(env.ledger().timestamp());
                        grant.reason = Some(reason.clone());
                        Storage::set_grant(&env, grant_id, &grant);
                        Events::emit_grant_cancellation_requested(
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
            if grant.milestones_paid_out >= grant.total_milestones {
                return Err(ContractError::InvalidState);
            }

            let total_refundable = grant.escrow_balance;
            if total_refundable > 0 {
                let mut total_contributions: i128 = 0;
                for fund_entry in grant.funders.iter() {
                    total_contributions += fund_entry.amount;
                }

                if total_contributions <= 0 {
                    return Err(ContractError::InvalidInput);
                }

                let token_client = token::Client::new(&env, &grant.token);
                let funders_len = grant.funders.len();
                let mut distributed = 0i128;

                for i in 0..funders_len {
                    let fund_entry = grant.funders.get(i).unwrap();
                    let is_last = i + 1 == funders_len;
                    let refund_amount = if is_last {
                        total_refundable - distributed
                    } else {
                        let amount = fund_entry
                            .amount
                            .checked_mul(total_refundable)
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
                        Events::emit_refund_issued(
                            &env,
                            grant_id,
                            fund_entry.funder.clone(),
                            refund_amount,
                        );
                    }
                }
            }

            // Update state
            grant.status = GrantStatus::Cancelled;
            grant.escrow_balance = 0;
            grant.reason = Some(reason.clone());
            grant.timestamp = env.ledger().timestamp();

            Storage::set_grant(&env, grant_id, &grant);

            // Emit cancellation event
            Events::emit_grant_cancelled(&env, grant_id, caller, reason, total_refundable);

            Ok(())
        })
    }

    /// Mark a grant as completed when all milestones are approved and refund the remaining balance
    pub fn grant_complete(env: Env, grant_id: u64) -> Result<(), ContractError> {
        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }

            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let mut escrow_state = Storage::get_escrow_state(&env, grant_id);
            if escrow_state.lifecycle == EscrowLifecycleState::Released {
                return Err(ContractError::GrantAlreadyReleased);
            }

            // Quorum is interpreted as all milestones approved in current contract design.
            let _ =
                Self::compute_total_paid_if_quorum_ready(&env, grant_id, grant.total_milestones)?;
            escrow_state.quorum_ready = true;

            if escrow_state.mode == EscrowMode::Standard {
                Self::finalize_grant_release(&env, grant_id)?;
                return Ok(());
            }

            // High-security grants remain locked until every multisig signer calls sign_release.
            escrow_state.lifecycle = EscrowLifecycleState::AwaitingMultisig;
            Storage::set_escrow_state(&env, grant_id, &escrow_state);
            Ok(())
        })
    }

    /// Sign release for a high-security grant.
    ///
    /// # Arguments
    /// * `grant_id` - Grant identifier.
    /// * `signer` - Multisig signer address.
    ///
    /// # Returns
    /// * `Ok(())` on successful signature.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] if grant is missing.
    /// * [`ContractError::InvalidState`] if grant is not active or not high-security.
    /// * [`ContractError::NotMultisigSigner`] if signer is not allowed.
    /// * [`ContractError::AlreadySignedRelease`] if signer already signed.
    ///
    /// # Side Effects
    /// * Updates release approval state and can call `finalize_grant_release` if quorum is met.
    pub fn sign_release(env: Env, grant_id: u64, signer: Address) -> Result<(), ContractError> {
        signer.require_auth();
        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }

            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let mut escrow_state = Storage::get_escrow_state(&env, grant_id);
            if escrow_state.mode != EscrowMode::HighSecurity {
                return Err(ContractError::InvalidState);
            }
            if escrow_state.lifecycle == EscrowLifecycleState::Released {
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
            escrow_state.approvals_count += 1;
            Storage::set_escrow_state(&env, grant_id, &escrow_state);

            let approvals_complete = escrow_state.approvals_count >= signers.len();
            if approvals_complete && escrow_state.quorum_ready {
                Self::finalize_grant_release(&env, grant_id)?;
            } else if approvals_complete {
                escrow_state.lifecycle = EscrowLifecycleState::AwaitingMultisig;
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
                if milestone.state != MilestoneState::Approved
                    && milestone.state != MilestoneState::Paid
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
        if grant.status == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        let total_paid =
            Self::compute_total_paid_if_quorum_ready(env, grant_id, grant.total_milestones)?;
        if grant.escrow_balance < total_paid {
            return Err(ContractError::InvalidInput);
        }
        let remaining_balance = grant.escrow_balance - total_paid;
        let token_client = token::Client::new(env, &grant.token);

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
                    let fund_entry = grant.funders.get(i).unwrap();
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
                            env,
                            grant_id,
                            fund_entry.funder.clone(),
                            refund_amount,
                        );
                    }
                }
            }
        }

        // Mark all approved milestones as paid
        for milestone_idx in 0..grant.total_milestones {
            if let Some(mut milestone) = Storage::get_milestone(env, grant_id, milestone_idx) {
                if milestone.state == MilestoneState::Approved {
                    milestone.state = MilestoneState::Paid;
                    milestone.status_updated_at = env.ledger().timestamp();
                    Storage::set_milestone(env, grant_id, milestone_idx, &milestone);

                    Events::milestone_status_changed(
                        env,
                        grant_id,
                        milestone_idx,
                        MilestoneState::Paid,
                    );
                    Events::emit_milestone_paid(env, grant_id, milestone_idx, milestone.amount);
                }
            }
        }

        grant.status = GrantStatus::Completed;
        grant.escrow_balance = 0;
        grant.milestones_paid_out = grant.total_milestones;
        grant.timestamp = env.ledger().timestamp();
        Storage::set_grant(env, grant_id, &grant);

        if total_paid > 0 {
            if let Some(mut profile) = Storage::get_contributor(env, grant.owner.clone()) {
                profile.total_earned = profile
                    .total_earned
                    .checked_add(total_paid)
                    .ok_or(ContractError::InvalidInput)?;
                profile.reputation_score = profile
                    .reputation_score
                    .checked_add(grant.total_milestones as u64)
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
        escrow_state.lifecycle = EscrowLifecycleState::Released;
        escrow_state.quorum_ready = true;
        Storage::set_escrow_state(env, grant_id, &escrow_state);

        Events::emit_payee_receipt(env, grant_id, grant.owner.clone(), total_paid);

        Events::emit_grant_completed(env, grant_id, total_paid, remaining_balance);
        Ok(())
    }

    /// Allows authorized reviewers to vote on submitted milestones.
    /// Voting is gated behind the community review period: if the milestone is
    /// still in [`MilestoneState::CommunityReview`] and the period has not yet
    /// elapsed, this returns [`ContractError::CommunityReviewPeriod`].
    pub fn milestone_vote(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approve: bool,
        feedback: Option<String>,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();

        if Storage::is_blacklisted(&env, &reviewer) {
            return Err(ContractError::Blacklisted);
        }

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;

        if milestone.state == MilestoneState::CommunityReview {
            if env.ledger().timestamp() < milestone.submission_timestamp + COMMUNITY_REVIEW_PERIOD {
                return Err(ContractError::CommunityReviewPeriod);
            }
            // Community period has elapsed — transition to Submitted so voting proceeds.
            milestone.state = MilestoneState::Submitted;
        } else if milestone.state != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }
        if milestone.state == MilestoneState::Disputed {
            return Err(ContractError::InvalidState);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
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
            milestone.approvals += reputation;
        } else {
            milestone.rejections += reputation;
        }

        let quorum_reached = milestone.approvals >= grant.quorum;
        if quorum_reached {
            milestone.state = MilestoneState::Approved;
            milestone.status_updated_at = env.ledger().timestamp();

            // Emit QuorumReached event
            Events::emit_quorum_reached(
                &env,
                grant_id,
                milestone_idx,
                milestone.approvals,
                grant.quorum,
            );

            // Reward harmonious voters who voted approve
            for (voter, voted_approve) in milestone.votes.iter() {
                if voted_approve {
                    let mut rep = Storage::get_reviewer_reputation(&env, voter.clone());
                    rep += 1;
                    Storage::set_reviewer_reputation(&env, voter.clone(), rep);
                }
            }

            Events::milestone_status_changed(
                &env,
                grant_id,
                milestone_idx,
                MilestoneState::Approved,
            );
        }

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_voted(&env, grant_id, milestone_idx, reviewer, approve, feedback);

        Ok(quorum_reached)
    }

    /// Allows authorized reviewers to reject milestones with a reason.
    /// Subject to the same community review period gate as [`Self::milestone_vote`].
    pub fn milestone_reject(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        reason: String,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();

        if reason.len() > 256 {
            return Err(ContractError::InvalidInput);
        }

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;

        if milestone.state == MilestoneState::CommunityReview {
            if env.ledger().timestamp() < milestone.submission_timestamp + COMMUNITY_REVIEW_PERIOD {
                return Err(ContractError::CommunityReviewPeriod);
            }
            milestone.state = MilestoneState::Submitted;
        } else if milestone.state != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }

        if milestone.votes.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        let reputation = Storage::get_reviewer_reputation(&env, reviewer.clone());
        milestone.votes.set(reviewer.clone(), false);
        milestone.rejections += reputation;
        milestone.reasons.set(reviewer.clone(), reason.clone());

        let majority_rejected = milestone.rejections >= grant.quorum;

        if majority_rejected {
            milestone.state = MilestoneState::Rejected;
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

    /// Allow grant owner to open a dispute when milestone is rejected.
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

        if milestone.state != MilestoneState::Rejected {
            return Err(ContractError::InvalidState);
        }

        milestone.state = MilestoneState::Disputed;
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::milestone_status_changed(&env, grant_id, milestone_idx, MilestoneState::Disputed);
        Ok(())
    }

    /// Council resolves a disputed milestone, either approving or confirming rejection.
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

        if milestone.state != MilestoneState::Disputed {
            return Err(ContractError::InvalidState);
        }

        milestone.state = if approve {
            MilestoneState::Approved
        } else {
            MilestoneState::Rejected
        };
        milestone.status_updated_at = env.ledger().timestamp();
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::milestone_status_changed(&env, grant_id, milestone_idx, milestone.state.clone());

        Ok(())
    }

    /// Allows a grant recipient to submit a completed milestone for reviewer evaluation.
    ///
    /// # Arguments
    /// * `grant_id` - The unique identifier of the grant.
    /// * `milestone_idx` - Zero-based index of the milestone to submit (must be < `total_milestones`).
    /// * `recipient` - The address of the grant recipient submitting the milestone.
    /// * `description` - A human-readable description of work completed for this milestone.
    /// * `proof_url` - A URL pointing to proof of completion (e.g. GitHub PR, report link).
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] – if no grant exists with the given `grant_id`.
    /// * [`ContractError::InvalidState`] – if the grant is not in `Active` status.
    /// * [`ContractError::InvalidInput`] – if `milestone_idx` is out of bounds.
    /// * [`ContractError::Unauthorized`] – if `recipient` is not the grant owner.
    /// * [`ContractError::MilestoneAlreadySubmitted`] – if the milestone is already submitted or approved.
    pub fn milestone_submit(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        recipient: Address,
        description: String,
        proof_url: String,
    ) -> Result<(), ContractError> {
        recipient.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        check_heartbeat(&env, &mut grant);

        if grant.status == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status != GrantStatus::Active {
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
        )
    }

    /// Submits multiple milestones in one transaction.
    ///
    /// # Errors
    /// * [`ContractError::BatchEmpty`] – if `submissions` is empty.
    /// * [`ContractError::BatchTooLarge`] – if more than 20 submissions.
    /// * Same errors as [`Self::milestone_submit`] for grant and per-milestone validation.
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

        if grant.status == GrantStatus::Inactive {
            return Err(ContractError::HeartbeatMissed);
        }
        if grant.status != GrantStatus::Active {
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
            )?;
        }

        Ok(())
    }

    /// Allows a funder to deposit tokens into escrow for a specific grant.
    ///
    /// # Arguments
    /// * `grant_id` - The unique identifier of the grant.
    /// * `funder` - The address of the entity sending funds.
    /// * `amount` - The amount of tokens to deposit.
    ///
    /// # Errors
    /// * [`ContractError::InvalidInput`] – if `amount <= 0` or if addition overflows.
    /// * [`ContractError::GrantNotFound`] – if no grant exists with the given `grant_id`.
    /// * [`ContractError::InvalidState`] – if the grant is not in `Active` status.
    pub fn grant_fund(
        env: Env,
        grant_id: u64,
        funder: Address,
        amount: i128,
        memo: Option<String>,
    ) -> Result<(), ContractError> {
        funder.require_auth();
        reentrancy::with_non_reentrant(&env, || {
            if amount <= 0 {
                return Err(ContractError::InvalidInput);
            }

            let mut grant =
                Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

            check_heartbeat(&env, &mut grant);

            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            // Perform the token transfer from the funder to the contract
            let token_client = token::Client::new(&env, &grant.token);
            let contract_address = env.current_contract_address();
            token_client.transfer(&funder, &contract_address, &amount);

            // Update escrow balance with overflow protection
            grant.escrow_balance = grant
                .escrow_balance
                .checked_add(amount)
                .ok_or(ContractError::InvalidInput)?;

            // Update funds tracking
            let mut funder_found = false;
            for i in 0..grant.funders.len() {
                let mut fund_entry = grant.funders.get(i).unwrap();
                if fund_entry.funder == funder {
                    fund_entry.amount = fund_entry
                        .amount
                        .checked_add(amount)
                        .ok_or(ContractError::InvalidInput)?;
                    grant.funders.set(i, fund_entry);
                    funder_found = true;
                    break;
                }
            }

            if !funder_found {
                grant.funders.push_back(GrantFund {
                    funder: funder.clone(),
                    amount,
                });
            }

            Storage::set_grant(&env, grant_id, &grant);

            Events::emit_grant_funded(&env, grant_id, funder.clone(), amount, grant.escrow_balance);
            Events::emit_payer_receipt(&env, grant_id, funder, amount, memo);

            Ok(())
        })
    }

    /// Record a community upvote on a milestone in [`MilestoneState::CommunityReview`].
    /// Each address may upvote at most once per milestone.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] – grant does not exist.
    /// * [`ContractError::MilestoneNotFound`] – milestone does not exist.
    /// * [`ContractError::InvalidState`] – milestone is not in `CommunityReview`.
    /// * [`ContractError::AlreadyUpvoted`] – voter has already upvoted.
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

        if milestone.state != MilestoneState::CommunityReview {
            return Err(ContractError::InvalidState);
        }
        if Storage::has_milestone_upvote(&env, grant_id, milestone_idx, &voter) {
            return Err(ContractError::AlreadyUpvoted);
        }

        Storage::set_milestone_upvote(&env, grant_id, milestone_idx, &voter);
        milestone.community_upvotes += 1;
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::emit_milestone_upvoted(
            &env,
            grant_id,
            milestone_idx,
            voter,
            milestone.community_upvotes,
        );
        Ok(())
    }

    /// Record a community comment on a milestone in [`MilestoneState::CommunityReview`].
    /// Each address may post one comment; posting again overwrites the previous one.
    /// Comments are informational signals only — they do not affect the voting outcome.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] – grant does not exist.
    /// * [`ContractError::MilestoneNotFound`] – milestone does not exist.
    /// * [`ContractError::InvalidState`] – milestone is not in `CommunityReview`.
    /// * [`ContractError::InvalidInput`] – comment exceeds 512 characters.
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

        if milestone.state != MilestoneState::CommunityReview {
            return Err(ContractError::InvalidState);
        }

        milestone
            .community_comments
            .set(voter.clone(), comment.clone());
        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        Events::emit_milestone_commented(&env, grant_id, milestone_idx, voter, comment);
        Ok(())
    }

    /// Add a new reviewer to an active grant. Only callable by the grant owner.
    ///
    /// # Arguments
    /// * `grant_id` - The grant to update.
    /// * `owner` - The grant owner (must authenticate).
    /// * `new_reviewer` - Address of the reviewer to add.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] – grant does not exist.
    /// * [`ContractError::Unauthorized`] – caller is not the grant owner.
    /// * [`ContractError::InvalidState`] – grant is not active.
    /// * [`ContractError::InvalidInput`] – reviewer is already in the list.
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
        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }
        if grant.reviewers.contains(new_reviewer.clone()) {
            return Err(ContractError::InvalidInput);
        }

        grant.reviewers.push_back(new_reviewer.clone());
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_reviewer_added(&env, grant_id, owner, new_reviewer);
        Ok(())
    }

    /// Remove an existing reviewer from an active grant. Only callable by the grant owner.
    /// Ensures at least one reviewer remains after removal.
    /// Past quorum decisions on milestones are NOT retroactively changed.
    ///
    /// # Arguments
    /// * `grant_id` - The grant to update.
    /// * `owner` - The grant owner (must authenticate).
    /// * `old_reviewer` - Address of the reviewer to remove.
    ///
    /// # Errors
    /// * [`ContractError::GrantNotFound`] – grant does not exist.
    /// * [`ContractError::Unauthorized`] – caller is not the grant owner, or reviewer not found.
    /// * [`ContractError::InvalidState`] – grant is not active.
    /// * [`ContractError::InvalidInput`] – removing would leave zero reviewers, or quorum would exceed reviewer count.
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
        if grant.status != GrantStatus::Active {
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
        if grant.quorum > new_reviewers.len() {
            return Err(ContractError::InvalidInput);
        }

        grant.reviewers = new_reviewers;
        Storage::set_grant(&env, grant_id, &grant);

        Events::emit_reviewer_removed(&env, grant_id, owner, old_reviewer);
        Ok(())
    }

    /// Retrieve a grant by its ID
    pub fn get_grant(env: Env, grant_id: u64) -> Result<Grant, ContractError> {
        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)
    }

    pub fn get_milestone(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<Milestone, ContractError> {
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if milestone_idx >= grant.total_milestones {
            return Err(ContractError::InvalidInput);
        }

        Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotFound)
    }

    /// Retrieve all reviewer feedback for a milestone
    pub fn get_milestone_feedback(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<soroban_sdk::Map<Address, String>, ContractError> {
        let milestone = Self::get_milestone(env, grant_id, milestone_idx)?;
        Ok(milestone.reasons)
    }

    // ── Reviewer Staking (#42) ──────────────────────────────────────

    /// Admin sets the minimum stake required for reviewers and the treasury address.
    pub fn set_staking_config(
        env: Env,
        admin: Address,
        min_stake: i128,
        treasury: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let global = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if global != admin {
            return Err(ContractError::NotContractAdmin);
        }
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

    /// Reviewer stakes tokens to participate in a grant's review quorum.
    pub fn stake_to_review(
        env: Env,
        reviewer: Address,
        grant_id: u64,
        amount: i128,
    ) -> Result<(), ContractError> {
        reviewer.require_auth();

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status == GrantStatus::Inactive {
                return Err(ContractError::HeartbeatMissed);
            }
            if grant.status != GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let min_stake = Storage::get_min_reviewer_stake(&env);
            if amount < min_stake {
                return Err(ContractError::InsufficientStake);
            }

            let contract_addr = env.current_contract_address();
            let client = token::Client::new(&env, &grant.token);
            client.transfer(&reviewer, &contract_addr, &amount);

            let current = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            Storage::set_reviewer_stake(&env, grant_id, &reviewer, current + amount);

            Ok(())
        })
    }

    /// Admin slashes a malicious reviewer's stake, sending it to treasury.
    pub fn slash_reviewer(
        env: Env,
        admin: Address,
        grant_id: u64,
        reviewer: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let global = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if global != admin {
            return Err(ContractError::NotContractAdmin);
        }

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            let stake = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            if stake <= 0 {
                return Err(ContractError::StakeNotFound);
            }

            let treasury = Storage::get_treasury(&env).ok_or(ContractError::InvalidInput)?;
            let client = token::Client::new(&env, &grant.token);
            client.transfer(&env.current_contract_address(), &treasury, &stake);

            Storage::set_reviewer_stake(&env, grant_id, &reviewer, 0);

            Ok(())
        })
    }

    /// Reviewer unstakes tokens after a grant lifecycle completes.
    pub fn unstake(env: Env, reviewer: Address, grant_id: u64) -> Result<(), ContractError> {
        reviewer.require_auth();

        reentrancy::with_non_reentrant(&env, || {
            let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
            if grant.status == GrantStatus::Active {
                return Err(ContractError::InvalidState);
            }

            let stake = Storage::get_reviewer_stake(&env, grant_id, &reviewer);
            if stake <= 0 {
                return Err(ContractError::StakeNotFound);
            }

            let client = token::Client::new(&env, &grant.token);
            client.transfer(&env.current_contract_address(), &reviewer, &stake);

            Storage::set_reviewer_stake(&env, grant_id, &reviewer, 0);

            Ok(())
        })
    }

    // ── KYC Integration (#43) ───────────────────────────────────────

    /// Admin sets the identity oracle contract address for KYC verification.
    pub fn set_identity_oracle(
        env: Env,
        admin: Address,
        oracle: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let global = Storage::get_global_admin(&env).ok_or(ContractError::NotContractAdmin)?;
        if global != admin {
            return Err(ContractError::NotContractAdmin);
        }
        env.storage()
            .persistent()
            .set(&storage::DataKey::IdentityOracle, &oracle);
        Ok(())
    }

    // ── Bulk Funding (#44) ──────────────────────────────────────────

    /// Fund multiple grants in a single transaction.
    ///
    /// Accepts a vector of (grant_id, amount) tuples. Reverts the entire
    /// batch if any individual grant fails validation.
    pub fn fund_batch(
        env: Env,
        funder: Address,
        grants: Vec<(u64, i128)>,
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
                let (grant_id, amount) = item;
                if amount <= 0 {
                    return Err(ContractError::InvalidInput);
                }

                let mut grant =
                    Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

                check_heartbeat(&env, &mut grant);

                if grant.status == GrantStatus::Inactive {
                    return Err(ContractError::HeartbeatMissed);
                }
                if grant.status != GrantStatus::Active {
                    return Err(ContractError::InvalidState);
                }

                let contract_addr = env.current_contract_address();
                let client = token::Client::new(&env, &grant.token);
                client.transfer(&funder, &contract_addr, &amount);

                grant.escrow_balance = grant
                    .escrow_balance
                    .checked_add(amount)
                    .ok_or(ContractError::InvalidInput)?;

                let mut found = false;
                let mut new_funders = soroban_sdk::Vec::new(&env);
                for f in grant.funders.iter() {
                    if f.funder == funder {
                        new_funders.push_back(GrantFund {
                            funder: f.funder,
                            amount: f.amount + amount,
                        });
                        found = true;
                    } else {
                        new_funders.push_back(f);
                    }
                }
                if !found {
                    new_funders.push_back(GrantFund {
                        funder: funder.clone(),
                        amount,
                    });
                }
                grant.funders = new_funders;

                Storage::set_grant(&env, grant_id, &grant);

                Events::emit_grant_funded(
                    &env,
                    grant_id,
                    funder.clone(),
                    amount,
                    grant.escrow_balance,
                );
                Events::emit_payer_receipt(&env, grant_id, funder.clone(), amount, None);
            }

            Ok(())
        })
    }

    /// Update the grant's heartbeat to the current ledger timestamp.
    /// Can only be called by the grant owner while the grant is Active or Inactive.
    /// If the grant was Inactive, it will be restored to Active.
    pub fn grant_ping(env: Env, grant_id: u64, owner: Address) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }

        // Grant must be in a state where pinging makes sense
        if grant.status != GrantStatus::Active && grant.status != GrantStatus::Inactive {
            return Err(ContractError::InvalidState);
        }

        let now = env.ledger().timestamp();
        grant.last_heartbeat = now;

        // If it was inactive, restore it to active
        if grant.status == GrantStatus::Inactive {
            grant.status = GrantStatus::Active;
        }

        Storage::set_grant(&env, grant_id, &grant);
        Events::emit_heartbeat_updated(&env, grant_id, now);

        Ok(())
    }

    /// Admin function to blacklist an address from creating or interacting with grants.
    pub fn admin_blacklist_add(
        env: Env,
        admin: Address,
        target: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let global_admin = Storage::get_global_admin(&env).ok_or(ContractError::Unauthorized)?;
        if admin != global_admin {
            return Err(ContractError::Unauthorized);
        }

        Storage::set_blacklisted(&env, &target);
        Ok(())
    }

    /// Admin function to remove an address from the blacklist.
    pub fn admin_blacklist_remove(
        env: Env,
        admin: Address,
        target: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        let global_admin = Storage::get_global_admin(&env).ok_or(ContractError::Unauthorized)?;
        if admin != global_admin {
            return Err(ContractError::Unauthorized);
        }

        Storage::remove_blacklisted(&env, &target);
        Ok(())
    }
}

fn check_heartbeat(env: &Env, grant: &mut Grant) {
    if grant.status != GrantStatus::Active {
        return;
    }

    let now = env.ledger().timestamp();
    let seconds_since_heartbeat = now.saturating_sub(grant.last_heartbeat);

    // 30 days = 30 * 24 * 60 * 60 = 2,592,000 seconds
    if seconds_since_heartbeat > 30 * 24 * 60 * 60 {
        grant.status = GrantStatus::Inactive;
        Storage::set_grant(env, grant.id, grant);
        Events::emit_grant_gone_inactive(env, grant.id, now);
    }
}

fn apply_milestone_submission(
    env: &Env,
    grant_id: u64,
    grant: &Grant,
    milestone_idx: u32,
    description: String,
    proof_url: String,
) -> Result<(), ContractError> {
    if Storage::is_blacklisted(env, &grant.owner) {
        return Err(ContractError::Blacklisted);
    }

    if grant.status == GrantStatus::Inactive {
        return Err(ContractError::HeartbeatMissed);
    }

    if milestone_idx >= grant.total_milestones {
        return Err(ContractError::InvalidInput);
    }

    let mut milestone = Storage::get_milestone(env, grant_id, milestone_idx)
        .ok_or(ContractError::MilestoneNotFound)?;

    if milestone.state == MilestoneState::CommunityReview
        || milestone.state == MilestoneState::Submitted
        || milestone.state == MilestoneState::Approved
        || milestone.state == MilestoneState::Paid
    {
        return Err(ContractError::MilestoneAlreadySubmitted);
    }

    if milestone.deadline > 0 && env.ledger().timestamp() > milestone.deadline {
        Events::emit_milestone_expired(env, grant_id, milestone_idx);
        return Err(ContractError::DeadlinePassed);
    }

    milestone.description = description.clone();
    // Milestone enters the community review window before official voting opens.
    milestone.state = MilestoneState::CommunityReview;
    milestone.proof_url = Some(proof_url);
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
