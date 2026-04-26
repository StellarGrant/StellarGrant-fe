/**
 * Interface for signing transactions.
 * Implement this to integrate with various wallets (e.g., Freighter, Albedo).
 */
export interface WalletAdapter {
  /**
   * Returns the public key of the signer.
   */
  getPublicKey(): Promise<string>;
  /**
   * Signs a transaction XDR and returns the signed version.
   * @param txXdr The base64 encoded transaction XDR.
   * @param networkPassphrase The passphrase of the network the transaction is for.
   */
  signTransaction(txXdr: string, networkPassphrase: string): Promise<string>;
}

/**
 * Configuration options for the StellarGrants SDK.
 */
export type StellarGrantsSDKConfig = {
  /** The ID of the StellarGrants contract on the network. */
  contractId: string;
  /** The URL of the Soroban RPC server. */
  rpcUrl: string;
  /** The network passphrase (e.g., "Test SDF Network ; September 2015"). */
  networkPassphrase: string;
  /** The signer (wallet adapter) used to authorize transactions. */
  signer: WalletAdapter;
  /** Default fee to use for transactions (in stroops). Defaults to "100". */
  defaultFee?: string;
  /** Polling interval in milliseconds when waiting for transactions. Defaults to 1000. */
  pollingIntervalMs?: number;
  /** Maximum time in milliseconds to wait for a transaction confirmation. Defaults to 30000. */
  pollingTimeoutMs?: number;
};

/**
 * Input for creating a new grant.
 */
export type GrantCreateInput = {
  /** The address that will own the grant. */
  owner: string;
  /** The title of the grant project. */
  title: string;
  /** A detailed description of the grant. */
  description: string;
  /** The total budget for the grant (in base units of the token). */
  budget: bigint;
  /** The deadline for the grant as a UNIX timestamp (seconds). */
  deadline: bigint;
  /** The number of milestones required for the grant. */
  milestoneCount: number;
};

/**
 * Input for funding an existing grant.
 */
export type GrantFundInput = {
  /** The unique numeric ID of the grant. */
  grantId: number;
  /** The address of the token being used for funding. */
  token: string;
  /** The amount to fund (in base units of the token). */
  amount: bigint;
};

/**
 * Input for submitting a milestone proof.
 */
export type MilestoneSubmitInput = {
  /** The unique numeric ID of the grant. */
  grantId: number;
  /** The index of the milestone (0-based). */
  milestoneIdx: number;
  /** The hash of the proof or documentation for the milestone. */
  proofHash: string;
};

/**
 * Input for voting on a milestone.
 */
export type MilestoneVoteInput = {
  /** The unique numeric ID of the grant. */
  grantId: number;
  /** The index of the milestone (0-based). */
  milestoneIdx: number;
  /** Whether to approve (true) or reject (false) the milestone. */
  approve: boolean;
};

/**
 * Options for state-changing transaction invocations.
 */
export type WriteOptions = {
  /** Optional multiplier for the simulated resource fee. */
  feeMultiplier?: number;
  /** Pre-calculated Soroban transaction data. */
  transactionData?: any; // xdr.SorobanTransactionData
  /** Explicit fee to use, bypassing automatic calculation. */
  simulatedFee?: string;
};
