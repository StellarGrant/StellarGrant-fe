/**
 * Grant Detail Page
 *
 * Full grant page showing metadata, funding progress, milestone list,
 * reviewer panel, and event history.
 */

"use client";

import { useEffect, useState } from "react";
import { use, Suspense } from "react";
import { FundingProgress } from "@/components/grants/FundingProgress";
import { MilestoneList } from "@/components/milestones/MilestoneList";
import { GrantStatusBadge } from "@/components/grants/GrantStatusBadge";
import { formatTokenAmount, getTokenMetadata, TokenMetadata } from "@/lib/tokens";
import { Grant, Milestone } from "@/types";

interface GrantDetailPageProps {
  params: Promise<{
    id: string;
  }>;
}

// Mock data for demonstration - will be replaced with actual hook calls
function useMockGrant(grantId: string) {
  const [grant, setGrant] = useState<Grant | null>(null);
  const [milestones, setMilestones] = useState<Milestone[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Simulate loading - replace with actual useGrant hook when implemented
    setTimeout(() => {
      setGrant({
        id: grantId,
        owner: "GABC123...",
        title: "Sample Grant",
        description: "This is a sample grant description",
        budget: 10000000000n, // 1000 XLM in stroops
        funded: 7500000000n, // 750 XLM
        deadline: BigInt(Math.floor(Date.now() / 1000) + 86400 * 30),
        status: 2, // In Progress
        milestones: 3,
        reviewers: ["GDEF456..."],
        created_at: BigInt(Math.floor(Date.now() / 1000)),
        token: "native",
      });

      setMilestones([
        {
          idx: 0,
          title: "Phase 1: Research",
          description: "Complete initial research and documentation",
          proof_hash: "QmABC123...",
          submitted: true,
          approved: true,
          paid: true,
          submitted_at: 1700000000n,
          approved_at: 1700000100n,
          paid_at: 1700000200n,
          token: "native",
          amount: 3000000000n, // 300 XLM
        },
        {
          idx: 1,
          title: "Phase 2: Development",
          description: "Build the core features",
          proof_hash: null,
          submitted: false,
          approved: false,
          paid: false,
          submitted_at: null,
          approved_at: null,
          paid_at: null,
          token: "USDC",
          amount: 500000000n, // 500 USDC (6 decimals)
        },
        {
          idx: 2,
          title: "Phase 3: Launch",
          description: "Deploy and launch the project",
          proof_hash: null,
          submitted: false,
          approved: false,
          paid: false,
          submitted_at: null,
          approved_at: null,
          paid_at: null,
          token: "native",
          amount: 2000000000n, // 200 XLM
        },
      ]);

      setIsLoading(false);
    }, 500);
  }, [grantId]);

  return { grant, milestones, isLoading };
}

function GrantDetailContent({ grantId }: { grantId: string }) {
  const { grant, milestones, isLoading } = useMockGrant(grantId);
  const [tokenMetadata, setTokenMetadata] = useState<TokenMetadata | null>(null);

  useEffect(() => {
    async function fetchMetadata() {
      if (grant?.token) {
        const metadata = await getTokenMetadata(grant.token);
        setTokenMetadata(metadata);
      }
    }
    fetchMetadata();
  }, [grant?.token]);

  if (isLoading || !grant) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse space-y-6">
          <div className="h-8 bg-gray-200 rounded w-1/3"></div>
          <div className="h-32 bg-gray-200 rounded"></div>
          <div className="h-48 bg-gray-200 rounded"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl">
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-3xl font-bold">{grant.title}</h1>
          <GrantStatusBadge status={grant.status} />
        </div>
        <p className="text-gray-600">{grant.description}</p>
      </div>

      {/* Funding Section */}
      <section className="mb-8 p-6 bg-white rounded-lg shadow">
        <h2 className="text-xl font-semibold mb-4">Funding Progress</h2>
        <FundingProgress
          current={grant.funded}
          target={grant.budget}
          token={grant.token}
        />
        <div className="mt-4 grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-500">Owner:</span>
            <p className="font-mono text-sm">{grant.owner}</p>
          </div>
          <div>
            <span className="text-gray-500">Primary Token:</span>
            <p className="font-medium">{tokenMetadata?.symbol ?? "UNKNOWN"}</p>
          </div>
        </div>
      </section>

      {/* Milestones Section */}
      <section className="mb-8 p-6 bg-white rounded-lg shadow">
        <h2 className="text-xl font-semibold mb-4">Milestones</h2>
        <MilestoneList
          milestones={milestones}
          grantId={grant.id}
          grantToken={grant.token}
        />
      </section>

      {/* Info Section */}
      <section className="p-6 bg-white rounded-lg shadow">
        <h2 className="text-xl font-semibold mb-4">Grant Details</h2>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <span className="text-gray-500">Budget:</span>
            <p className="font-medium">
              {tokenMetadata
                ? formatTokenAmount(grant.budget, tokenMetadata.decimals, {
                    symbol: tokenMetadata.symbol,
                    showSymbol: true,
                  })
                : grant.budget.toString()}
            </p>
          </div>
          <div>
            <span className="text-gray-500">Deadline:</span>
            <p className="font-medium">
              {new Date(Number(grant.deadline) * 1000).toLocaleDateString()}
            </p>
          </div>
          <div>
            <span className="text-gray-500">Milestones:</span>
            <p className="font-medium">{grant.milestones}</p>
          </div>
          <div>
            <span className="text-gray-500">Created:</span>
            <p className="font-medium">
              {new Date(Number(grant.created_at) * 1000).toLocaleDateString()}
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

export default function GrantDetailPage({ params }: GrantDetailPageProps) {
  const { id } = use(params);

  return (
    <Suspense fallback={
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse space-y-6">
          <div className="h-8 bg-gray-200 rounded w-1/3"></div>
          <div className="h-32 bg-gray-200 rounded"></div>
          <div className="h-48 bg-gray-200 rounded"></div>
        </div>
      </div>
    }>
      <GrantDetailContent grantId={id} />
    </Suspense>
  );
}
