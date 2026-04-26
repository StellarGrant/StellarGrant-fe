/**
 * useGrant Hook
 * 
 * Fetches a single grant by ID from the contract using TanStack Query.
 * Automatically refetches on focus and every 30 seconds.
 */

interface UseGrantOptions {
  refetchInterval?: number; // default: 30_000ms
  enabled?: boolean; // default: true
}

export function useGrant( _grantId: string,  _options?: UseGrantOptions) {
  // TODO: Implement grant fetching hook with TanStack Query
  return {
    data: null,
    isLoading: false,
    error: null,
    refetch: async () => {},
  };
}
