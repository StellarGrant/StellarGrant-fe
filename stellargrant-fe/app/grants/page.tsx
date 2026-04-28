 "use client";

import { useEffect, useState } from "react";
import { GrantCard } from "@/components/grants/GrantCard";

/**
 * Grant Listing Page
 * 
 * Paginated, filterable list of all grants stored on-chain.
 * 
 * Query Parameters:
 * - status: open | active | completed | cancelled
 * - token: XLM | USDC | all
 * - page: number (pagination)
 * - sort: newest | funded | deadline
 * - q: string (search query)
 */

type GrantListItem = {
  id: number;
  title: string;
  status: number;
  totalAmount: string;
  funded: number | bigint;
  budget: number | bigint;
  deadline: number | bigint;
  token?: string;
  owner?: string;
  hasOverdueMilestones?: boolean;
  milestoneSummary?: {
    total: number;
    submitted: number;
    overdue: number;
    upcoming: number;
    nextDeadline: string | null;
  };
};

export default function GrantsPage() {
  const [grants, setGrants] = useState<GrantListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    const loadGrants = async () => {
      try {
        setLoading(true);
        const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000";
        const response = await fetch(`${baseUrl}/grants`, {
          signal: controller.signal,
          cache: "no-store",
        });

        if (!response.ok) {
          throw new Error("Failed to load grants");
        }

        const payload = await response.json();
        setGrants(payload.data ?? []);
        setError(null);
      } catch (err) {
        if (controller.signal.aborted) return;
        setError(err instanceof Error ? err.message : "Failed to load grants");
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
        }
      }
    };

    void loadGrants();
    return () => controller.abort();
  }, []);

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="font-mono text-xs uppercase tracking-[0.32em] text-accent-secondary">
            Live Delivery Board
          </p>
          <h1 className="mt-3 text-3xl font-bold">Grants</h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-text-muted">
            Track open grants, see which milestone tracks are slipping, and jump straight into the creator work queue.
          </p>
        </div>
      </div>

      {loading && (
        <div className="grid gap-4 md:grid-cols-2">
          {Array.from({ length: 4 }).map((_, index) => (
            <div key={index} className="shimmer h-52 rounded-[4px]" />
          ))}
        </div>
      )}

      {error && (
        <div className="rounded-[4px] border border-danger/40 bg-danger/10 p-4 text-sm text-danger">
          {error}
        </div>
      )}

      {!loading && !error && (
        <div className="grid gap-4 md:grid-cols-2">
          {grants.map((grant) => (
            <GrantCard key={grant.id} grant={grant} />
          ))}
        </div>
      )}
    </div>
  );
}
