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

<<<<<<< HEAD
        // 1. Validation
        let grant = Storage::get_grant_v(&env, grant_id);
        let mut milestone = Storage::get_milestone_v(&env, grant_id, milestone_idx);
=======
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;
>>>>>>> 8a269c2 (chore: resolve upstream merge conflicts and update test schemas)

        if milestone.state != MilestoneState::Submitted {
            env.panic_with_error(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            env.panic_with_error(ContractError::Unauthorized);
        }

        if milestone.votes.contains_key(reviewer.clone()) {
            env.panic_with_error(ContractError::AlreadyVoted);
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

<<<<<<< HEAD
        // 1. Validation
        let grant = Storage::get_grant_v(&env, grant_id);
        let mut milestone = Storage::get_milestone_v(&env, grant_id, milestone_idx);
=======
        let grant = Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)?;
        let mut milestone = Storage::get_milestone(&env, grant_id, milestone_idx)
            .ok_or(ContractError::MilestoneNotSubmitted)?;
>>>>>>> 8a269c2 (chore: resolve upstream merge conflicts and update test schemas)

        if milestone.state != MilestoneState::Submitted {
            env.panic_with_error(ContractError::MilestoneNotSubmitted);
        }

        if !grant.reviewers.contains(reviewer.clone()) {
            env.panic_with_error(ContractError::Unauthorized);
        }

        if milestone.votes.contains_key(reviewer.clone()) {
            env.panic_with_error(ContractError::AlreadyVoted);
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

    /// Retrieve a grant by its ID
    pub fn get_grant(env: Env, grant_id: u64) -> Result<Grant, ContractError> {
        Storage::get_grant(&env, grant_id).ok_or(ContractError::GrantNotFound)
    }

    pub fn get_milestone(
        env: Env,
        grant_id: u64,
        milestone_idx: u32,
    ) -> Result<Milestone, ContractError> {
        let grant = Storage::get_grant_v(&env, grant_id);

        if milestone_idx >= grant.total_milestones {
            env.panic_with_error(ContractError::InvalidInput);
        }

        let milestone = Storage::get_milestone_v(&env, grant_id, milestone_idx);
        Ok(milestone)
    }
}

#[cfg(test)]
mod test;
