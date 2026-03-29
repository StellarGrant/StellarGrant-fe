/**
 * MilestoneList Component
 * 
 * Ordered list of milestones with status indicators.
 * Clicking a milestone navigates to its detail page.
 */

interface Milestone {
  idx: number;
  title: string;
  status: string;
  submitted: boolean;
  approved: boolean;
  paid: boolean;
}

interface MilestoneListProps {
  milestones: Milestone[];
  grantId: string;
}

export function MilestoneList({ milestones, grantId }: MilestoneListProps) {
  // TODO: Implement milestone list component
  const renderStatus = (milestone: Milestone) => {
    if (milestone.paid) return "Paid";
    if (milestone.approved) return "Approved";
    if (milestone.submitted) return "Submitted";
    return "Pending";
  };

  return (
    <div className="space-y-4">
      {milestones.map((milestone) => (
        <div key={milestone.idx} className="border rounded p-4">
          <h4 className="font-semibold">{milestone.title}</h4>
          <p className="text-sm text-gray-500">Status: {renderStatus(milestone)}</p>
        </div>
      ))}
    </div>
  );
}
