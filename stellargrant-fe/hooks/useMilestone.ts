/**
 * useMilestone Hook
 * 
 * Fetches milestone state for a specific grant + milestone index.
 * Includes vote data and submission proof.
 */

export function useMilestone( _grantId: string,  _milestoneIdx: number) {
  // TODO: Implement milestone fetching hook
  return {
    milestone: null,
    votes: [],
    isReviewer: false,
    hasVoted: false,
    isLoading: false,
    error: null,
  };
}
