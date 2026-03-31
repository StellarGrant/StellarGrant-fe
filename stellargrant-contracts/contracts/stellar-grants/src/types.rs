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
    /// Contract is globally paused; all state-modifying operations are blocked.
    ContractPaused = 33,
    /// Donation would exceed the grant's hard cap.
    CapReached = 34,
    /// Grant has more than 5 tags.
    TooManyTags = 35,
    /// A tag exceeds 20 characters.
    TagTooLong = 36,
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
    pub packed_stats: u128,
}

impl EscrowState {
    pub fn new(mode: EscrowMode, lifecycle: EscrowLifecycleState, quorum_ready: bool, approvals_count: u32) -> Self {
        let mut state = Self { packed_stats: 0 };
        state.set_mode(mode);
        state.set_lifecycle(lifecycle);
        state.set_quorum_ready(quorum_ready);
        state.set_approvals_count(approvals_count);
        state
    }

    pub fn mode(&self) -> EscrowMode {
        match (self.packed_stats & 0xFFFFFFFF) as u32 {
            1 => EscrowMode::Standard,
            2 => EscrowMode::HighSecurity,
            _ => EscrowMode::Standard,
        }
    }

    pub fn set_mode(&mut self, mode: EscrowMode) {
        self.packed_stats = (self.packed_stats & !0xFFFFFFFF) | (mode as u32 as u128);
    }

    pub fn lifecycle(&self) -> EscrowLifecycleState {
        match ((self.packed_stats >> 32) & 0xFFFFFFFF) as u32 {
            1 => EscrowLifecycleState::Funding,
            2 => EscrowLifecycleState::AwaitingMultisig,
            3 => EscrowLifecycleState::Released,
            _ => EscrowLifecycleState::Funding,
        }
    }

    pub fn set_lifecycle(&mut self, lifecycle: EscrowLifecycleState) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 32)) | ((lifecycle as u32 as u128) << 32);
    }

    pub fn quorum_ready(&self) -> bool {
        ((self.packed_stats >> 64) & 1) != 0
    }

    pub fn set_quorum_ready(&mut self, ready: bool) {
        let b = if ready { 1u128 } else { 0u128 };
        self.packed_stats = (self.packed_stats & !(1 << 64)) | (b << 64);
    }

    pub fn approvals_count(&self) -> u32 {
        ((self.packed_stats >> 96) & 0xFFFFFFFF) as u32
    }

    pub fn set_approvals_count(&mut self, count: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 96)) | ((count as u128) << 96);
    }
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
    /// Quorum reached, but payment is delayed by a challenge period.
    AwaitingPayout = 8,
    /// An AwaitingPayout milestone was challenged by a funder.
    Challenged = 9,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub idx: u32,
    pub description: String,
    pub amount: i128,
    pub payout_token: Address,
    pub votes: Map<Address, bool>,
    pub reasons: Map<Address, String>,
    pub status_updated_at: u64,
    pub proof_url: Option<String>,
    pub submission_timestamp: u64,
    pub deadline: u64,
    pub community_comments: Map<Address, String>,
    pub packed_stats: u128,
}

impl Milestone {
    pub fn new(idx: u32, description: String, amount: i128, payout_token: Address, deadline: u64, env: &soroban_sdk::Env) -> Self {
        let mut m = Self {
            idx,
            description,
            amount,
            payout_token,
            votes: Map::new(env),
            reasons: Map::new(env),
            status_updated_at: 0,
            proof_url: None,
            submission_timestamp: 0,
            deadline,
            community_comments: Map::new(env),
            packed_stats: 0,
        };
        m.set_state(MilestoneState::Pending);
        m.set_approvals(0);
        m.set_rejections(0);
        m.set_community_upvotes(0);
        m
    }

    pub fn state(&self) -> MilestoneState {
        match (self.packed_stats & 0xFFFFFFFF) as u32 {
            0 => MilestoneState::Pending,
            1 => MilestoneState::Submitted,
            2 => MilestoneState::Approved,
            3 => MilestoneState::Paid,
            4 => MilestoneState::Rejected,
            5 => MilestoneState::Disputed,
            6 => MilestoneState::Resolved,
            7 => MilestoneState::CommunityReview,
            _ => MilestoneState::Pending,
        }
    }

