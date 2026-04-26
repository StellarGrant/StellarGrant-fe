export class StellarGrantsError extends Error {
  readonly code: string;
  readonly details?: unknown;

  constructor(message: string, code = "STELLAR_GRANTS_ERROR", details?: unknown) {
    super(message);
    this.name = "StellarGrantsError";
    this.code = code;
    this.details = details;
  }
}

export class SorobanRevertError extends StellarGrantsError {
  constructor(message: string, details?: unknown) {
    super(message, "SOROBAN_REVERT", details);
    this.name = "SorobanRevertError";
  }
}

// Contract-specific error codes mapped from Soroban contract
export const CONTRACT_ERROR_CODES = {
  GrantNotFound: 1,
  Unauthorized: 2,
  MilestoneAlreadyApproved: 3,
  QuorumNotReached: 4,
  DeadlinePassed: 5,
  InvalidInput: 6,
  MilestoneNotSubmitted: 7,
  AlreadyVoted: 8,
  MilestoneNotFound: 9,
  InvalidState: 10,
  NoRefundableAmount: 11,
  NotAllMilestonesApproved: 12,
  AlreadyRegistered: 13,
  MilestoneAlreadySubmitted: 14,
  InsufficientStake: 15,
  StakeNotFound: 16,
  NotVerified: 17,
  BatchEmpty: 18,
  BatchTooLarge: 19,
  ReentrancyDetected: 20,
  NotMultisigSigner: 21,
  AlreadySignedRelease: 22,
  ReleaseNotReady: 23,
  GrantAlreadyReleased: 24,
  InsufficientReputation: 25,
  CommunityReviewPeriod: 26,
  AlreadyUpvoted: 27,
  CancellationGracePeriod: 28,
  HeartbeatMissed: 29,
  Blacklisted: 30,
  NotContractAdmin: 31,
  InsufficientBalance: 32,
  ContractPaused: 33,
  CapReached: 34,
  TooManyTags: 35,
  TagTooLong: 36,
  DisputeFeeInsufficient: 37,
  DisputeAlreadyCharged: 38,
  ExtensionDenied: 39,
  DeadlineNotSet: 40,
  ExpiryNotReached: 41,
  RoleAlreadyAssigned: 42,
  RoleNotAssigned: 43,
  HeartbeatNotStale: 44,
  DuplicateBountySubmitter: 45,
  ContributorProfileRequired: 46,
  BountySubmissionsCap: 47,
  InvalidTokenInterface: 48,
} as const;

// Reverse map for looking up error names from codes
const ERROR_CODE_TO_NAME: Record<number, string> = Object.entries(CONTRACT_ERROR_CODES).reduce(
  (acc, [name, code]) => {
    acc[code] = name;
    return acc;
  },
  {} as Record<number, string>
);

// Specific error classes for each contract error type
export class GrantNotFoundError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Grant not found", details);
    this.name = "GrantNotFoundError";
  }
}

export class UnauthorizedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Unauthorized: insufficient permissions for this operation", details);
    this.name = "UnauthorizedError";
  }
}

export class MilestoneAlreadyApprovedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Milestone has already been approved", details);
    this.name = "MilestoneAlreadyApprovedError";
  }
}

export class QuorumNotReachedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Vote quorum not reached", details);
    this.name = "QuorumNotReachedError";
  }
}

export class DeadlinePassedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Deadline has passed for this operation", details);
    this.name = "DeadlinePassedError";
  }
}

export class InvalidInputError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Invalid input provided", details);
    this.name = "InvalidInputError";
  }
}

export class MilestoneNotSubmittedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Milestone proof not yet submitted", details);
    this.name = "MilestoneNotSubmittedError";
  }
}

export class AlreadyVotedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("You have already voted on this milestone", details);
    this.name = "AlreadyVotedError";
  }
}

export class MilestoneNotFoundError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Milestone not found", details);
    this.name = "MilestoneNotFoundError";
  }
}

export class InvalidStateError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Invalid state for this operation", details);
    this.name = "InvalidStateError";
  }
}

export class NoRefundableAmountError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("No refundable amount available", details);
    this.name = "NoRefundableAmountError";
  }
}

export class NotAllMilestonesApprovedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Not all milestones have been approved", details);
    this.name = "NotAllMilestonesApprovedError";
  }
}

export class AlreadyRegisteredError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("User is already registered", details);
    this.name = "AlreadyRegisteredError";
  }
}

export class MilestoneAlreadySubmittedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Milestone proof has already been submitted", details);
    this.name = "MilestoneAlreadySubmittedError";
  }
}

export class InsufficientStakeError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Insufficient stake amount", details);
    this.name = "InsufficientStakeError";
  }
}

export class StakeNotFoundError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Stake not found", details);
    this.name = "StakeNotFoundError";
  }
}

export class NotVerifiedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("User is not verified", details);
    this.name = "NotVerifiedError";
  }
}

export class BatchEmptyError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Batch is empty", details);
    this.name = "BatchEmptyError";
  }
}

export class BatchTooLargeError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Batch size exceeds limit", details);
    this.name = "BatchTooLargeError";
  }
}

export class ReentrancyDetectedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Reentrant call detected", details);
    this.name = "ReentrancyDetectedError";
  }
}

export class NotMultisigSignerError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Not a valid multisig signer", details);
    this.name = "NotMultisigSignerError";
  }
}

export class AlreadySignedReleaseError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Release has already been signed", details);
    this.name = "AlreadySignedReleaseError";
  }
}

export class ReleaseNotReadyError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Release is not ready", details);
    this.name = "ReleaseNotReadyError";
  }
}

