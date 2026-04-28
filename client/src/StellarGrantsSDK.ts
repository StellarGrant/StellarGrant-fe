import {
  Account,
  Contract,
  rpc,
  TransactionBuilder,
  nativeToScVal,
  scValToNative,
  xdr,
} from "@stellar/stellar-sdk";
import { parseSorobanError } from "./errors/parseSorobanError";
import { StellarGrantsError } from "./errors/StellarGrantsError";
import {
  AllowanceCheckResult,
  AllowanceResult,
  FeeEstimate,
  FeeLevelModifiers,
  FeeNetworkLoad,
  FeePriority,
  GrantCreateInput,
  GrantFundInput,
  MilestoneSubmitInput,
  MilestoneVoteInput,
  StellarGrantsSDKConfig,
  WalletAdapter,
  WriteOptions,
} from "./types";
import { EventParser, ParsedEvent } from "./events";

const READ_ONLY_SIMULATION_ACCOUNT =
  "GB3KJPLFUYN5VL6R3GU3EGCGVCKFDSD7BEDX42HWG5BWFKB3KQGJJRMA";

/**
 * The SDK's current contract interface version.
 *
 * Increment this whenever the on-chain contract ABI changes in a way that is
 * incompatible with this SDK. `checkCompatibility()` compares this value
 * against the version stored in the deployed contract.
 */
export const CONTRACT_INTERFACE_VERSION = 1;

/** Default multipliers applied to `minResourceFee` for each priority tier. */
const DEFAULT_FEE_LEVEL_MODIFIERS: FeeLevelModifiers = {
  low: 1.0,
  medium: 1.5,
  high: 2.0,
};

const DYNAMIC_FEE_LEVELS: Array<{
  maxUsage: number;
  load: FeeNetworkLoad;
  modifiers: FeeLevelModifiers;
}> = [
  { maxUsage: 0.5, load: "low", modifiers: { low: 0.9, medium: 1.2, high: 1.6 } },
  { maxUsage: 0.8, load: "moderate", modifiers: { low: 1.0, medium: 1.5, high: 2.0 } },
  { maxUsage: 0.95, load: "high", modifiers: { low: 1.2, medium: 1.8, high: 2.6 } },
  { maxUsage: Infinity, load: "surge", modifiers: { low: 1.6, medium: 2.5, high: 3.5 } },
];

/**
 * Encapsulated client for StellarGrants Soroban contract interactions.
 *
 * This SDK provides a high-level interface to interact with the StellarGrants
 * smart contract. It handles transaction building, simulation, signing (via a
 * provided signer), and submission.
 *
 * @example
 * ```typescript
 * const sdk = new StellarGrantsSDK({
 *   contractId: "CD...",
 *   rpcUrl: "https://soroban-testnet.stellar.org",
 *   signer: freighterSigner
 * });
 * ```
 */
export class StellarGrantsSDK {
  private readonly contract: Contract;
  private readonly server: rpc.Server;
  private readonly config: StellarGrantsSDKConfig;
  private networkPassphrasePromise?: Promise<string>;

  /**
   * Initializes a new instance of the StellarGrantsSDK.
   * @param config Configuration options including contract ID, RPC URL, and optional signer.
   */
  constructor(config: StellarGrantsSDKConfig) {
    this.config = config;
    this.contract = new Contract(config.contractId);

    // #274 — Route all RPC traffic through proxyUrl when provided. Merge any
    // custom headers into the fetch options so authenticated endpoints work
    // in restricted network environments.
    const effectiveUrl = config.proxyUrl ?? config.rpcUrl;
    const fetchOptions: RequestInit | undefined = config.customHeaders
      ? { headers: config.customHeaders }
      : undefined;

    this.server = new rpc.Server(effectiveUrl, {
      allowHttp: effectiveUrl.startsWith("http://"),
      ...(fetchOptions && { fetchOptions }),
    });
  }

  /**
   * Creates a new grant in the system.
   *
   * @param input Details of the grant to create.
   * @param options Optional transaction configuration.
   * @returns A promise that resolves to the transaction submission result.
   */
  async grantCreate(input: GrantCreateInput, options?: WriteOptions): Promise<rpc.Api.SendTransactionResponse> {
    return this.invokeWrite("grant_create", [
      nativeToScVal(input.owner, { type: "address" }),
      nativeToScVal(input.title),
      nativeToScVal(input.description),
      nativeToScVal(input.budget, { type: "i128" }),
      nativeToScVal(input.deadline, { type: "u64" }),
      nativeToScVal(input.milestoneCount, { type: "u32" }),
    ], options) as Promise<rpc.Api.SendTransactionResponse>;
  }