    pub fn set_state(&mut self, state: MilestoneState) {
        self.packed_stats = (self.packed_stats & !0xFFFFFFFF) | (state as u32 as u128);
    }

    pub fn approvals(&self) -> u32 {
        ((self.packed_stats >> 32) & 0xFFFFFFFF) as u32
    }

    pub fn set_approvals(&mut self, approvals: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 32)) | ((approvals as u128) << 32);
    }

    pub fn rejections(&self) -> u32 {
        ((self.packed_stats >> 64) & 0xFFFFFFFF) as u32
    }

    pub fn set_rejections(&mut self, rejections: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 64)) | ((rejections as u128) << 64);
    }

    pub fn community_upvotes(&self) -> u32 {
        ((self.packed_stats >> 96) & 0xFFFFFFFF) as u32
    }

    pub fn set_community_upvotes(&mut self, upvotes: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 96)) | ((upvotes as u128) << 96);
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneSubmission {
    pub idx: u32,
    pub description: String,
    pub proof: String,
    pub payout_token: Option<Address>, // New: Optional override for the payout token
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
    pub token: Address, // New: Specify which token was contributed
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    pub id: u64,
    pub owner: Address,
    pub title: String,
    pub description: String,
    pub primary_token: Address,
    pub total_amount: i128,
    pub milestone_amount: i128,
    pub reviewers: Vec<Address>,
    pub escrow_balances: Map<Address, i128>,
    pub funders: Vec<GrantFund>,
    pub reason: Option<String>,
    pub timestamp: u64,
    pub cancellation_requested_at: Option<u64>,
    pub last_heartbeat: u64,
    pub min_funding: i128,
    pub packed_stats: u128,
}

impl Grant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        owner: Address,
        title: String,
        description: String,
        primary_token: Address,
        total_amount: i128,
        milestone_amount: i128,
        reviewers: Vec<Address>,
        status: GrantStatus,
        quorum: u32,
        total_milestones: u32,
        timestamp: u64,
        min_funding: i128,
        env: &soroban_sdk::Env,
    ) -> Self {
        let mut g = Self {
            id,
            owner,
            title,
            description,
            primary_token,
            total_amount,
            milestone_amount,
            reviewers,
            escrow_balances: Map::new(env),
            funders: Vec::new(env),
            reason: None,
            timestamp,
            cancellation_requested_at: None,
            last_heartbeat: timestamp,
            min_funding,
            packed_stats: 0,
        };
        g.set_status(status);
        g.set_quorum(quorum);
        g.set_total_milestones(total_milestones);
        g.set_milestones_paid_out(0);
        g
    }

    pub fn status(&self) -> GrantStatus {
        match (self.packed_stats & 0xFFFFFFFF) as u32 {
            1 => GrantStatus::Active,
            2 => GrantStatus::Cancelled,
            3 => GrantStatus::Completed,
            4 => GrantStatus::CancellationPending,
            5 => GrantStatus::Paused,
            6 => GrantStatus::Inactive,
            7 => GrantStatus::PendingFunding,
            8 => GrantStatus::PendingAcceptance,
            _ => GrantStatus::Active,
        }
    }

    pub fn set_status(&mut self, status: GrantStatus) {
        self.packed_stats = (self.packed_stats & !0xFFFFFFFF) | (status as u32 as u128);
    }

    pub fn quorum(&self) -> u32 {
        ((self.packed_stats >> 32) & 0xFFFFFFFF) as u32
    }

    pub fn set_quorum(&mut self, quorum: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 32)) | ((quorum as u128) << 32);
    }

    pub fn total_milestones(&self) -> u32 {
        ((self.packed_stats >> 64) & 0xFFFFFFFF) as u32
    }

    pub fn set_total_milestones(&mut self, total: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 64)) | ((total as u128) << 64);
    }

    pub fn milestones_paid_out(&self) -> u32 {
        ((self.packed_stats >> 96) & 0xFFFFFFFF) as u32
    }

    pub fn set_milestones_paid_out(&mut self, paid: u32) {
        self.packed_stats = (self.packed_stats & !(0xFFFFFFFF << 96)) | ((paid as u128) << 96);
    }
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
