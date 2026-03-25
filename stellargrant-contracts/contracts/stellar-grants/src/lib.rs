#![no_std]
mod events;
mod storage;
mod types;

pub use events::Events;
pub use storage::Storage;
pub use types::{ContractError, Grant, GrantFund, GrantStatus, Milestone, MilestoneState};

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

#[contract]
pub struct StellarGrantsContract;

#[contractimpl]
impl StellarGrantsContract {
    /// Initialize the contract
    pub fn initialize(_env: Env) -> Result<(), ContractError> {
        Ok(())
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
            grants_count: 0,
            total_earned: 0,
        };

        Storage::set_contributor(&env, contributor.clone(), &profile);

        Events::emit_contributor_registered(&env, contributor, name);

        Ok(())
    }

    /// Cancel a grant and refund remaining balance to funders
    pub fn grant_cancel(
        env: Env,
        grant_id: u64,
        owner: Address,
        reason: String,
    ) -> Result<(), ContractError> {
        owner.require_auth();

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.owner != owner {
            return Err(ContractError::Unauthorized);
        }

        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        // Cannot cancel if all milestones are approved/paid out
        if grant.milestones_paid_out >= grant.total_milestones {
            return Err(ContractError::InvalidState);
        }

        let total_refundable = grant.escrow_balance;
        if total_refundable <= 0 {
            return Err(ContractError::NoRefundableAmount);
        }

        let mut total_contributions: i128 = 0;
        for fund_entry in grant.funders.iter() {
            total_contributions += fund_entry.amount;
        }

        if total_contributions <= 0 {
            return Err(ContractError::NoRefundableAmount);
        }

        let token_client = token::Client::new(&env, &grant.token);

        for fund_entry in grant.funders.iter() {
            let refund_amount = fund_entry
                .amount
                .checked_mul(total_refundable)
                .ok_or(ContractError::InvalidInput)?
                .checked_div(total_contributions)
                .ok_or(ContractError::InvalidInput)?;

            if refund_amount > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &fund_entry.funder,
                    &refund_amount,
                );
                Events::emit_refund_executed(
                    &env,
                    grant_id,
                    fund_entry.funder.clone(),
                    refund_amount,
                );
            }
        }

        // Update state
        grant.status = GrantStatus::Cancelled;
        grant.escrow_balance = 0;
        grant.reason = Some(reason.clone());
        grant.timestamp = env.ledger().timestamp();

        Storage::set_grant(&env, grant_id, &grant);

        // Emit cancellation event
        Events::emit_grant_cancelled(&env, grant_id, owner, reason, total_refundable);

        Ok(())
    }

    /// Mark a grant as completed when all milestones are approved and refund the remaining balance
    pub fn grant_complete(env: Env, grant_id: u64) -> Result<(), ContractError> {
        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        // Verify all milestones are approved and calculate total paid
        let mut total_paid: i128 = 0;
        let mut approved_count = 0;

        for milestone_idx in 0..grant.total_milestones {
            if let Some(milestone) = Storage::get_milestone(&env, grant_id, milestone_idx) {
                if milestone.state != MilestoneState::Approved {
                    return Err(ContractError::NotAllMilestonesApproved);
                }
                total_paid += milestone.amount;
                approved_count += 1;
            } else {
                return Err(ContractError::NotAllMilestonesApproved);
            }
        }

        if approved_count != grant.total_milestones {
            return Err(ContractError::NotAllMilestonesApproved);
        }

        // Calculate remaining balance refunds
        let remaining_balance = grant.escrow_balance - total_paid;

        if remaining_balance > 0 {
            let mut total_contributions: i128 = 0;
            for fund_entry in grant.funders.iter() {
                total_contributions += fund_entry.amount;
            }

            if total_contributions > 0 {
                let token_client = token::Client::new(&env, &grant.token);

                for fund_entry in grant.funders.iter() {
                    let refund_amount = fund_entry
                        .amount
                        .checked_mul(remaining_balance)
                        .ok_or(ContractError::InvalidInput)?
                        .checked_div(total_contributions)
                        .ok_or(ContractError::InvalidInput)?;

                    if refund_amount > 0 {
                        token_client.transfer(
                            &env.current_contract_address(),
                            &fund_entry.funder,
                            &refund_amount,
                        );
                        Events::emit_final_refund(
                            &env,
                            grant_id,
                            fund_entry.funder.clone(),
                            refund_amount,
                        );
                    }
                }
            }
        }

        // Update state
        grant.status = GrantStatus::Completed;
        grant.escrow_balance = 0;
        grant.timestamp = env.ledger().timestamp();

        Storage::set_grant(&env, grant_id, &grant);

        // Emit completion event
        Events::emit_grant_completed(&env, grant_id, total_paid, remaining_balance);

        Ok(())
    }

    /// Allows authorized reviewers to vote on submitted milestones.
    pub fn milestone_vote(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approve: bool,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;

        if milestone.state != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }

        if milestone.votes.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        milestone.votes.set(reviewer.clone(), approve);
        if approve {
            milestone.approvals += 1;
        } else {
            milestone.rejections += 1;
        }

        let total_reviewers = grant.reviewers.len();
        let quorum_threshold = (total_reviewers / 2) + 1;
        let quorum_reached = milestone.approvals >= quorum_threshold;

        if quorum_reached {
            milestone.state = MilestoneState::Approved;
            milestone.status_updated_at = env.ledger().timestamp();
            Events::milestone_status_changed(
                &env,
                grant_id,
                milestone_idx,
                MilestoneState::Approved,
            );
        }

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);
        Events::milestone_voted(&env, grant_id, milestone_idx, reviewer, approve);

        Ok(quorum_reached)
    }

    /// Allows authorized reviewers to reject milestones with a reason.
    pub fn milestone_reject(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        reason: String,
    ) -> Result<bool, ContractError> {
        reviewer.require_auth();

        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;

        if milestone.state != MilestoneState::Submitted {
            return Err(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            return Err(ContractError::Unauthorized);
        }

        if milestone.votes.contains_key(reviewer.clone()) {
            return Err(ContractError::AlreadyVoted);
        }

        milestone.votes.set(reviewer.clone(), false);
        milestone.rejections += 1;
        milestone.reasons.set(reviewer.clone(), reason.clone());

        let total_reviewers = grant.reviewers.len();
        let majority_threshold = (total_reviewers / 2) + 1;
        let majority_rejected = milestone.rejections >= majority_threshold;

        if majority_rejected {
            milestone.state = MilestoneState::Rejected;
            milestone.status_updated_at = env.ledger().timestamp();
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

        // 1. Grant must exist
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

        // 2. Grant must be Active
        if grant.status != GrantStatus::Active {
            return Err(ContractError::InvalidState);
        }

        // 3. Milestone index must be valid
        if milestone_idx >= grant.total_milestones {
            return Err(ContractError::InvalidInput);
        }

        // 4. Caller must be the grant recipient (owner)
        if grant.owner != recipient {
            return Err(ContractError::Unauthorized);
        }

        // 5. Milestone must not already be submitted or approved
        if let Some(existing) = Storage::get_milestone(&env, grant_id, milestone_idx) {
            if existing.state == MilestoneState::Submitted
                || existing.state == MilestoneState::Approved
            {
                return Err(ContractError::MilestoneAlreadySubmitted);
            }
        }

        // Build and store the milestone in Submitted state
        let milestone = Milestone {
            idx: milestone_idx,
            description: description.clone(),
            amount: 0,
            state: MilestoneState::Submitted,
            votes: soroban_sdk::Map::new(&env),
            approvals: 0,
            rejections: 0,
            reasons: soroban_sdk::Map::new(&env),
            status_updated_at: 0,
            proof_url: Some(proof_url),
            submission_timestamp: env.ledger().timestamp(),
        };

        Storage::set_milestone(&env, grant_id, milestone_idx, &milestone);

        // Emit submission event
        Events::emit_milestone_submitted(&env, grant_id, milestone_idx, description);

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
    ) -> Result<(), ContractError> {
        funder.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidInput);
        }

        let mut grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;

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

        Events::emit_grant_funded(&env, grant_id, funder, amount, grant.escrow_balance);

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
}

#[cfg(test)]
mod test;
