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
  /** The network passphrase (e.g., "Test SDF Network ; September 2015"). If omitted, it is resolved from the RPC server. */
  networkPassphrase?: string;
  /** The signer (wallet adapter) used to authorize transactions. Optional for read-only usage. */
  signer?: WalletAdapter;
  /** Default fee to use for transactions (in stroops). Defaults to "100". */
  defaultFee?: string;
  /** Polling interval in milliseconds when waiting for transactions. Defaults to 1000. */
  pollingIntervalMs?: number;
  /** Maximum time in milliseconds to wait for a transaction confirmation. Defaults to 30000. */
  pollingTimeoutMs?: number;
  /**
   * Custom HTTP headers forwarded to every RPC request.
   * Use for authentication tokens, API keys, or enterprise gateway requirements.
   *
   * @example { "X-Api-Key": "my-secret" }
   */
  customHeaders?: Record<string, string>;
  /**
   * Optional proxy URL that intercepts all RPC traffic.
   * When set, the SDK routes every RPC call through this URL instead of
   * `rpcUrl`. Useful in environments where direct RPC access is blocked by
   * CORS or firewall policies.
   *
   * @example "https://my-proxy.example.com/stellar-rpc"
   */
  proxyUrl?: string;
  /**
   * Optional Horizon base URL used for dynamic fee recommendations via
   * `/fee_stats`. If omitted, static multipliers are used.
   */
  horizonUrl?: string;
  /**
   * Optional explicit endpoint for fee stats. Takes precedence over
   * `${horizonUrl}/fee_stats` when provided.
   */
  feeStatsEndpoint?: string;
};

/** Result of an allowance check. */
export type AllowanceResult = {
  /** Current approved amount (in base token units). */
  amount: bigint;
  /** Ledger sequence at which the allowance expires (0 = does not expire). */
  expirationLedger: number;
};

/** Result returned by `checkAndSetAllowance`. */
export type AllowanceCheckResult = {
  /** Whether the allowance was already sufficient (no transaction needed). */
  sufficient: boolean;
  /** Current allowance before any update. */
  current: bigint;
  /** The required amount that was checked against. */
  required: bigint;
};

/** Pinata IPFS upload configuration. */
export type IpfsUploadConfig = {
  /** Pinata API JWT (preferred) or API key for authentication. */
  pinataJwt?: string;
  /** Pinata API key (legacy). Use `pinataJwt` when available. */
  pinataApiKey?: string;
  /** Pinata API secret (legacy, required alongside `pinataApiKey`). */
  pinataSecretKey?: string;
  /** Optional display name for the pinned object. */
  name?: string;
  /** Explicit schema name for metadata validation before upload. */
  metadataSchema?: IpfsMetadataSchemaName;
  /** Disable local schema validation in exceptional flows. */
  skipSchemaValidation?: boolean;
};

/** Supported built-in metadata schemas for IPFS artifacts. */
export type IpfsMetadataSchemaName = "grant" | "milestone";

/** Result of a successful IPFS upload. */
export type IpfsUploadResult = {
  /** IPFS Content Identifier. */
  cid: string;
  /** Public gateway URL for convenient browser access. */
  gatewayUrl: string;
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
 * Fee priority tiers used by the SDK when estimating transaction fees.
 *
 * - `"low"`    – 1.0× the simulated resource fee. Cheapest but may be slow
 *               during network congestion.
 * - `"medium"` – 1.5× the simulated resource fee (default). Balances cost
 *               and inclusion speed.
 * - `"high"`   – 2.0× the simulated resource fee. Prioritises fast inclusion
 *               at higher cost.
 */
export type FeePriority = "low" | "medium" | "high";

/**
 * Per-priority fee estimate returned by `StellarGrantsSDK.estimateFees()`.
 */
export type FeeEstimate = {
  /** Raw simulated resource fee (in stroops) before any multiplier. */
  base: string;
  /** Optional dynamic recommended base fee derived from recent network stats. */
  recommendedBase?: string;
  /** Fee at low priority using the effective low-tier modifier. */
  low: string;
  /** Fee at medium priority using the effective medium-tier modifier. */
  medium: string;
  /** Fee at high priority using the effective high-tier modifier. */
  high: string;
  /** Effective per-tier multipliers used for this estimate. */
  modifiers: FeeLevelModifiers;
  /** Network load bucket used to derive modifiers. */
  networkLoad: FeeNetworkLoad;
  /** Source used to derive fee modifiers. */
  source: "horizon" | "simulation-fallback";
};

/** Frontend-safe per-tier multipliers returned with fee estimates. */
export type FeeLevelModifiers = {
  low: number;
  medium: number;
  high: number;
};

/** Coarse network load buckets for dynamic fee guidance. */
export type FeeNetworkLoad = "low" | "moderate" | "high" | "surge";

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
  /**
   * Fee priority tier. When set this takes precedence over `feeMultiplier`
   * (unless `simulatedFee` is also provided, which always wins).
   *
   * Defaults to `"medium"` when neither `feeMultiplier` nor `simulatedFee`
   * is specified.
   */
  feePriority?: FeePriority;
};

/**
 * Structured representation of a Grant returned by the on-chain contract.
 */
export type GrantData = {
  id: number;
  owner?: string;
  title?: string;
  description?: string;
  budget?: bigint | string | number;
  deadline?: bigint | string | number;
  milestoneCount?: number;
  status?: string;
  [k: string]: unknown;
};

/**
 * Structured representation of a Milestone returned by the on-chain contract.
 */
export type MilestoneData = {
  grantId?: number;
  idx?: number;
  title?: string;
  proofHash?: string;
  approved?: boolean;
  approvals?: number;
  status?: string;
  [k: string]: unknown;
};
