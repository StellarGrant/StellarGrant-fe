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
    /// Vesting period has not yet elapsed; payout is still time-locked.
    VestingPeriodNotElapsed = 26,
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
    /// Approved and committed but waiting for vesting_period to elapse before payout.
    VestingPending = 6,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    pub metadata: u128, // Packed: [rejections(32) | approvals(32) | state(32) | idx(32)]
    pub description: String,
    pub amount: i128,
    pub votes: Map<Address, bool>,
    pub reasons: Map<Address, String>,
    pub status_updated_at: u64,
    pub proof_url: Option<String>,
    pub submission_timestamp: u64,
    pub deadline: u64,
    /// Time-lock (seconds) after approval before the payout can be claimed.
    /// `0` = pay immediately (no vesting). Non-zero amounts are held in escrow
    /// until `env.ledger().timestamp() >= status_updated_at + vesting_period`.
    pub vesting_period: u64,
}

impl Milestone {
    pub fn get_idx(&self) -> u32 {
        (self.metadata & 0xFFFFFFFF) as u32
    }
    pub fn set_idx(&mut self, idx: u32) {
        self.metadata = (self.metadata & !0xFFFFFFFF) | (idx as u128);
    }
    pub fn get_state(&self) -> MilestoneState {
        match ((self.metadata >> 32) & 0xFFFFFFFF) as u32 {
            1 => MilestoneState::Submitted,
            2 => MilestoneState::Approved,
            3 => MilestoneState::Paid,
            4 => MilestoneState::Rejected,
            5 => MilestoneState::Disputed,
            6 => MilestoneState::VestingPending,
            _ => MilestoneState::Pending,
        }
    }
    pub fn set_state(&mut self, state: MilestoneState) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 32)) | ((state as u32 as u128) << 32);
    }
    pub fn get_approvals(&self) -> u32 {
        ((self.metadata >> 64) & 0xFFFFFFFF) as u32
    }
    pub fn set_approvals(&mut self, approvals: u32) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 64)) | ((approvals as u128) << 64);
    }
    pub fn get_rejections(&self) -> u32 {
        ((self.metadata >> 96) & 0xFFFFFFFF) as u32
    }
    pub fn set_rejections(&mut self, rejections: u32) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 96)) | ((rejections as u128) << 96);
    }
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
    pub metadata: u128, // Packed: [milestones_paid_out(32) | total_milestones(32) | quorum(32) | status(32)]
    pub total_amount: i128,
    pub milestone_amount: i128,
    pub reviewers: Vec<Address>,
    pub escrow_balance: i128,
    pub funders: Vec<GrantFund>,
    pub reason: Option<String>,
    pub timestamp: u64,
}

impl Grant {
    pub fn get_status(&self) -> GrantStatus {
        match (self.metadata & 0xFFFFFFFF) as u32 {
            2 => GrantStatus::Cancelled,
            3 => GrantStatus::Completed,
            _ => GrantStatus::Active,
        }
    }
    pub fn set_status(&mut self, status: GrantStatus) {
        self.metadata =
            (self.metadata & !0xFFFFFFFF) | (status as u32 as u128);
    }
    pub fn get_quorum(&self) -> u32 {
        ((self.metadata >> 32) & 0xFFFFFFFF) as u32
    }
    pub fn set_quorum(&mut self, quorum: u32) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 32)) | ((quorum as u128) << 32);
    }
    pub fn get_total_milestones(&self) -> u32 {
        ((self.metadata >> 64) & 0xFFFFFFFF) as u32
    }
    pub fn set_total_milestones(&mut self, total_milestones: u32) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 64)) | ((total_milestones as u128) << 64);
    }
    pub fn get_milestones_paid_out(&self) -> u32 {
        ((self.metadata >> 96) & 0xFFFFFFFF) as u32
    }
    pub fn set_milestones_paid_out(&mut self, milestones_paid_out: u32) {
        self.metadata =
            (self.metadata & !(0xFFFFFFFF << 96)) | ((milestones_paid_out as u128) << 96);
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
