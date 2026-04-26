/**
 * useContractEvents Hook
 * 
 * Subscribes to Soroban contract events for a specific grant ID
 * via Server-Sent Events. Returns an array of decoded events
 * in chronological order.
 */

interface ContractEvent {
  type: "GrantFunded" | "MilestoneSubmitted" | "MilestoneApproved" | "PayoutReleased" | string;
  data: Record<string, unknown>;
  ledger: number;
  timestamp: Date;
}

interface UseContractEventsOptions {
  grantId: string;
}

export function useContractEvents({ grantId: _grantId }: UseContractEventsOptions) {
  // TODO: Implement contract events hook with SSE
  return {
    events: [] as ContractEvent[],
    isConnected: false,
    error: null,
  };
}
