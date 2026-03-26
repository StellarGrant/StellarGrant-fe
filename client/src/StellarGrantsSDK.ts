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
  async grantCreate(input: GrantCreateInput): Promise<unknown> {
    return this.invokeWrite("grant_create", [
      nativeToScVal(input.owner, { type: "address" }),
      nativeToScVal(input.title),
      nativeToScVal(input.description),
      nativeToScVal(input.budget, { type: "i128" }),
      nativeToScVal(input.deadline, { type: "u64" }),
      nativeToScVal(input.milestoneCount, { type: "u32" }),
    ]);
  }

  /**
   * Funds an existing grant.
   */
  async grantFund(input: GrantFundInput): Promise<unknown> {
    return this.invokeWrite("grant_fund", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.token, { type: "address" }),
      nativeToScVal(input.amount, { type: "i128" }),
    ]);
  }

  /**
   * Submits milestone proof for a grant.
   */
  async milestoneSubmit(input: MilestoneSubmitInput): Promise<unknown> {
    return this.invokeWrite("milestone_submit", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.proofHash),
    ]);
  }

  /**
   * Casts an approval/rejection vote for a milestone.
   */
  async milestoneVote(input: MilestoneVoteInput): Promise<unknown> {
    return this.invokeWrite("milestone_vote", [
      nativeToScVal(input.grantId, { type: "u32" }),
      nativeToScVal(input.milestoneIdx, { type: "u32" }),
      nativeToScVal(input.approve),
    ]);
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

  private async invokeWrite(method: string, args: xdr.ScVal[]): Promise<unknown> {
    try {
      const tx = await this.buildTx(method, args);
      const simulation = await this.server.simulateTransaction(tx);
      this.ensureSimulationSuccess(simulation);

      const prepared = await this.server.prepareTransaction(tx);
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

  private async buildTx(method: string, args: xdr.ScVal[]) {
    const source = await this.config.signer.getPublicKey();
    const account = await this.server.getAccount(source);
    return new TransactionBuilder(account, {
      fee: this.config.defaultFee ?? "100",
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(60)
      .build();
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
}
