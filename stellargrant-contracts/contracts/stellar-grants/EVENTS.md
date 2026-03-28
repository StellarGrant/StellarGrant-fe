# Stellar Grants Event Schema

This contract emits typed Soroban `#[contractevent]` events with a consistent schema for indexers.

## Standard Fields

Every event payload includes:

- `event_version: u32` - Schema version for event consumers. Current value: `1`.
- `grant_id: u64` - Grant identifier for filtering and joins.
  - For contract-level events not tied to a specific grant, `grant_id = 0`.
- `timestamp: u64` - Ledger timestamp when the event was emitted.

## Contract Lifecycle Events

- `ContractInitialized`
  - Emitted once when `initialize` completes (after global admin and council are stored).
  - Fields: `event_version`, `grant_id` (always `0` for contract-level events), `council`, `timestamp`.

- `ContractUpgraded`
  - Emitted when contract-wide configuration changes without swapping WASM.
  - Current emit points:
    - `admin_change` with `component = "admin_changed"`.
    - `set_council` with `component = "council_updated"`.
  - Fields: `event_version`, `grant_id`, `actor`, `component`, `timestamp`.

## Grant and Milestone Events

Grant and milestone events keep business fields minimal and include the standard metadata fields to improve off-chain indexing:

- `GrantCreated`
- `GrantFunded`
- `GrantMetadataUpdated`
- `GrantCancelled`
- `GrantCompleted`
- `MilestoneSubmitted`
- `MilestoneVoted`
- `MilestoneRejected`
- `MilestoneStatusChanged`
- `MilestonePaid`
- `MilestoneExpired`
- `RefundIssued`
- `RefundExecuted`
- `FinalRefund`
- `ContributorRegistered`
- `ReputationIncreased`

## Indexing Guidance

- Filter first by event type and `grant_id`.
- Use `event_version` to support future schema changes without breaking old parsers.
- Prefer lightweight event payloads and read full state from storage only when necessary.
