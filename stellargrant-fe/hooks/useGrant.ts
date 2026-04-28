"use client";

/**
 * useGrant Hook
 *
 * Fetches a single grant by ID and keeps it fresh via polling.
 * Handles loading and error states out of the box.
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { Grant } from "@/types";
import { logger } from "@/lib/logger";

interface UseGrantOptions {
  refetchInterval?: number; // default: 30_000 ms
  enabled?: boolean;        // default: true
}

interface UseGrantResult {
  data: Grant | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

const hookLogger = logger.child("useGrant");

export function useGrant(grantId: string, options?: UseGrantOptions): UseGrantResult {
  const { refetchInterval = 30_000, enabled = true } = options ?? {};

  const [data, setData] = useState<Grant | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const fetchGrant = useCallback(async () => {
    if (!enabled || !grantId) return;
    abortRef.current?.abort();
    abortRef.current = new AbortController();

    setIsLoading(true);
    setError(null);
    hookLogger.debug("Fetching grant", { grantId });

    try {
      const res = await fetch(`/api/grants/${grantId}`, {
        signal: abortRef.current.signal,
      });
      if (!res.ok) throw new Error(`Failed to fetch grant ${grantId}: ${res.status}`);
      const json = await res.json() as { grant: Grant };
      hookLogger.debug("Grant fetched", { grantId, status: json.grant?.status });
      setData(json.grant);
    } catch (err) {
      if ((err as { name?: string }).name === "AbortError") return;
      const error = err instanceof Error ? err : new Error(String(err));
      hookLogger.error("Error fetching grant", { grantId, error: error.message });
      setError(error);
    } finally {
      setIsLoading(false);
    }
  }, [grantId, enabled]);

  useEffect(() => {
    void fetchGrant();
    if (!enabled || refetchInterval <= 0) return;
    const id = setInterval(() => void fetchGrant(), refetchInterval);
    return () => {
      clearInterval(id);
      abortRef.current?.abort();
    };
  }, [fetchGrant, enabled, refetchInterval]);

  return { data, isLoading, error, refetch: fetchGrant };
}
