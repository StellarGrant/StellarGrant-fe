/**
 * GrantCard Component
 * 
 * Compact card for grant listing pages. Shows title, status badge,
 * funding progress, deadline, and token.
 */

import Link from "next/link";

interface GrantCardProps {
  grant: {
    id: number;
    title: string;
    status: string;
    totalAmount: string;
    milestoneSummary?: {
      total: number;
      submitted: number;
      overdue: number;
      upcoming: number;
      nextDeadline: string | null;
    };
    hasOverdueMilestones?: boolean;
  };
  onClick?: () => void;
  showOwner?: boolean;
  compact?: boolean;
}

const formatDate = (value: string | null) =>
  value
    ? new Date(value).toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
      })
    : "No pending deadline";

export function GrantCard({ grant, onClick, showOwner: _showOwner = false, compact: _compact = false }: GrantCardProps) {
  const summary = grant.milestoneSummary ?? {
    total: 0,
    submitted: 0,
    overdue: 0,
    upcoming: 0,
    nextDeadline: null,
  };

  return (
    <Link
      href={`/grants/${grant.id}/milestones`}
      onClick={onClick}
      className="group block rounded-[4px] border border-border-color bg-surface/80 p-5 transition-all duration-300 hover:border-accent-secondary/60 hover:shadow-[0_0_24px_rgba(59,130,246,0.12)]"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="font-mono text-[11px] uppercase tracking-[0.28em] text-text-muted">
            Grant #{grant.id}
          </p>
          <h3 className="mt-2 text-xl font-semibold text-text-primary group-hover:text-accent-primary">
            {grant.title}
          </h3>
        </div>
        <span
          className={`rounded-full px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] ${
            grant.hasOverdueMilestones
              ? "bg-danger/15 text-danger"
              : "bg-accent-secondary/15 text-accent-secondary"
          }`}
        >
          {grant.status}
        </span>
      </div>

      <div className="mt-5 grid gap-3 text-sm text-text-muted sm:grid-cols-3">
        <div>
          <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-text-muted/80">
            Pending Deadline
          </p>
          <p className="mt-1 text-text-primary">{formatDate(summary.nextDeadline)}</p>
        </div>
        <div>
          <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-text-muted/80">
            Milestones
          </p>
          <p className="mt-1 text-text-primary">
            {summary.submitted}/{summary.total} submitted
          </p>
        </div>
        <div>
          <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-text-muted/80">
            Attention Needed
          </p>
          <p className="mt-1 text-text-primary">
            {summary.overdue} overdue, {summary.upcoming} upcoming
          </p>
        </div>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {summary.overdue > 0 && (
          <span className="rounded-full border border-danger/30 bg-danger/10 px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] text-danger">
            {summary.overdue} overdue
          </span>
        )}
        {summary.upcoming > 0 && (
          <span className="rounded-full border border-warning/30 bg-warning/10 px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] text-warning">
            {summary.upcoming} due soon
          </span>
        )}
        <span className="rounded-full border border-border-color px-3 py-1 font-mono text-[11px] uppercase tracking-[0.22em] text-text-muted">
          {grant.totalAmount} stroops
        </span>
      </div>
    </Link>
  );
}
