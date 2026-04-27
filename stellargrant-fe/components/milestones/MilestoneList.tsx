/**
 * MilestoneList Component
 *
 * Ordered list of milestones with status indicators.
 * Displays token and payout amount for each milestone.
 * Clicking a milestone navigates to its detail page.
 */

"use client";

import { useEffect, useState } from "react";
import { formatTokenAmount, getTokenMetadata, TokenMetadata } from "@/lib/tokens";
import { Milestone as MilestoneType } from "@/types";

interface MilestoneListProps {
  milestones: MilestoneType[];
  grantId: string;
  grantToken?: string; // Fallback token if milestone doesn't specify one
}

interface MilestoneDisplay extends MilestoneType {
  statusLabel: string;
  tokenSymbol: string;
  amountFormatted: string;
}

export function MilestoneList({ milestones, grantId, grantToken }: MilestoneListProps) {
  const [tokenMetadataMap, setTokenMetadataMap] = useState<Map<string, TokenMetadata>>(new Map());
  const [isLoading, setIsLoading] = useState(true);

  // Collect all unique tokens from milestones
  const uniqueTokens = Array.from(
    new Set(
      milestones
        .map((m) => m.token || grantToken)
        .filter((t): t is string => !!t)
    )
  );

  // Fetch metadata for all tokens
  useEffect(() => {
    async function fetchMetadata() {
      setIsLoading(true);
      const metadataMap = new Map<string, TokenMetadata>();

      for (const token of uniqueTokens) {
        const metadata = await getTokenMetadata(token);
        metadataMap.set(token, metadata);
      }

      setTokenMetadataMap(metadataMap);
      setIsLoading(false);
    }

    fetchMetadata();
  }, [uniqueTokens]);

  const getMilestoneStatus = (milestone: MilestoneType): string => {
    if (milestone.paid) return "Paid";
    if (milestone.approved) return "Approved";
    if (milestone.submitted) return "Submitted";
    if (milestone.overdue) return "Overdue";
    if (milestone.daysUntilDeadline <= 7) return "Due Soon";
    return "Pending";
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case "Paid":
        return "bg-green-100 text-green-800";
      case "Approved":
        return "bg-blue-100 text-blue-800";
      case "Submitted":
        return "bg-yellow-100 text-yellow-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  };

  const formatMilestoneAmount = (milestone: MilestoneType): string => {
    const token = milestone.token || grantToken;
    if (!token || milestone.amount === undefined || milestone.amount === null) {
      return "";
    }

    const metadata = tokenMetadataMap.get(token);
    const decimals = metadata?.decimals ?? 7;
    const symbol = metadata?.symbol ?? "UNKNOWN";

    return formatTokenAmount(milestone.amount, decimals, { symbol, showSymbol: true });
  };

  const getTokenSymbol = (milestone: MilestoneType): string => {
    const token = milestone.token || grantToken;
    if (!token) return "XLM";

    const metadata = tokenMetadataMap.get(token);
    return metadata?.symbol ?? "UNKNOWN";
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="animate-pulse space-y-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="border rounded p-4 bg-gray-50">
              <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2"></div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {milestones.map((milestone) => {
        const statusLabel = getMilestoneStatus(milestone);
        const statusColor = getStatusColor(statusLabel);
        const amountFormatted = formatMilestoneAmount(milestone);
        const tokenSymbol = getTokenSymbol(milestone);

        return (
          <div
            key={milestone.idx}
            className="border rounded-lg p-4 hover:shadow-md transition-shadow bg-white"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-3 mb-2">
                  <span className="text-sm font-mono text-gray-500">
                    #{milestone.idx}
                  </span>
                  <h4 className="font-semibold text-lg">{milestone.title}</h4>
                </div>
                <p className="text-sm text-gray-600 mb-3">{milestone.description}</p>
                <div className="flex items-center gap-4 text-sm">
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusColor}`}>
                    {statusLabel}
                  </span>
                  {amountFormatted && (
                    <span className="text-gray-700 font-medium">
                      Payout: {amountFormatted}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