  /**
   * Funds an existing grant with tokens.
   *
   * @param input Funding details including grant ID, token address, and amount.
   * @param options Optional transaction configuration.
   * @returns A promise that resolves to the transaction submission result.
   */
  async grantFund(input: GrantFundInput, options?: WriteOptions): Promise<rpc.Api.SendTransactionResponse> {
    return this.invokeWrite("grant_fund", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.token, { type: "address" }),
      nativeToScVal(input.amount, { type: "i128" }),
    ], options) as Promise<rpc.Api.SendTransactionResponse>;
  }

  /**
   * Submits a proof hash for a specific milestone.
   *
   * @param input Milestone details and the proof hash.
   * @param options Optional transaction configuration.
   * @returns A promise that resolves to the transaction submission result.
   */
  async milestoneSubmit(input: MilestoneSubmitInput, options?: WriteOptions): Promise<rpc.Api.SendTransactionResponse> {
    return this.invokeWrite("milestone_submit", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.proofHash),
    ], options) as Promise<rpc.Api.SendTransactionResponse>;
  }

  /**
   * Casts a vote (approval or rejection) for a milestone.
   *
   * @param input Vote details including grant ID, milestone index, and approval flag.
   * @param options Optional transaction configuration.
   * @returns A promise that resolves to the transaction submission result.
   */
  async milestoneVote(input: MilestoneVoteInput, options?: WriteOptions): Promise<rpc.Api.SendTransactionResponse> {
    return this.invokeWrite("milestone_vote", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.approve),
    ], options) as Promise<rpc.Api.SendTransactionResponse>;
  }

  /**
   * Retrieves the details of a grant from the contract (read-only).
   *
   * @param grantId The unique numeric ID of the grant.
   * @returns A promise that resolves to the grant data.
   */
  async grantGet(grantId: number): Promise<unknown> {
    return this.invokeRead("grant_get", [nativeToScVal(grantId, { type: "u32" })]);
  }

  /**
   * Retrieves milestone details for a specific grant (read-only).
   *
   * @param grantId The unique numeric ID of the grant.
   * @param milestoneIdx The 0-based index of the milestone.
   * @returns A promise that resolves to the milestone data.
   */
  async milestoneGet(grantId: number, milestoneIdx: number): Promise<unknown> {
    return this.invokeRead("milestone_get", [
      nativeToScVal(grantId, { type: "u32" }),
      nativeToScVal(milestoneIdx, { type: "u32" }),
    ]);
  }

  /**
   * Estimates transaction fees for a contract method at all priority tiers
   * without submitting a transaction.
   *
   * Use this to give users a cost preview before they sign.
   *
   * @param method The contract method name.
   * @param args The ScVal arguments for the method.
   * @returns Fee estimates at low / medium / high priority tiers.
   *
   * @example
   * ```typescript
   * const fees = await sdk.estimateFees("grant_create", args);
   * console.log(`Medium fee: ${fees.medium} stroops`);
   * ```
   */
  async estimateFees(method: string, args: xdr.ScVal[]): Promise<FeeEstimate> {
    const tx = await this.buildTx(method, args, { skipAccountLookup: true });
    const simulation = await this.server.simulateTransaction(tx) as any;
    this.ensureSimulationSuccess(simulation);

    const simulationBase = Number(simulation.minResourceFee ?? 0);
    const dynamicFees = await this.resolveDynamicFeeModifiers();
    const recommendedBase = dynamicFees?.recommendedBaseFee ?? simulationBase;
    const effectiveBase = Math.max(simulationBase, recommendedBase);
    const modifiers = dynamicFees?.modifiers ?? DEFAULT_FEE_LEVEL_MODIFIERS;

    const calc = (multiplier: number) =>
      String(Math.ceil(effectiveBase * multiplier));

    return {
      base: String(simulationBase),
      ...(dynamicFees?.recommendedBaseFee !== undefined
        ? { recommendedBase: String(dynamicFees.recommendedBaseFee) }
        : {}),
      low: calc(modifiers.low),
      medium: calc(modifiers.medium),
      high: calc(modifiers.high),
      modifiers,
      networkLoad: dynamicFees?.networkLoad ?? "moderate",
      source: dynamicFees ? "horizon" : "simulation-fallback",
    };
  }

  // ── Token Allowance Management (#272) ──────────────────────────────────────

  /**
   * Reads the current allowance granted by `owner` to the StellarGrants
   * contract for a given SAC token.
   *
   * @param tokenAddress The Stellar Asset Contract (SAC) address.
   * @param owner The account whose allowance is being checked.
   */
  async getAllowance(tokenAddress: string, owner: string): Promise<AllowanceResult> {
    const tokenContract = new Contract(tokenAddress);
    const spender = this.config.contractId;

    const args = [
      nativeToScVal(owner, { type: "address" }),
      nativeToScVal(spender, { type: "address" }),
    ];

    const tx = new TransactionBuilder(
      await this.getSourceAccount(true),
      { fee: this.config.defaultFee ?? "100", networkPassphrase: await this.resolveNetworkPassphrase() },
    )
      .addOperation(tokenContract.call("allowance", ...args))
      .setTimeout(30)
      .build();

    const simulation = await this.server.simulateTransaction(tx) as any;
    this.ensureSimulationSuccess(simulation);

    const raw = this.parseSimulationResult(simulation) as any;
    // SAC allowance returns a struct {amount: i128, expiration_ledger: u32}
    const amount: bigint = typeof raw?.amount === "bigint"
      ? raw.amount
      : BigInt(raw?.amount ?? 0);
    const expirationLedger: number = Number(raw?.expiration_ledger ?? 0);

    return { amount, expirationLedger };
  }

  /**
   * Approves the StellarGrants contract to spend `amount` tokens on behalf
   * of the authenticated signer.
   *
   * @param tokenAddress The Stellar Asset Contract (SAC) address.
   * @param amount The allowance amount (in base token units).
   * @param expirationLedger Ledger sequence at which the allowance expires.
   *   Defaults to current ledger + ~7 days (~100 000 ledgers).
   */
  async setAllowance(
    tokenAddress: string,
    amount: bigint,
    expirationLedger?: number,
  ): Promise<rpc.Api.SendTransactionResponse> {
    const signer = this.requireSigner();
    const owner = await signer.getPublicKey();
    const spender = this.config.contractId;

    // Default expiration: ~7 days at 6 s/ledger ≈ 100 800 ledgers
    let expLedger = expirationLedger;
    if (expLedger === undefined) {
      const latestLedger = await this.server.getLatestLedger?.() as any;
      expLedger = (latestLedger?.sequence ?? 0) + 100_800;
    }

    const tokenContract = new Contract(tokenAddress);
    const args = [
      nativeToScVal(owner, { type: "address" }),
      nativeToScVal(spender, { type: "address" }),
      nativeToScVal(amount, { type: "i128" }),
      nativeToScVal(expLedger, { type: "u32" }),
    ];

    const networkPassphrase = await this.resolveNetworkPassphrase();
    const tx = new TransactionBuilder(
      await this.getSourceAccount(),
      { fee: this.config.defaultFee ?? "100", networkPassphrase },
    )
      .addOperation(tokenContract.call("approve", ...args))
      .setTimeout(30)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    const signedXdr = await signer.signTransaction(prepared.toXDR(), networkPassphrase);
    const signedTx = TransactionBuilder.fromXDR(signedXdr, networkPassphrase);
    const sent = await this.server.sendTransaction(signedTx);
    if ((sent as any).status === "ERROR") {
      throw new StellarGrantsError(`setAllowance failed: ${(sent as any).errorResult ?? "unknown"}`);
    }
    return sent as rpc.Api.SendTransactionResponse;
  }

  /**
   * Checks whether the current allowance is sufficient for `required`. If not,
   * it automatically calls `setAllowance` to bring the allowance up to
   * `required` and prompts the user to sign once.
   *
   * @param tokenAddress The Stellar Asset Contract (SAC) address.
   * @param required The minimum required allowance (in base token units).
   * @param owner The account to check. Defaults to the signer's public key.
   * @returns A summary of the check including whether a new allowance was set.
   */
  async checkAndSetAllowance(
    tokenAddress: string,
    required: bigint,
    owner?: string,
  ): Promise<AllowanceCheckResult> {
    const signer = this.requireSigner();
    const resolvedOwner = owner ?? (await signer.getPublicKey());
    const { amount: current } = await this.getAllowance(tokenAddress, resolvedOwner);

    if (current >= required) {
      return { sufficient: true, current, required };
    }

    await this.setAllowance(tokenAddress, required);
    return { sufficient: false, current, required };
  }

  /**
   * Polls the RPC server for the status of a transaction until it reaches a
   * terminal state.
   *
   * @param hash The transaction hash to wait for.
   * @param intervalMs The polling interval in milliseconds.
   * @param timeoutMs The total timeout in milliseconds.
   * @returns The transaction response.
   */
  async waitForTransaction(
    hash: string,
    intervalMs: number = this.config.pollingIntervalMs ?? 1000,
    timeoutMs: number = this.config.pollingTimeoutMs ?? 30000,
  ): Promise<rpc.Api.GetTransactionResponse> {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      const response = await this.server.getTransaction(hash);
      if (response.status !== "NOT_FOUND") {
        if (response.status === "SUCCESS") {
          return response;
        }
        if (response.status === "FAILED") {
          throw new StellarGrantsError(`Transaction failed: ${hash}`, "TRANSACTION_FAILED", response);
        }
      }
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
    throw new StellarGrantsError(`Transaction timed out: ${hash}`, "TRANSACTION_TIMEOUT");
  }

  /**
   * Extracts and parses contract events from a successful transaction response.
   *
   * @param response The successful transaction response.
   * @returns An array of parsed events.
   */
  parseEvents(response: rpc.Api.GetTransactionResponse): ParsedEvent[] {
    return EventParser.parseEvents(response);
  }

  /**
   * Simulates a transaction without submitting it.
   *
   * @param method The contract method to call.
   * @param args The arguments for the method.
   * @returns The simulation response.
   */
  public async simulateTransaction(method: string, args: xdr.ScVal[]): Promise<rpc.Api.SimulateTransactionResponse> {
    const tx = await this.buildTx(method, args, { skipAccountLookup: true });
    const simulation = await this.server.simulateTransaction(tx);
    this.ensureSimulationSuccess(simulation);
    return simulation;
  }

  /**
   * Checks whether the SDK is compatible with the deployed contract.
   *
   * Queries the `sdk_version` read-only method on the contract. If the method
   * is not present the check is skipped and the result is marked as unknown.
   * A warning is emitted (via `console.warn`) when a mismatch is detected.
   *
   * @returns A compatibility report.
   *
   * @example
   * ```typescript
   * const report = await sdk.checkCompatibility();
   * if (!report.compatible) {
   *   console.warn(report.warning);
   * }
   * ```
   */
  async checkCompatibility(): Promise<{
    compatible: boolean;
    sdkVersion: number;
    contractVersion: number | null;
    warning?: string;
  }> {
    let contractVersion: number | null = null;

    try {
      const raw = await this.invokeRead("sdk_version", []);
      contractVersion = typeof raw === "number" ? raw : Number(raw);
    } catch {
      // Contract does not expose sdk_version — compatibility is unknown.
      return {
        compatible: true,
        sdkVersion: CONTRACT_INTERFACE_VERSION,
        contractVersion: null,
        warning:
          "Could not determine contract interface version. " +
          "The contract may not expose an `sdk_version` method.",
      };
    }

    const compatible = contractVersion === CONTRACT_INTERFACE_VERSION;

    if (!compatible) {
      const msg =
        `SDK interface version (${CONTRACT_INTERFACE_VERSION}) does not match ` +
        `contract version (${contractVersion}). ` +
        (contractVersion > CONTRACT_INTERFACE_VERSION
          ? "Please upgrade the SDK to the latest version."
          : "The deployed contract may be outdated.");
      console.warn(`[StellarGrantsSDK] ${msg}`);
      return { compatible, sdkVersion: CONTRACT_INTERFACE_VERSION, contractVersion, warning: msg };
    }

    return { compatible: true, sdkVersion: CONTRACT_INTERFACE_VERSION, contractVersion };
  }

  /**
   * Subscribes to contract events.
   *
   * @param callback Function called for each new event.
   * @param options Filter options for events.
   * @returns A function to unsubscribe.
   */
  public subscribeToEvents(
    callback: (event: any) => void,
    options?: { eventName?: string; startLedger?: number },
  ): () => void {
    let active = true;
    let currentCursor: string | undefined = undefined;

    const poll = async () => {
      if (!active) return;
      try {
        const req: any = {
          filters: [{ type: "contract", contractIds: [this.config.contractId] }],
        };
        if (!currentCursor && options?.startLedger) {
          req.startLedger = options.startLedger;
        }
        if (currentCursor) {
          req.pagination = { cursor: currentCursor };
        }

        const response = await this.server.getEvents(req);
        if (response.events) {
          for (const ev of response.events) {
            currentCursor = ev.id || ev.pagingToken || currentCursor;

            if (options?.eventName) {
              const topicMatches = ev.topic && ev.topic.some((t: any) => {
                try {
                  const scVal = typeof t === "string" ? xdr.ScVal.fromXDR(t, "base64") : t;
                  const parsed = scValToNative(scVal);
                  return parsed === options.eventName || String(parsed) === options.eventName;
                } catch { return false; }
              });
              if (!topicMatches) continue;
            }
            callback(ev);
          }
        }
      } catch (err) {
        console.warn("Event poll error, continuing...", err);
      }
      if (active) setTimeout(poll, 5000);
    };

    poll();
    return () => { active = false; };
  }

  // ---------------------------------------------------------------------------
  // Private helpers
  // ---------------------------------------------------------------------------

  /**
   * Internal helper for read-only contract invocations.
   */
  private async invokeRead(method: string, args: xdr.ScVal[]): Promise<unknown> {
    try {
      const tx = await this.buildTx(method, args, { skipAccountLookup: true });
      const simulation = await this.server.simulateTransaction(tx);
      this.ensureSimulationSuccess(simulation);
      return this.parseSimulationResult(simulation);
    } catch (error) {
      throw parseSorobanError(error);
    }
  }

  /**
   * Internal helper for state-changing contract invocations.
   *
   * Fee resolution order (highest precedence first):
   *  1. `options.simulatedFee`   – explicit override (wins unless feeMultiplier is also set).
   *  2. `options.feeMultiplier`  – multiply simulated resource fee.
   *  3. `options.feePriority`    – use a predefined tier multiplier ("low" | "medium" | "high").
   *  4. Default                  – medium priority (1.5× simulated resource fee).
   *
   * Simulation is skipped only when `transactionData` is supplied without `feeMultiplier`.
   */
  private async invokeWrite(
    method: string,
    args: xdr.ScVal[],
    options?: WriteOptions,
  ): Promise<rpc.Api.SendTransactionResponse | unknown> {
    const signer = this.requireSigner();
    try {
      let finalFee = this.config.defaultFee ?? "100";

      // Simulate when there is no pre-built transaction data OR when the caller
      // wants to override the fee with a multiplier (mirrors original behaviour).
      if (!options?.transactionData || options?.feeMultiplier) {
        const txForSim = await this.buildTx(method, args);
        const simulation = await this.server.simulateTransaction(txForSim) as any;
        this.ensureSimulationSuccess(simulation);

        const base = Number(simulation.minResourceFee ?? 0);
        const dynamicFees = await this.resolveDynamicFeeModifiers();
        const recommendedBase = dynamicFees?.recommendedBaseFee ?? base;
        const effectiveBase = Math.max(base, recommendedBase);

        if (options?.feeMultiplier) {
          finalFee = String(Math.ceil(effectiveBase * options.feeMultiplier));
        } else {
          const priority: FeePriority = options?.feePriority ?? "medium";
          const modifiers = dynamicFees?.modifiers ?? DEFAULT_FEE_LEVEL_MODIFIERS;
          finalFee = String(Math.ceil(effectiveBase * modifiers[priority]));
        }
      }

      // simulatedFee is the highest-priority override unless feeMultiplier is present.
      if (options?.simulatedFee && !options?.feeMultiplier) {
        finalFee = options.simulatedFee;
      }

      const tx = await this.buildTx(method, args, {
        overrideFee: finalFee,
        sorobanData: options?.transactionData,
      });
      let prepared = tx;

      if (!options?.transactionData) {
        prepared = await this.server.prepareTransaction(tx);
      }

      const networkPassphrase = await this.resolveNetworkPassphrase();
      const signedXdr = await signer.signTransaction(
        prepared.toXDR(),
        networkPassphrase,
      );
      const signedTx = TransactionBuilder.fromXDR(signedXdr, networkPassphrase);

      const sent = await this.server.sendTransaction(signedTx);
      if (sent.status === "ERROR") {
        throw new StellarGrantsError(`Send failed: ${sent.errorResult ?? "unknown error"}`);
      }
      return sent;
    } catch (error) {
      throw parseSorobanError(error);
    }
  }

  /**
   * Builds a transaction for a contract call.
   */
  private async buildTx(
    method: string,
    args: xdr.ScVal[],
    options?: {
      overrideFee?: string;
      sorobanData?: string | xdr.SorobanTransactionData;
      skipAccountLookup?: boolean;
    },
  ) {
    const account = await this.getSourceAccount(options?.skipAccountLookup);
    const networkPassphrase = await this.resolveNetworkPassphrase();
    const builder = new TransactionBuilder(account, {
      fee: options?.overrideFee ?? this.config.defaultFee ?? "100",
      networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(60);

    if (options?.sorobanData) {
      builder.setSorobanData(options.sorobanData);
    }

    return builder.build();
  }

  private async resolveNetworkPassphrase(): Promise<string> {
    if (this.config.networkPassphrase) {
      return this.config.networkPassphrase;
    }

    if (!this.networkPassphrasePromise) {
      this.networkPassphrasePromise = this.server.getNetwork().then((network: any) => network.passphrase);
    }

    return this.networkPassphrasePromise;
  }

  private requireSigner(): WalletAdapter {
    if (!this.config.signer) {
      throw new StellarGrantsError(
        "A signer is required for write operations. Initialize StellarGrantsSDK with a signer to submit transactions.",
        "SIGNER_REQUIRED",
      );
    }
    return this.config.signer;
  }

  private async getSourceAccount(skipAccountLookup = false): Promise<Account> {
    if (skipAccountLookup) {
      const source = this.config.signer
        ? await this.config.signer.getPublicKey()
        : READ_ONLY_SIMULATION_ACCOUNT;
      return new Account(source, "0");
    }

    const source = await this.requireSigner().getPublicKey();
    return this.server.getAccount(source);
  }

  /**
   * Validates that the simulation was successful.
   */
  private ensureSimulationSuccess(simulation: any) {
    if (simulation?.error) {
      throw new StellarGrantsError(String(simulation.error));
    }
  }

  /**
   * Parses the return value from a simulation result.
   */
  private parseSimulationResult(simulation: any): unknown {
    const retval = simulation?.result?.retval;
    if (!retval) return null;
    return scValToNative(retval);
  }

  private async resolveDynamicFeeModifiers(): Promise<{
    modifiers: FeeLevelModifiers;
    networkLoad: FeeNetworkLoad;
    recommendedBaseFee?: number;
  } | null> {
    const endpoint = this.config.feeStatsEndpoint
      ?? (this.config.horizonUrl
        ? `${this.config.horizonUrl.replace(/\/$/, "")}/fee_stats`
        : undefined);

    if (!endpoint) {
      return null;
    }

    try {
      const stats = await this.fetchJsonWithTimeout(endpoint, 5000);
      const usage = Number(stats?.ledger_capacity_usage);
      if (!Number.isFinite(usage)) {
        return null;
      }

      const feeBuckets = stats?.max_fee ?? {};
      const recommendedBaseFee = Number(
        feeBuckets.p70 ?? feeBuckets.p60 ?? feeBuckets.p50 ?? 0,
      );

      const level = DYNAMIC_FEE_LEVELS.find((entry) => usage <= entry.maxUsage);
      if (!level) return null;

      return {
        modifiers: level.modifiers,
        networkLoad: level.load,
        ...(Number.isFinite(recommendedBaseFee) && recommendedBaseFee > 0
          ? { recommendedBaseFee }
          : {}),
      };
    } catch (error) {
      console.warn("Failed to fetch dynamic fee stats; falling back to static multipliers", error);
      return null;
    }
  }

  private async fetchJsonWithTimeout(url: string, timeoutMs: number): Promise<any> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetch(url, { signal: controller.signal });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return response.json();
    } finally {
      clearTimeout(timer);
    }
  }
}
