use soroban_sdk::{contracterror, contracttype, Address, Map, String, Vec};

/// Contract error types
#[contracterror]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    GrantNotFound = 1,
    Unauthorized = 2,
    MilestoneAlreadyApproved = 3,
    QuorumNotReached = 4,
    DeadlinePassed = 5,
    InvalidInput = 6,
    MilestoneNotSubmitted = 7,
    AlreadyVoted = 8,
    MilestoneNotFound = 9,
    InvalidState = 10,
    NoRefundableAmount = 11,
    NotAllMilestonesApproved = 12,
    AlreadyRegistered = 13,
    MilestoneAlreadySubmitted = 14,
    InsufficientStake = 15,
    StakeNotFound = 16,
    NotVerified = 17,
    BatchEmpty = 18,
    BatchTooLarge = 19,
    ReentrancyDetected = 20,
    NotMultisigSigner = 21,
    AlreadySignedRelease = 22,
    ReleaseNotReady = 23,
    GrantAlreadyReleased = 24,
    InsufficientReputation = 25,
    /// Reviewer vote rejected because the community review period has not elapsed yet.
    CommunityReviewPeriod = 26,
    /// The voter has already upvoted this milestone.
    AlreadyUpvoted = 27,
    /// Grant cancellation is pending; grace period has not elapsed yet.
    CancellationGracePeriod = 28,
    HeartbeatMissed = 29,
    Blacklisted = 30,
    /// Caller is not the contract global admin for this operation.
    NotContractAdmin = 31,
    InsufficientBalance = 32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EscrowMode {
    Standard = 1,
    HighSecurity = 2,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EscrowLifecycleState {
    Funding = 1,
    AwaitingMultisig = 2,
    Released = 3,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscrowState {
    pub mode: EscrowMode,
    pub lifecycle: EscrowLifecycleState,
    pub quorum_ready: bool,
    pub approvals_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MilestoneState {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Paid = 3,
    Rejected = 4,
    Disputed = 5,
    Resolved = 6,
    /// Open for community upvotes / comments before reviewer voting begins.
    CommunityReview = 7,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub idx: u32,
    pub description: String,
    pub amount: i128,
    pub state: MilestoneState,
    pub votes: Map<Address, bool>,
    pub approvals: u32,
    pub rejections: u32,
    pub reasons: Map<Address, String>,
    pub status_updated_at: u64,
    pub proof_url: Option<String>,
    pub submission_timestamp: u64,
    pub deadline: u64,
    /// Number of community upvotes received during the CommunityReview period.
    pub community_upvotes: u32,
    /// One comment per address recorded during the CommunityReview period.
    pub community_comments: Map<Address, String>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneSubmission {
    pub idx: u32,
    pub description: String,
    pub proof: String,
}

#[contracttype]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GrantStatus {
    Active = 1,
    Cancelled = 2,
    Completed = 3,
    /// Cancellation requested but grace period has not elapsed yet.
    CancellationPending = 4,
    /// Grant is temporarily paused; no funding, submissions, or payouts allowed.
    Paused = 5,
    /// Grant became inactive due to missed heartbeats; can be restored via grant_ping.
    Inactive = 6,
    /// Grant is waiting to reach its minimum funding threshold before becoming Active.
    PendingFunding = 7,
    /// Grant has been created but not yet accepted by the recipient (owner).
    /// No funding is allowed until the grant transitions out of this state.
    PendingAcceptance = 8,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantFund {
    pub funder: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    pub id: u64,
    pub owner: Address,
    pub title: String,
    pub description: String,
    pub token: Address,
    pub status: GrantStatus,
    pub total_amount: i128,
    pub milestone_amount: i128,
    pub reviewers: Vec<Address>,
    pub quorum: u32,
    pub total_milestones: u32,
    pub milestones_paid_out: u32,
    pub escrow_balance: i128,
    pub funders: Vec<GrantFund>,
    pub reason: Option<String>,
    pub timestamp: u64,
    /// Timestamp when a cancellation was first requested (grace-period cancellation).
    pub cancellation_requested_at: Option<u64>,
    pub last_heartbeat: u64,
    /// Minimum escrow balance required before the grant transitions from PendingFunding to Active.
    pub min_funding: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributorProfile {
    pub contributor: Address,
    pub name: String,
    pub bio: String,
    pub skills: Vec<String>,
    pub github_url: String,
    pub registration_timestamp: u64,
    pub reputation_score: u64,
    pub grants_count: u32,
    pub total_earned: i128,
}