export class GrantAlreadyReleasedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Grant funds have already been released", details);
    this.name = "GrantAlreadyReleasedError";
  }
}

export class InsufficientReputationError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Insufficient reputation score", details);
    this.name = "InsufficientReputationError";
  }
}

export class CommunityReviewPeriodError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Community review period has not elapsed yet", details);
    this.name = "CommunityReviewPeriodError";
  }
}

export class AlreadyUpvotedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("You have already upvoted this milestone", details);
    this.name = "AlreadyUpvotedError";
  }
}

export class CancellationGracePeriodError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Cancellation grace period has not elapsed yet", details);
    this.name = "CancellationGracePeriodError";
  }
}

export class HeartbeatMissedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Heartbeat missed - grant may be inactive", details);
    this.name = "HeartbeatMissedError";
  }
}

export class BlacklistedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("User is blacklisted", details);
    this.name = "BlacklistedError";
  }
}

export class NotContractAdminError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Caller is not the contract admin", details);
    this.name = "NotContractAdminError";
  }
}

export class InsufficientBalanceError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Insufficient token balance", details);
    this.name = "InsufficientBalanceError";
  }
}

export class ContractPausedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Contract is currently paused", details);
    this.name = "ContractPausedError";
  }
}

export class CapReachedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Grant has reached its funding cap", details);
    this.name = "CapReachedError";
  }
}

export class TooManyTagsError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Grant has too many tags (max 5)", details);
    this.name = "TooManyTagsError";
  }
}

export class TagTooLongError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Tag exceeds maximum length (20 characters)", details);
    this.name = "TagTooLongError";
  }
}

export class DisputeFeeInsufficientError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Insufficient balance to pay dispute fee", details);
    this.name = "DisputeFeeInsufficientError";
  }
}

export class DisputeAlreadyChargedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Dispute fee has already been charged for this milestone", details);
    this.name = "DisputeAlreadyChargedError";
  }
}

export class ExtensionDeniedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Grant extension request denied", details);
    this.name = "ExtensionDeniedError";
  }
}

export class DeadlineNotSetError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Deadline has not been set", details);
    this.name = "DeadlineNotSetError";
  }
}

export class ExpiryNotReachedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Expiry date has not been reached yet", details);
    this.name = "ExpiryNotReachedError";
  }
}

export class RoleAlreadyAssignedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Role is already assigned", details);
    this.name = "RoleAlreadyAssignedError";
  }
}

export class RoleNotAssignedError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Role is not assigned to this user", details);
    this.name = "RoleNotAssignedError";
  }
}

export class HeartbeatNotStaleError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Heartbeat is not stale yet", details);
    this.name = "HeartbeatNotStaleError";
  }
}

export class DuplicateBountySubmitterError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Bounty submitter already exists", details);
    this.name = "DuplicateBountySubmitterError";
  }
}

export class ContributorProfileRequiredError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Contributor profile is required", details);
    this.name = "ContributorProfileRequiredError";
  }
}

export class BountySubmissionsCapError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Bounty submissions cap reached", details);
    this.name = "BountySubmissionsCapError";
  }
}

export class InvalidTokenInterfaceError extends SorobanRevertError {
  constructor(details?: unknown) {
    super("Invalid token interface", details);
    this.name = "InvalidTokenInterfaceError";
  }
}

// Map of error code to error class
export const ERROR_CLASSES: Record<number, new (details?: unknown) => SorobanRevertError> = {
  1: GrantNotFoundError,
  2: UnauthorizedError,
  3: MilestoneAlreadyApprovedError,
  4: QuorumNotReachedError,
  5: DeadlinePassedError,
  6: InvalidInputError,
  7: MilestoneNotSubmittedError,
  8: AlreadyVotedError,
  9: MilestoneNotFoundError,
  10: InvalidStateError,
  11: NoRefundableAmountError,
  12: NotAllMilestonesApprovedError,
  13: AlreadyRegisteredError,
  14: MilestoneAlreadySubmittedError,
  15: InsufficientStakeError,
  16: StakeNotFoundError,
  17: NotVerifiedError,
  18: BatchEmptyError,
  19: BatchTooLargeError,
  20: ReentrancyDetectedError,
  21: NotMultisigSignerError,
  22: AlreadySignedReleaseError,
  23: ReleaseNotReadyError,
  24: GrantAlreadyReleasedError,
  25: InsufficientReputationError,
  26: CommunityReviewPeriodError,
  27: AlreadyUpvotedError,
  28: CancellationGracePeriodError,
  29: HeartbeatMissedError,
  30: BlacklistedError,
  31: NotContractAdminError,
  32: InsufficientBalanceError,
  33: ContractPausedError,
  34: CapReachedError,
  35: TooManyTagsError,
  36: TagTooLongError,
  37: DisputeFeeInsufficientError,
  38: DisputeAlreadyChargedError,
  39: ExtensionDeniedError,
  40: DeadlineNotSetError,
  41: ExpiryNotReachedError,
  42: RoleAlreadyAssignedError,
  43: RoleNotAssignedError,
  44: HeartbeatNotStaleError,
  45: DuplicateBountySubmitterError,
  46: ContributorProfileRequiredError,
  47: BountySubmissionsCapError,
  48: InvalidTokenInterfaceError,
};

/**
 * Get the error name from an error code
 */
export function getErrorName(code: number): string | undefined {
  return ERROR_CODE_TO_NAME[code];
}

/**
 * Get the error class for an error code
 */
export function getErrorClass(code: number): new (details?: unknown) => SorobanRevertError | undefined {
  return ERROR_CLASSES[code];
}
