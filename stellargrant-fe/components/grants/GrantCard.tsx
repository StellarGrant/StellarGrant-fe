/**
 * GrantCard Component
 *
 * Compact card for grant listing pages. Shows title, status badge,
 * funding progress, deadline, and token.
 */

"use client";

import { useEffect, useState } from "react";
import { formatTokenAmount, getTokenMetadata, TokenMetadata } from "@/lib/tokens";
import { GrantStatusBadge } from "./GrantStatusBadge";
import { FundingProgress } from "./FundingProgress";

interface GrantCardProps {
  grant: {
    id: number;
    title: string;
    status: number;
    funded: bigint | number;
    budget: bigint | number;
    deadline: bigint | number;
    token?: string;
    owner?: string;
  };
  onClick?: () => void;
  showOwner?: boolean;
  compact?: boolean;
}

export function GrantCard({ grant, onClick, showOwner = false, compact = false }: GrantCardProps) {
  const [tokenMetadata, setTokenMetadata] = useState<TokenMetadata | null>(null);
  useEffect(() => {
    async function fetchMetadata() {
      if (grant.token) {
        const metadata = await getTokenMetadata(grant.token);
        setTokenMetadata(metadata);
      }
    }
    fetchMetadata();
  }, [grant.token]);

  const decimals = tokenMetadata?.decimals ?? 7;
  const symbol = tokenMetadata?.symbol ?? (grant.token ? "UNKNOWN" : "XLM");

  const fundedFormatted = formatTokenAmount(grant.funded, decimals, { symbol, showSymbol: true });
  const budgetFormatted = formatTokenAmount(grant.budget, decimals, { symbol, showSymbol: true });

  const deadlineDate = typeof grant.deadline === "bigint"
    ? new Date(Number(grant.deadline) * 1000)
    : new Date(grant.deadline);

  return (
    <div
      className="border rounded-lg p-4 cursor-pointer hover:shadow-md transition-shadow bg-white"
      onClick={onClick}
    >
      <div className="flex justify-between items-start mb-3">
        <h3 className="text-xl font-semibold flex-1">{grant.title}</h3>
        <GrantStatusBadge status={grant.status} />
      </div>

      {!compact && (
        <>
          <FundingProgress
            current={grant.funded}
            target={grant.budget}
            token={grant.token}
            showBreakdown={false}
          />

          <div className="mt-4 flex justify-between text-sm text-gray-600">
            <span>
              Target: <span className="font-medium">{budgetFormatted}</span>
            </span>
            <span>
              Deadline:{" "}
              <span className="font-medium">
                {deadlineDate.toLocaleDateString()}
              </span>
            </span>
          </div>

          {showOwner && grant.owner && (
            <div className="mt-2 text-xs text-gray-500">
              Owner: <span className="font-mono">{grant.owner.slice(0, 8)}...{grant.owner.slice(-8)}</span>
            </div>
          )}
        </>
      )}

      {compact && (
        <div className="mt-2 text-sm text-gray-600">
          <span className="font-medium">{fundedFormatted}</span> raised
        </div>
      )}
    </div>
  );
}
