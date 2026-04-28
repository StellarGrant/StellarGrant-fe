"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useWalletStore } from "@/lib/store/walletStore";

/**
 * Contributor Profile Page
 * 
 * Shows the connected wallet's contributor profile:
 * - Registered GitHub handle
 * - Skills
 * - Grants participated in
 * - Milestones completed
 * - Reputation score fetched from contract
 */

type DeadlineItem = {
  grantId: number;
  grantTitle: string;
  idx: number;
  title: string;
  deadline: string;
  daysUntilDeadline: number;
};

type DashboardPayload = {
  summary: {
    upcomingCount: number;
    overdueCount: number;
  };
  upcomingDeadlines: DeadlineItem[];
  overdueMilestones: DeadlineItem[];
};

const formatDate = (value: string) =>
  new Date(value).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

export default function ProfilePage() {
  const address = useWalletStore((state) => state.address);
  const [dashboard, setDashboard] = useState<DashboardPayload | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!address) {
      setDashboard(null);
      setLoading(false);
      return;
    }

    const controller = new AbortController();
    const loadDashboard = async () => {
      try {
        setLoading(true);
        const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000";
        const response = await fetch(`${baseUrl}/dashboard/${address}`, {
          signal: controller.signal,
          cache: "no-store",
        });

        if (!response.ok) {
          throw new Error("Failed to load dashboard");
        }

        const payload = await response.json();
        setDashboard(payload.data);
        setError(null);
      } catch (err) {
        if (controller.signal.aborted) return;
        setError(err instanceof Error ? err.message : "Failed to load dashboard");
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
        }
      }
    };

    void loadDashboard();
    return () => controller.abort();
  }, [address]);

  return (
    <div className="container mx-auto px-4 py-8">
      <p className="font-mono text-xs uppercase tracking-[0.32em] text-accent-secondary">
        Creator Dashboard
      </p>
      <h1 className="mb-3 mt-3 text-3xl font-bold">Profile</h1>
      <p className="mb-8 max-w-2xl text-sm leading-6 text-text-muted">
        Stay ahead of milestone commitments with a single view of what is due soon and what already needs follow-up.
      </p>

      {!address && (
        <div className="rounded-[4px] border border-border-color bg-surface/80 p-6">
          <h2 className="text-xl font-semibold">Connect a creator wallet</h2>
          <p className="mt-2 text-sm leading-6 text-text-muted">
            Deadline alerts are personalized per grant creator. Once a wallet is connected, upcoming and overdue milestones will appear here automatically.
          </p>
        </div>
      )}

      {loading && <div className="shimmer h-48 rounded-[4px]" />}
      {error && (
        <div className="rounded-[4px] border border-danger/40 bg-danger/10 p-4 text-sm text-danger">
          {error}
        </div>
      )}

      {address && dashboard && !loading && !error && (
        <div className="space-y-6">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="rounded-[4px] border border-warning/30 bg-warning/10 p-5">
              <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-warning">
                Due Soon
              </p>
              <p className="mt-3 text-3xl font-bold text-text-primary">{dashboard.summary.upcomingCount}</p>
            </div>
            <div className="rounded-[4px] border border-danger/30 bg-danger/10 p-5">
              <p className="font-mono text-[11px] uppercase tracking-[0.24em] text-danger">
                Overdue
              </p>
              <p className="mt-3 text-3xl font-bold text-text-primary">{dashboard.summary.overdueCount}</p>
            </div>
          </div>

          <section className="rounded-[4px] border border-border-color bg-surface/80 p-6">
            <h2 className="text-xl font-semibold">Upcoming Deadlines</h2>
            <div className="mt-4 space-y-3">
              {dashboard.upcomingDeadlines.length === 0 && (
                <p className="text-sm text-text-muted">No upcoming milestone deadlines in the next cycle.</p>
              )}
              {dashboard.upcomingDeadlines.map((item) => (
                <Link
                  key={`${item.grantId}-${item.idx}`}
                  href={`/grants/${item.grantId}/milestones`}
                  className="flex flex-wrap items-center justify-between gap-3 rounded-[4px] border border-border-color px-4 py-3 hover:border-accent-secondary/50"
                >
                  <div>
                    <p className="font-medium text-text-primary">{item.title}</p>
                    <p className="text-sm text-text-muted">{item.grantTitle}</p>
                  </div>
                  <div className="text-right text-sm text-text-muted">
                    <p>{formatDate(item.deadline)}</p>
                    <p>{item.daysUntilDeadline} day(s) remaining</p>
                  </div>
                </Link>
              ))}
            </div>
          </section>

          <section className="rounded-[4px] border border-danger/30 bg-danger/5 p-6">
            <h2 className="text-xl font-semibold">Overdue Milestones</h2>
            <div className="mt-4 space-y-3">
              {dashboard.overdueMilestones.length === 0 && (
                <p className="text-sm text-text-muted">Nothing overdue right now.</p>
              )}
              {dashboard.overdueMilestones.map((item) => (
                <Link
                  key={`${item.grantId}-${item.idx}`}
                  href={`/grants/${item.grantId}/milestones`}
                  className="flex flex-wrap items-center justify-between gap-3 rounded-[4px] border border-danger/30 px-4 py-3 hover:border-danger/60"
                >
                  <div>
                    <p className="font-medium text-text-primary">{item.title}</p>
                    <p className="text-sm text-text-muted">{item.grantTitle}</p>
                  </div>
                  <div className="text-right text-sm text-danger">
                    <p>{formatDate(item.deadline)}</p>
                    <p>{Math.abs(item.daysUntilDeadline)} day(s) overdue</p>
                  </div>
                </Link>
              ))}
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
