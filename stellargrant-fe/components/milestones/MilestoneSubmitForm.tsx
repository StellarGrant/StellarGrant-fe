/**
 * MilestoneSubmitForm Component
 * 
 * Form for contributors to submit milestone proof.
 * Accepts text description and IPFS file upload.
 * Builds and submits the milestone_submit() contract transaction.
 */

interface MilestoneSubmitFormProps {
  grantId: string;
  milestoneIdx: number;
}

export function MilestoneSubmitForm({ grantId: _grantId, milestoneIdx: _milestoneIdx }: MilestoneSubmitFormProps) {
  // TODO: Implement milestone submission form
  return (
    <form>
      {/* Milestone submission form will be implemented here */}
    </form>
  );
}
