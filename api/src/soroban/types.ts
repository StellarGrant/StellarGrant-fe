export type SorobanGrant = {
  id: number;
  title: string;
  status: string;
  recipient: string;
  totalAmount: string;
  /** Comma-separated tag string, e.g. "web3,climate,open-source" */
  tags?: string | null;
  localizedMetadata?: Record<string, { title?: string; description?: string }> | null;
};

export type ContributorScore = {
  address: string;
  reputation: number;
  totalEarned: string;
};

/**
 * Normalised on-chain event emitted by the StellarGrants contract.
 * Maps to the contract's event topics/data as decoded by the SDK.
 */
export type SorobanContractEvent = {
  /** Ledger sequence the event was emitted in */
  ledger: number;
  /** Unique identifier: "<ledger>-<txHash>-<eventIdx>" */
  id: string;
  /** Event topic, e.g. "grant_created", "grant_funded", "milestone_submitted" */
  type: string;
  /** Grant ID this event relates to */
  grantId: number;
  /** Stellar address of the actor */
  actorAddress: string | null;
  /** Arbitrary event payload decoded from the contract */
  data: Record<string, unknown>;
};

export interface SorobanContractClient {
  fetchGrants(): Promise<SorobanGrant[]>;
  fetchGrantById(id: number): Promise<SorobanGrant | null>;
  fetchContributorScore(address: string): Promise<ContributorScore | null>;
  /**
   * Fetch contract events for a ledger range (inclusive).
   * Implementations should cap the range to avoid oversized responses.
   */
  fetchEvents(fromLedger: number, toLedger: number): Promise<SorobanContractEvent[]>;
  /** Returns the latest confirmed ledger sequence on the network */
  getLatestLedger(): Promise<number>;
}
