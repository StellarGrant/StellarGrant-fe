/**
 * Global TypeScript Types
 * 
 * Shared type definitions for the StellarGrants frontend.
 */

export interface Grant {
  id: string;
  owner: string;
  title: string;
  description: string;
  budget: bigint;
  funded: bigint;
  deadline: bigint;
  status: number; // 0: Pending, 1: Active, 2: In Progress, 3: Complete, 4: Cancelled
  milestones: number;
  reviewers: string[];
  created_at: bigint;
}

export interface Milestone {
  idx: number;
  title: string;
  description: string;
  proof_hash: string | null;
  submitted: boolean;
  approved: boolean;
  paid: boolean;
  submitted_at: bigint | null;
  approved_at: bigint | null;
  paid_at: bigint | null;
}

export interface MilestoneVote {
  reviewer: string;
  vote: "approve" | "reject" | null;
  voted_at: bigint | null;
}

export interface Contributor {
  address: string;
  github_handle: string | null;
  skills: string[];
  reputation_score: number;
  grants_participated: number;
  milestones_completed: number;
}

export interface ContractEvent {
  type: string;
  data: Record<string, unknown>;
  ledger: number;
  timestamp: Date;
}
