# Milestone Deadlines and Expiry Logic

## Overview
The StellarGrant smart contract supports optional milestone deadlines. Grant creators can set specific ledger timestamps as delivery deadlines for individual milestones. Once a deadline passes (`ledger_timestamp > deadline_timestamp`), the milestone is considered expired. Expired milestones cannot be submitted or voted upon, and their allocated funds can be reclaimed by the grant funders.

## Design Decisions
- **Decoupled Storage:** Deadlines are kept outside the core `Grant` and `Milestone` structs.
- **Tombstone State (`Expired`):** When funds are reclaimed, a dummy `Milestone` struct is written with an `Expired` state. This safely tomb-stones the milestone index, preventing double-withdrawals or subsequent late submissions.
- **Immutable Deadlines:** Once a deadline is explicitly set for a milestone, it cannot be modified or overwritten. 

## How It Works
1. **Setting Deadlines:** The grant owner calls `set_milestone_deadline` before the milestone is submitted. The deadline must be a future ledger timestamp.
2. **Submission Verification:** During `milestone_submit`, the contract checks the current ledger timestamp against the stored deadline. If the deadline has passed, submission is rejected with `ContractError::DeadlinePassed`.
3. **Reclaiming Funds:** If a milestone expires unsubmitted, any authorized funder or the owner can call `claim_expired_funds`. 
4. **Fund Distribution:** The `milestone_amount` is refunded proportionally to all contributors based on their active funding share. The `grant.escrow_balance` is updated, and `MilestoneExpired` / `FundsReclaimed` events are emitted.

## Security Considerations
- **Immutability:** Deadlines are immutable once set, preventing malicious grant creators from shortening deadlines at the last minute to grief recipients.
- **Timestamp Security:** Relying on `env.ledger().timestamp()` is secure within the Stellar Consensus Protocol bounds.
- **Double Claim Prevention:** Generating the `Expired` tombstone reliably intercepts duplicate reclaim attempts on the same milestone index.

## Backward Compatibility
This feature is fully backward compatible. Deadlines are stored using a discrete key: `DataKey::MilestoneDeadline(grant_id, milestone_idx)`. The existing persistent layout of `Grant` and `Milestone` records remains completely untouched, ensuring no disruption to already-deployed data.

## Test Coverage
The expiry logic is rigorously verified by unit tests in the Soroban test environment (`test.rs`), including:
- Submission validation exactly at, before, and after deadline boundaries.
- Rejection of deadline overwrite attempts.
- Rejection of past-timestamp inputs.
- Validated voting flows for on-time submissions reviewed post-deadline.
- Precision checks on proportional refunds across multiple funders.
- Explicit prevention of fractional or double-claims.
