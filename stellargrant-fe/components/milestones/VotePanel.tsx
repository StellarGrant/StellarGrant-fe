/**
 * VotePanel Component
 * 
 * Displays the reviewer vote tally for a milestone.
 * Shows quorum progress bar, individual reviewer votes,
 * and Approve/Reject action buttons for eligible reviewers.
 */

interface MilestoneVote {
  reviewer: string;
  vote: "approve" | "reject" | null;
}

interface VotePanelProps {
  grantId: string;
  milestoneIdx: number;
  votes: MilestoneVote[];
  quorum: number; // required vote count
  threshold: number; // % required for approval (e.g. 0.67)
  connectedAddress?: string;
}

export function VotePanel({
  grantId: _grantId,
  milestoneIdx: _milestoneIdx,
  votes: _votes,
  quorum: _quorum,
  threshold: _threshold,
  connectedAddress: _connectedAddress,
}: VotePanelProps) {
  // TODO: Implement vote panel component
  return (
    <div>
      <h3 className="text-lg font-semibold mb-4">Votes</h3>
      {/* Vote panel will be implemented here */}
    </div>
  );
}
