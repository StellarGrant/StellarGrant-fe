use crate::types::{MilestoneState, Role};
use soroban_sdk::{contractevent, Address, BytesN, Env, String, Vec};

const EVENT_VERSION: u32 = 1;
const GLOBAL_EVENT_GRANT_ID: u64 = 0;

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneVoted {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub reviewer: Address,
    pub approve: bool,
    pub feedback: Option<String>,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneRejected {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub reviewer: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneChallenged {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub funder: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneStatusChanged {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub new_state: MilestoneState,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestonePaid {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub amount: i128,
    pub token: Address, // New: Specify the token paid
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneApproved {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub payout_amount: i128,
    pub payout_token: Address, // New: Specify the token approved
    pub recipient: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayoutExecuted {
    pub event_version: u32,
    pub grant_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub token: Address, // New: Specify the token paid
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCancelled {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub reason: String,
    pub refund_amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefundExecuted {
    pub event_version: u32,
    pub grant_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefundIssued {
    pub event_version: u32,
    pub grant_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub token: Address, // New: Specify which token was refunded
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCompleted {
    pub event_version: u32,
    pub grant_id: u64,
    pub total_paid: i128,
    pub remaining_balance: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalRefund {
    pub event_version: u32,
    pub grant_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpiredFundsClaimed {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub caller: Address,
    pub amount: i128,
    pub token: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributorRegistered {
    pub event_version: u32,
    pub grant_id: u64,
    pub contributor: Address,
    pub name: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneExtensionRequested {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub new_deadline: u64,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneExtensionApproved {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub reviewer: Address,
    pub approvals: u32,
    pub quorum: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReputationIncreased {
    pub event_version: u32,
    pub grant_id: u64,
    pub contributor: Address,
    pub new_reputation_score: u64,
    pub total_earned: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneSubmitted {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub description: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BountyClaimed {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub winner: Address,
    pub submission_index: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantFunded {
    pub event_version: u32,
    pub grant_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub token: Address, // New: Specify which token was funded
    pub new_balance: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneTopUpFunded {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub funder: Address,
    pub token: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCreated {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub title: String,
    pub total_amount: i128,
    pub tags: Vec<soroban_sdk::String>,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantMetadataUpdated {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub title: String,
    pub description: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractInitialized {
    pub event_version: u32,
    pub grant_id: u64,
    pub council: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractUpgraded {
    pub event_version: u32,
    pub grant_id: u64,
    pub actor: Address,
    pub component: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractWasmUpgraded {
    pub event_version: u32,
    pub grant_id: u64,
    pub admin: Address,
    pub new_wasm_hash: BytesN<32>,
    pub new_storage_version: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleGranted {
    pub event_version: u32,
    pub grant_id: u64,
    pub role: Role,
    pub account: Address,
    pub sender: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleRevoked {
    pub event_version: u32,
    pub grant_id: u64,
    pub role: Role,
    pub account: Address,
    pub sender: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleRenounced {
    pub event_version: u32,
    pub grant_id: u64,
    pub role: Role,
    pub account: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewerDelegated {
    pub event_version: u32,
    pub grant_id: u64,
    pub delegator: Address,
    pub delegatee: Address,
    pub timestamp: u64,
}

pub struct Events;

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuorumReached {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub approvals: u32,
    pub quorum: u32,
    pub timestamp: u64,
}

impl Events {
    pub fn emit_quorum_reached(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        approvals: u32,
        quorum: u32,
    ) {
        let event = QuorumReached {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            approvals,
            quorum,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }
    pub fn emit_contract_initialized(env: &Env, council: Address) {
        let event = ContractInitialized {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            council,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_contract_upgraded(env: &Env, actor: Address, component: String) {
        let event = ContractUpgraded {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            actor,
            component,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_contract_wasm_upgraded(
        env: &Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
        new_storage_version: u32,
    ) {
        let event = ContractWasmUpgraded {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            admin,
            new_wasm_hash,
            new_storage_version,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_role_granted(env: &Env, sender: Address, account: Address, role: Role) {
        let event = RoleGranted {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            role,
            account,
            sender,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_role_revoked(env: &Env, sender: Address, account: Address, role: Role) {
        let event = RoleRevoked {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            role,
            account,
            sender,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_role_renounced(env: &Env, account: Address, role: Role) {
        let event = RoleRenounced {
            event_version: EVENT_VERSION,
            grant_id: GLOBAL_EVENT_GRANT_ID,
            role,
            account,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_cancelled(
        env: &Env,
        grant_id: u64,
        owner: Address,
        reason: String,
        refund_amount: i128,
    ) {
        let event = GrantCancelled {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            reason,
            refund_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_refund_executed(env: &Env, grant_id: u64, funder: Address, amount: i128) {
        let event = RefundExecuted {
            event_version: EVENT_VERSION,
            grant_id,
            funder,
            amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_refund_issued(
        env: &Env,
        grant_id: u64,
        funder: Address,
        amount: i128,
        token: Address,
    ) {
        let event = RefundIssued {
            event_version: EVENT_VERSION,
            grant_id,
            funder,
            amount,
            token,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_completed(
        env: &Env,
        grant_id: u64,
        total_paid: i128,
        remaining_balance: i128,
    ) {
        let event = GrantCompleted {
            event_version: EVENT_VERSION,
            grant_id,
            total_paid,
            remaining_balance,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_final_refund(env: &Env, grant_id: u64, funder: Address, amount: i128) {
        let event = FinalRefund {
            event_version: EVENT_VERSION,
            grant_id,
            funder,
            amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_expired_funds_claimed(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        caller: Address,
        amount: i128,
        token: Address,
    ) {
        let event = ExpiredFundsClaimed {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            caller,
            amount,
            token,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_submitted(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        description: String,
    ) {
        let event = MilestoneSubmitted {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            description,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_bounty_claimed(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        winner: Address,
        submission_index: u32,
    ) {
        let event = BountyClaimed {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            winner,
            submission_index,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_funded(
        env: &Env,
        grant_id: u64,
        funder: Address,
        amount: i128,
        token: Address,
        new_balance: i128,
    ) {
        let event = GrantFunded {
            event_version: EVENT_VERSION,
            grant_id,
            funder,
            amount,
            token,
            new_balance,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_top_up_funded(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        funder: Address,
        token: Address,
        amount: i128,
    ) {
        let event = MilestoneTopUpFunded {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            funder,
            token,
            amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_created(
        env: &Env,
        grant_id: u64,
        owner: Address,
        title: String,
        total_amount: i128,
        tags: Vec<soroban_sdk::String>,
    ) {
        let event = GrantCreated {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            title,
            total_amount,
            tags,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_metadata_updated(
        env: &Env,
        grant_id: u64,
        owner: Address,
        title: String,
        description: String,
    ) {
        let event = GrantMetadataUpdated {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            title,
            description,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_contributor_registered(
        env: &Env,
        grant_id: u64,
        contributor: Address,
        name: String,
    ) {
        let event = ContributorRegistered {
            event_version: EVENT_VERSION,
            grant_id,
            contributor,
            name,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_extension_requested(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        new_deadline: u64,
    ) {
        let event = MilestoneExtensionRequested {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            new_deadline,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_extension_approved(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approvals: u32,
        quorum: u32,
    ) {
        let event = MilestoneExtensionApproved {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            reviewer,
            approvals,
            quorum,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_reputation_increased(
        env: &Env,
        grant_id: u64,
        contributor: Address,
        new_reputation_score: u64,
        total_earned: i128,
    ) {
        let event = ReputationIncreased {
            event_version: EVENT_VERSION,
            grant_id,
            contributor,
            new_reputation_score,
            total_earned,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_voted(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        approve: bool,
        feedback: Option<String>,
    ) {
        let event = MilestoneVoted {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            reviewer,
            approve,
            feedback,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_rejected(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        reviewer: Address,
        reason: String,
    ) {
        let event = MilestoneRejected {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            reviewer,
            reason,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_challenged(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        funder: Address,
        reason: String,
    ) {
        let event = MilestoneChallenged {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            funder,
            reason,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn milestone_status_changed(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        new_state: MilestoneState,
    ) {
        let event = MilestoneStatusChanged {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            new_state,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_paid(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        amount: i128,
        token: Address,
    ) {
        let event = MilestonePaid {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            amount,
            token,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_approved(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        payout_amount: i128,
        payout_token: Address,
        recipient: Address,
    ) {
        let event = MilestoneApproved {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            payout_amount,
            payout_token,
            recipient,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_payout_executed(
        env: &Env,
        grant_id: u64,
        recipient: Address,
        amount: i128,
        token: Address,
    ) {
        let event = PayoutExecuted {
            event_version: EVENT_VERSION,
            grant_id,
            recipient,
            amount,
            token,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_expired(env: &Env, grant_id: u64, milestone_idx: u32) {
        let event = MilestoneExpired {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_reviewer_added(env: &Env, grant_id: u64, owner: Address, new_reviewer: Address) {
        let event = ReviewerAdded {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            new_reviewer,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_reviewer_removed(env: &Env, grant_id: u64, owner: Address, old_reviewer: Address) {
        let event = ReviewerRemoved {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            old_reviewer,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_upvoted(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: Address,
        total_upvotes: u32,
    ) {
        let event = MilestoneUpvoted {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            voter,
            total_upvotes,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_milestone_commented(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        voter: Address,
        comment: String,
    ) {
        let event = MilestoneCommented {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            voter,
            comment,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_cancellation_requested(
        env: &Env,
        grant_id: u64,
        owner: Address,
        reason: String,
        executable_after: u64,
    ) {
        let event = GrantCancellationRequested {
            event_version: EVENT_VERSION,
            grant_id,
            owner,
            reason,
            executable_after,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_paused(env: &Env, grant_id: u64, actor: Address) {
        let event = GrantPaused {
            event_version: EVENT_VERSION,
            grant_id,
            actor,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_resumed(env: &Env, grant_id: u64, actor: Address) {
        let event = GrantResumed {
            event_version: EVENT_VERSION,
            grant_id,
            actor,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_payee_receipt(
        env: &Env,
        grant_id: u64,
        recipient: Address,
        token: Address,
        amount: i128,
        milestone_index: Option<u32>,
    ) {
        let event = PayeeReceipt {
            event_version: EVENT_VERSION,
            grant_id,
            recipient,
            token,
            amount,
            milestone_index,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_payer_receipt(
        env: &Env,
        grant_id: u64,
        recipient: Address,
        amount: i128,
        token: Address,
        milestone_index: Option<u32>,
        memo: Option<String>,
    ) {
        let event = PayerReceipt {
            event_version: EVENT_VERSION,
            grant_id,
            recipient,
            token,
            amount,
            milestone_index,
            memo,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_heartbeat_updated(env: &Env, grant_id: u64, timestamp: u64) {
        let event = HeartbeatUpdated {
            event_version: EVENT_VERSION,
            grant_id,
            timestamp,
        };
        event.publish(env);
    }

    pub fn emit_grant_gone_inactive(env: &Env, grant_id: u64, timestamp: u64) {
        let event = GrantGoneInactive {
            event_version: EVENT_VERSION,
            grant_id,
            timestamp,
        };
        event.publish(env);
    }

    pub fn emit_grant_activated(env: &Env, grant_id: u64) {
        let event = GrantActivated {
            event_version: EVENT_VERSION,
            grant_id,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_grant_accepted(env: &Env, grant_id: u64, recipient: Address) {
        let event = GrantAccepted {
            event_version: EVENT_VERSION,
            grant_id,
            recipient,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    // ── Issue #152: dispute fee events ───────────────────────────────────────

    pub fn emit_dispute_fee_charged(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        payer: Address,
        fee_amount: i128,
    ) {
        let event = DisputeFeeCharged {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            payer,
            fee_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_dispute_fee_refunded(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        recipient: Address,
        fee_amount: i128,
    ) {
        let event = DisputeFeeRefunded {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            recipient,
            fee_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn emit_dispute_fee_slashed(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        treasury: Address,
        fee_amount: i128,
    ) {
        let event = DisputeFeeSlashed {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            treasury,
            fee_amount,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    // ── Issue #151: reputation event ─────────────────────────────────────────

    pub fn emit_reputation_updated(
        env: &Env,
        grant_id: u64,
        milestone_idx: u32,
        contributor: Address,
        new_reputation_score: u64,
        total_earned: i128,
    ) {
        let event = ReputationUpdated {
            event_version: EVENT_VERSION,
            grant_id,
            milestone_idx,
            contributor,
            new_reputation_score,
            total_earned,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    // ── Issue #163: grant clawback event ─────────────────────────────────────

    pub fn emit_grant_clawbacked(
        env: &Env,
        grant_id: u64,
        council: Address,
        total_clawed_back: i128,
    ) {
        let event = GrantClawbacked {
            event_version: EVENT_VERSION,
            grant_id,
            council,
            total_clawed_back,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }

    pub fn reviewer_delegated(env: &Env, grant_id: u64, delegator: Address, delegatee: Address) {
        let event = ReviewerDelegated {
            event_version: EVENT_VERSION,
            grant_id,
            delegator,
            delegatee,
            timestamp: env.ledger().timestamp(),
        };
        event.publish(env);
    }
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneExpired {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneUpvoted {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub voter: Address,
    pub total_upvotes: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneCommented {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub voter: Address,
    pub comment: String,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantCancellationRequested {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub reason: String,
    pub executable_after: u64,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewerAdded {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub new_reviewer: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewerRemoved {
    pub event_version: u32,
    pub grant_id: u64,
    pub owner: Address,
    pub old_reviewer: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantPaused {
    pub event_version: u32,
    pub grant_id: u64,
    pub actor: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantResumed {
    pub event_version: u32,
    pub grant_id: u64,
    pub actor: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayeeReceipt {
    pub event_version: u32,
    pub grant_id: u64,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub milestone_index: Option<u32>,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayerReceipt {
    pub event_version: u32,
    pub grant_id: u64,
    pub recipient: Address,
    pub token: Address,
    pub amount: i128,
    pub milestone_index: Option<u32>,
    pub memo: Option<String>,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatUpdated {
    pub event_version: u32,
    pub grant_id: u64,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantGoneInactive {
    pub event_version: u32,
    pub grant_id: u64,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantActivated {
    pub event_version: u32,
    pub grant_id: u64,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantAccepted {
    pub event_version: u32,
    pub grant_id: u64,
    pub recipient: Address,
    pub timestamp: u64,
}

/// Emitted when the dispute fee is deducted from the disputing party (issue #152).
#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisputeFeeCharged {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub payer: Address,
    pub fee_amount: i128,
    pub timestamp: u64,
}

/// Emitted when the dispute fee is refunded to the winning disputing party (issue #152).
#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisputeFeeRefunded {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub recipient: Address,
    pub fee_amount: i128,
    pub timestamp: u64,
}

/// Emitted when the dispute fee is sent to the treasury after a dismissed dispute (issue #152).
#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisputeFeeSlashed {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub treasury: Address,
    pub fee_amount: i128,
    pub timestamp: u64,
}

/// Emitted when a contributor's reputation increases after a milestone payout (issue #151).
#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReputationUpdated {
    pub event_version: u32,
    pub grant_id: u64,
    pub milestone_idx: u32,
    pub contributor: Address,
    pub new_reputation_score: u64,
    pub total_earned: i128,
    pub timestamp: u64,
}

/// Emitted when the DAO Council claws back escrowed funds from a fraudulent grant (issue #163).
#[contractevent]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantClawbacked {
    pub event_version: u32,
    pub grant_id: u64,
    /// The council address that initiated the clawback.
    pub council: Address,
    /// Total amount returned across all tokens.
    pub total_clawed_back: i128,
    pub timestamp: u64,
}
