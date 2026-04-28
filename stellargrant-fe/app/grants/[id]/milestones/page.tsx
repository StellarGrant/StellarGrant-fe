"use client";

import { useEffect, useState } from "react";
import { MilestoneList } from "@/components/milestones";
import { Milestone } from "@/types";

/**
 * Milestone List Page
 * 
 * Shows all milestones for a grant with their status and progress.
 */

interface MilestonesPageProps {
  params: {
    id: string;
  };
}

export default function MilestonesPage({ params }: MilestonesPageProps) {
  const [milestones, setMilestones] = useState<Milestone[]>([]);
  const [title, setTitle] = useState(`Grant #${params.id}`);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    const loadGrant = async () => {
      try {
        setLoading(true);
        const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000";
        const response = await fetch(`${baseUrl}/grants/${params.id}`, {
          signal: controller.signal,
          cache: "no-store",
        });

        if (!response.ok) {
          throw new Error("Failed to load milestones");
        }

        const payload = await response.json();
        setMilestones(payload.data?.milestones ?? []);
        setTitle(payload.data?.title ?? `Grant #${params.id}`);
        setError(null);
      } catch (err) {
        if (controller.signal.aborted) return;
        setError(err instanceof Error ? err.message : "Failed to load milestones");
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false);
        }
      }
    };

    void loadGrant();
    return () => controller.abort();
  }, [params.id]);

  return (
    <div className="container mx-auto px-4 py-8">
      <p className="font-mono text-xs uppercase tracking-[0.32em] text-accent-secondary">
        Creator Timeline
      </p>
      <h1 className="mb-3 mt-3 text-3xl font-bold">Milestones - {title}</h1>
      <p className="mb-8 max-w-2xl text-sm leading-6 text-text-muted">
        Upcoming deadlines, overdue work, and submitted proofs are grouped here so creators can see what needs attention first.
      </p>

      {loading && <div className="shimmer h-40 rounded-[4px]" />}
      {error && (
        <div className="rounded-[4px] border border-danger/40 bg-danger/10 p-4 text-sm text-danger">
          {error}
        </div>
      )}
      {!loading && !error && <MilestoneList milestones={milestones} grantId={params.id} />}
    </div>
  );
}
