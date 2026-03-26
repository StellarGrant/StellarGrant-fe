use soroban_sdk::{contracterror, contracttype, Address, Map, String, Vec};

/// Contract error types
#[contracterror]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    MilestoneAlreadyApproved = 2,
    QuorumNotReached = 3,
    DeadlinePassed = 4,
    InvalidInput = 5,
    MilestoneNotSubmitted = 6,
    AlreadyVoted = 7,
    MilestoneNotFound = 8,
    InvalidState = 9,
    NoRefundableAmount = 10,
    NotAllMilestonesApproved = 11,
    MilestoneAlreadySubmitted = 12,
    NotInitialized = 13,
    AlreadyInitialized = 14,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MilestoneState {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Rejected = 3,
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
    pub owner: Address,
    pub title: String,
    pub description: String,
    pub token: Address,
    pub status: GrantStatus,
    pub total_amount: i128,
    pub milestone_amount: i128,
    pub reviewers: Vec<Address>,
    pub total_milestones: u32,
    pub milestones_paid_out: u32,
    pub escrow_balance: i128,
    pub funders: Vec<GrantFund>,
    pub reason: Option<String>,
    pub timestamp: u64,
}
