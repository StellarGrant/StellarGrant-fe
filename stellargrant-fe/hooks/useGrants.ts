/**
 * useGrants Hook
 * 
 * Fetches paginated grant list with optional filters.
 * Returns grants array, pagination metadata, and loading state.
 */

interface UseGrantsOptions {
  status?: "open" | "active" | "completed" | "cancelled";
  token?: "XLM" | "USDC" | "all";
  sort?: "newest" | "funded" | "deadline";
  page?: number;
  q?: string;
}

export function useGrants( _options?: UseGrantsOptions) {
  // TODO: Implement grants list hook with TanStack Query
  return {
    data: {
      pages: [],
      pageParams: [],
    },
    fetchNextPage: async () => {},
    hasNextPage: false,
    isLoading: false,
    error: null,
  };
}
