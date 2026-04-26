import {
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
  GrantCreateInput,
  GrantFundInput,
  MilestoneSubmitInput,
  MilestoneVoteInput,
  StellarGrantsSDKConfig,
  WalletAdapter,
  WriteOptions,
} from "./types";
import { EventParser, ParsedEvent } from "./events";

/**
 * Encapsulated client for StellarGrants Soroban contract interactions.
 * 
 * This SDK provides a high-level interface to interact with the StellarGrants smart contract.
 * It handles transaction building, simulation, signing (via a provided signer), and submission.
 * 
 * @example
 * ```typescript
 * const sdk = new StellarGrantsSDK({
 *   contractId: "CD...",
 *   rpcUrl: "https://soroban-testnet.stellar.org",
 *   networkPassphrase: "Test SDF Network ; September 2015",
 *   signer: freighterSigner
 * });
 * ```
 */
export class StellarGrantsSDK {
  private readonly contract: Contract;
  private readonly server: rpc.Server;
  private readonly config: StellarGrantsSDKConfig;

  /**
   * Initializes a new instance of the StellarGrantsSDK.
   * @param config Configuration options including contract ID, RPC URL, and signer.
   */
  constructor(config: StellarGrantsSDKConfig) {
    this.config = config;
    this.contract = new Contract(config.contractId);
    this.server = new rpc.Server(config.rpcUrl, {
      allowHttp: config.rpcUrl.startsWith("http://"),
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
   * Polls the RPC server for the status of a transaction until it reaches a terminal state.
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
    const tx = await this.buildTx(method, args);
    const simulation = await this.server.simulateTransaction(tx);
    this.ensureSimulationSuccess(simulation);
    return simulation;
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

  /**
   * Internal helper for read-only contract invocations.
   */
  private async invokeRead(method: string, args: xdr.ScVal[]): Promise<unknown> {
    try {
      const tx = await this.buildTx(method, args);
      const simulation = await this.server.simulateTransaction(tx);
      this.ensureSimulationSuccess(simulation);
      return this.parseSimulationResult(simulation);
    } catch (error) {
      throw parseSorobanError(error);
    }
  }

  /**
   * Internal helper for state-changing contract invocations.
   */
  private async invokeWrite(
    method: string, 
    args: xdr.ScVal[],
    options?: WriteOptions
  ): Promise<rpc.Api.SendTransactionResponse | unknown> {
    try {
      let finalFee = this.config.defaultFee ?? "100";

      if (!options?.transactionData || options?.feeMultiplier) {
        const txForSim = await this.buildTx(method, args);
        const simulation = await this.server.simulateTransaction(txForSim) as any;
        this.ensureSimulationSuccess(simulation);
        
        if (options?.feeMultiplier) {
          finalFee = String(Math.ceil(Number(simulation.minResourceFee) * options.feeMultiplier));
        } else {
          finalFee = String(Number(simulation.minResourceFee || 0) + 10000);
        }
      }

      if (options?.simulatedFee && !options?.feeMultiplier) {
        finalFee = options.simulatedFee;
      }

      const tx = await this.buildTx(method, args, finalFee, options?.transactionData);
      let prepared = tx;

      if (!options?.transactionData) {
        prepared = await this.server.prepareTransaction(tx);
      }

      const signedXdr = await this.config.signer.signTransaction(
        prepared.toXDR(),
        this.config.networkPassphrase,
      );
      const signedTx = TransactionBuilder.fromXDR(signedXdr, this.config.networkPassphrase);

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
    overrideFee?: string, 
    sorobanData?: string | xdr.SorobanTransactionData
  ) {
    const source = await this.config.signer.getPublicKey();
    const account = await this.server.getAccount(source);
    const builder = new TransactionBuilder(account, {
      fee: overrideFee ?? this.config.defaultFee ?? "100",
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(60);
      
    if (sorobanData) {
      builder.setSorobanData(sorobanData);
    }
      
    return builder.build();
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
}
