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
} from "./types";

/**
 * Encapsulated client for StellarGrants Soroban contract interactions.
 * Uses RPC simulation/sending only and does not shell out to stellar-cli.
 */
export class StellarGrantsSDK {
  private readonly contract: Contract;
  private readonly server: any;
  private readonly config: StellarGrantsSDKConfig;

  constructor(config: StellarGrantsSDKConfig) {
    this.config = config;
    this.contract = new Contract(config.contractId);
    this.server = new rpc.Server(config.rpcUrl, {
      allowHttp: config.rpcUrl.startsWith("http://"),
    });
  }

  /**
   * Creates a new grant.
   */
  async grantCreate(input: GrantCreateInput, options?: any): Promise<unknown> {
    return this.invokeWrite("grant_create", [
      nativeToScVal(input.owner, { type: "address" }),
      nativeToScVal(input.title),
      nativeToScVal(input.description),
      nativeToScVal(input.budget, { type: "i128" }),
      nativeToScVal(input.deadline, { type: "u64" }),
      nativeToScVal(input.milestoneCount, { type: "u32" }),
    ], options);
  }

  /**
   * Funds an existing grant.
   */
  async grantFund(input: GrantFundInput, options?: any): Promise<unknown> {
    return this.invokeWrite("grant_fund", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.token, { type: "address" }),
      nativeToScVal(input.amount, { type: "i128" }),
    ], options);
  }

  /**
   * Submits milestone proof for a grant.
   */
  async milestoneSubmit(input: MilestoneSubmitInput, options?: any): Promise<unknown> {
    return this.invokeWrite("milestone_submit", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.proofHash),
    ], options);
  }

  /**
   * Casts an approval/rejection vote for a milestone.
   */
  async milestoneVote(input: MilestoneVoteInput, options?: any): Promise<unknown> {
    return this.invokeWrite("milestone_vote", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.approve),
    ], options);
  }

  /**
   * Reads a grant by id.
   */
  async grantGet(grantId: number): Promise<unknown> {
    return this.invokeRead("grant_get", [nativeToScVal(grantId, { type: "u32" })]);
  }

  /**
   * Reads milestone details by grant and milestone index.
   */
  async milestoneGet(grantId: number, milestoneIdx: number): Promise<unknown> {
    return this.invokeRead("milestone_get", [
      nativeToScVal(grantId, { type: "u32" }),
      nativeToScVal(milestoneIdx, { type: "u32" }),
    ]);
  }

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

  public async simulateTransaction(method: string, args: xdr.ScVal[]): Promise<any> {
    const tx = await this.buildTx(method, args);
    const simulation = await this.server.simulateTransaction(tx);
    this.ensureSimulationSuccess(simulation);
    return simulation;
  }

  private async invokeWrite(
    method: string, 
    args: xdr.ScVal[],
    options?: {
      feeMultiplier?: number;
      transactionData?: string | xdr.SorobanTransactionData;
      simulatedFee?: string;
    }
  ): Promise<unknown> {
    try {
      let finalFee = this.config.defaultFee ?? "100";
      let manualSim = false;

      if (!options?.transactionData || options?.feeMultiplier) {
        const txForSim = await this.buildTx(method, args);
        const simulation = await this.server.simulateTransaction(txForSim);
        this.ensureSimulationSuccess(simulation);
        
        if (options?.feeMultiplier) {
          finalFee = String(Math.ceil(Number(simulation.minResourceFee) * options.feeMultiplier));
        } else {
          finalFee = String(Number(simulation.minResourceFee || 0) + 10000);
        }
        manualSim = true;
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

  private async buildTx(method: string, args: xdr.ScVal[], overrideFee?: string, sorobanData?: string | xdr.SorobanTransactionData) {
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

  private ensureSimulationSuccess(simulation: any) {
    if (simulation?.error) {
      throw new StellarGrantsError(String(simulation.error));
    }
  }

  private parseSimulationResult(simulation: any): unknown {
    const retval = simulation?.result?.retval;
    if (!retval) return null;
    return scValToNative(retval);
  }

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
              const topicMatches = ev.topic && ev.topic.some((t: string) => {
                 try { 
                    const scVal = xdr.ScVal.fromXDR(t, "base64");
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
}
