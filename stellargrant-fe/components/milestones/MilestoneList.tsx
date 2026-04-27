/**
 * MilestoneList Component
 * 
 * Ordered list of milestones with status indicators.
 * Clicking a milestone navigates to its detail page.
 */

import Link from "next/link";

interface Milestone {
  idx: number;
  title: string;
  description: string | null;
  deadline: string;
  submitted: boolean;
  overdue: boolean;
  daysUntilDeadline: number;
}

interface MilestoneListProps {
  milestones: Milestone[];
  grantId: string;
}

const formatDeadline = (value: string) =>
  new Date(value).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

export function MilestoneList({ milestones, grantId }: MilestoneListProps) {
  const renderStatus = (milestone: Milestone) => {
    if (milestone.submitted) return "Submitted";
    if (milestone.overdue) return "Overdue";
    if (milestone.daysUntilDeadline <= 7) return "Due Soon";
    return "Pending";
  };

  return (
    <div className="space-y-4">
      {milestones.map((milestone) => (
        <Link
          key={milestone.idx}
          href={`/grants/${grantId}/milestones/${milestone.idx}`}
          className={`block rounded-[4px] border p-5 transition-colors ${
            milestone.overdue
              ? "border-danger/40 bg-danger/10"
              : milestone.submitted
                ? "border-success/30 bg-success/10"
                : "border-border-color bg-surface/70 hover:border-accent-secondary/60"
          }`}
        >
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="font-mono text-[11px] uppercase tracking-[0.26em] text-text-muted">
                Milestone {milestone.idx + 1}
              </p>
              <h4 className="mt-2 text-lg font-semibold text-text-primary">{milestone.title}</h4>
              {milestone.description && (
                <p className="mt-2 max-w-2xl text-sm leading-6 text-text-muted">
                  {milestone.description}
                </p>
              )}
            </div>
            <span
              className={`rounded-full px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] ${
                milestone.overdue
                  ? "bg-danger/15 text-danger"
                  : milestone.submitted
                    ? "bg-success/15 text-success"
                    : "bg-warning/15 text-warning"
              }`}
            >
              {renderStatus(milestone)}
            </span>
          </div>

          <div className="mt-4 flex flex-wrap gap-4 text-sm text-text-muted">
            <span>Deadline: {formatDeadline(milestone.deadline)}</span>
            <span>
              {milestone.submitted
                ? "Proof received"
                : milestone.overdue
                  ? `${Math.abs(milestone.daysUntilDeadline)} day(s) late`
                  : `${milestone.daysUntilDeadline} day(s) remaining`}
            </span>
          </div>
        </Link>
      ))}
    </div>
  );
}
