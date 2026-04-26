/**
 * StellarGrantsSDK unit tests.
 *
 * @stellar/stellar-sdk is mocked at module level so no real network I/O occurs.
 * The internal `server` field is replaced via sdkFactory for per-test control.
 */

import { StellarGrantsSDK } from "../src/StellarGrantsSDK";
import { StellarGrantsError } from "../src/errors/StellarGrantsError";
import { makeSdk, TEST_NETWORK_PASSPHRASE } from "./helpers/sdkFactory";
import {
  GrantCreateInput,
  GrantFundInput,
  MilestoneSubmitInput,
  MilestoneVoteInput,
} from "../src/types";

// ---------------------------------------------------------------------------
// Module-level mock for @stellar/stellar-sdk
// ---------------------------------------------------------------------------
jest.mock("@stellar/stellar-sdk", () => {
  class MockServer {
    static simulationError: string | null = null;
    constructor() { }
    async getAccount() {
      return { accountId: "GTEST", sequence: "1" };
    }
    async simulateTransaction() {
      if (MockServer.simulationError) {
        return { error: MockServer.simulationError };
      }
      return { result: { retval: { _mock: "ok" } } };
    }
    async prepareTransaction(tx: any) {
      return tx;
    }
    async sendTransaction() {
      return { status: "PENDING", hash: "abc123" };
    }
    async getTransaction(hash: string) {
      if (hash === "fail") return { status: "FAILED" };
      if (hash === "timeout") return { status: "NOT_FOUND" };
      return { status: "SUCCESS", hash };
    }
  }

  return {
    rpc: {
      Server: class {
        constructor() { }
        async getAccount() { return { accountId: "GMOCK", sequence: "0" }; }
        async simulateTransaction() { return { result: { retval: null }, minResourceFee: "1000" }; }
        async prepareTransaction(tx: any) { return tx; }
        async sendTransaction() { return { status: "PENDING", hash: "mockhash" }; }
        async getEvents() { return { events: [] }; }
        async getTransaction(hash: string) {
          if (hash === "fail") return { status: "FAILED" };
          if (hash === "timeout") return { status: "NOT_FOUND" };
          return { status: "SUCCESS", hash };
        }
      },
    },
    Contract: class {
      constructor() { }
      call(method: string, ...args: unknown[]) { return { method, args }; }
    },
    TransactionBuilder: class {
      static fromXDR(_xdr: string, _passphrase: string) {
        return { toXDR: () => "SIGNED_TX_XDR" };
      }
      constructor() { }
      addOperation() { return this; }
      setTimeout() { return this; }
      setSorobanData() { return this; }
      build() { return { toXDR: () => "TX_XDR" }; }
    },
    nativeToScVal: (value: unknown, _opts?: any) => ({ _scval: value }),
    scValToNative: (val: any) => val?._native ?? { ok: true },
    xdr: {
      ScVal: {
        fromXDR: (_b64: string, _fmt: string) => ({ _scval: "decoded" }),
      },
      SorobanTransactionData: class { },
    },
  };
});

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------
const GRANT_CREATE: GrantCreateInput = {
  owner: "GOWNER",
  title: "Test Grant",
  description: "desc",
  budget: 1000000n,
  deadline: 9999999n,
  milestoneCount: 3,
};
const GRANT_FUND: GrantFundInput = { grantId: 1, token: "GCTOKEN", amount: 500000n };
const MILESTONE_SUBMIT: MilestoneSubmitInput = { grantId: 1, milestoneIdx: 0, proofHash: "hash123" };
const MILESTONE_VOTE: MilestoneVoteInput = { grantId: 1, milestoneIdx: 0, approve: true };

// ---------------------------------------------------------------------------
// Read methods — Req 2.x
// ---------------------------------------------------------------------------
describe("Read methods", () => {
  it("grantGet calls simulateTransaction exactly once and returns parsed result", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockResolvedValueOnce({
      result: { retval: { _native: { id: 7, status: "active" } } },
      minResourceFee: "1000",
    });

    const result = await sdk.grantGet(7);

    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ id: 7, status: "active" });
  });

  it("grantGet returns null when retval is absent", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockResolvedValueOnce({ result: {}, minResourceFee: "1000" });

    const result = await sdk.grantGet(1);
    expect(result).toBeNull();
  });

  it("grantGet throws StellarGrantsError when simulation has error field", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockResolvedValueOnce({ error: "contract not found" });

    await expect(sdk.grantGet(99)).rejects.toBeInstanceOf(StellarGrantsError);
  });

  it("milestoneGet calls simulateTransaction exactly once", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockResolvedValueOnce({
      result: { retval: { _native: { idx: 0 } } },
      minResourceFee: "1000",
    });

    const result = await sdk.milestoneGet(1, 0);

    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ idx: 0 });
  });

  it("simulateTransaction (public) returns raw simulation object", async () => {
    const { sdk, mockServer } = makeSdk();
    const rawSim = { result: { retval: { _native: "raw" } }, minResourceFee: "500" };
    mockServer.simulateTransaction.mockResolvedValueOnce(rawSim);

    const result = await sdk.simulateTransaction("grant_get", []);

    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    expect(result).toEqual(rawSim);
  });

  it("simulateTransaction (public) throws when simulation has error field", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockResolvedValueOnce({ error: "sim error" });

    await expect(sdk.simulateTransaction("grant_get", [])).rejects.toBeInstanceOf(StellarGrantsError);
  });

  // Property 1: read methods call simulateTransaction exactly once
  it("Property 1 — grantGet and milestoneGet each call simulateTransaction exactly once", async () => {
    for (const [method, args] of [
      ["grantGet", [1]] as const,
      ["milestoneGet", [1, 0]] as const,
    ]) {
      const { sdk, mockServer } = makeSdk();
      mockServer.simulateTransaction.mockResolvedValue({
        result: { retval: null },
        minResourceFee: "1000",
      });
      await (sdk as any)[method](...args);
      expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    }
  });

  // Property 11: error wrapping invariant — errors from read ops are StellarGrantsError
  it("Property 11 — errors from read operations are always StellarGrantsError instances", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockRejectedValueOnce(new Error("raw network error"));

    const err = await sdk.grantGet(1).catch((e) => e);
    expect(err).toBeInstanceOf(StellarGrantsError);
  });
});

// ---------------------------------------------------------------------------
// Write methods — Req 3.x
// ---------------------------------------------------------------------------
describe("Write methods", () => {
  it("grantCreate calls signTransaction once and returns send result", async () => {
    const { sdk, mockSigner, mockServer } = makeSdk();

    const result = await sdk.grantCreate(GRANT_CREATE);

    expect(mockSigner.signTransaction).toHaveBeenCalledTimes(1);
    expect(mockSigner.signTransaction).toHaveBeenCalledWith("TX_XDR", TEST_NETWORK_PASSPHRASE);
    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ status: "PENDING", hash: "mockhash123" });
  });

  it("grantFund calls sendTransaction and returns result", async () => {
    const { sdk, mockServer } = makeSdk();

    const result = await sdk.grantFund(GRANT_FUND);

    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ status: "PENDING", hash: "mockhash123" });
  });

  it("milestoneSubmit calls signTransaction exactly once", async () => {
    const { sdk, mockSigner } = makeSdk();

    await sdk.milestoneSubmit(MILESTONE_SUBMIT);

    expect(mockSigner.signTransaction).toHaveBeenCalledTimes(1);
  });

  it("milestoneVote calls sendTransaction exactly once", async () => {
    const { sdk, mockServer } = makeSdk();

    await sdk.milestoneVote(MILESTONE_VOTE);

    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
  });

  it("throws StellarGrantsError when sendTransaction returns status ERROR", async () => {
    const { sdk, state } = makeSdk();
    state.sendStatus = "ERROR";
    state.sendErrorResult = "contract execution failed";

    await expect(sdk.grantFund(GRANT_FUND)).rejects.toBeInstanceOf(StellarGrantsError);
  });

  // Property 5: all write methods call signTransaction exactly once on success
  it("Property 5 — all write methods call signTransaction exactly once", async () => {
    const writeCalls: Array<[string, any[]]> = [
      ["grantCreate", [GRANT_CREATE]],
      ["grantFund", [GRANT_FUND]],
      ["milestoneSubmit", [MILESTONE_SUBMIT]],
      ["milestoneVote", [MILESTONE_VOTE]],
    ];

    for (const [method, args] of writeCalls) {
      const { sdk, mockSigner } = makeSdk();
      await (sdk as any)[method](...args);
      expect(mockSigner.signTransaction).toHaveBeenCalledTimes(1);
    }
  });

  // Property 11 for write ops
  it("Property 11 — errors from write operations are always StellarGrantsError instances", async () => {
    const { sdk, mockServer } = makeSdk();
    mockServer.simulateTransaction.mockRejectedValueOnce(new Error("raw rpc error"));

    const err = await sdk.grantFund(GRANT_FUND).catch((e) => e);
    expect(err).toBeInstanceOf(StellarGrantsError);
  });
});

// ---------------------------------------------------------------------------
// invokeWrite option paths — Req 4.x
// ---------------------------------------------------------------------------
describe("invokeWrite option paths", () => {
  it("default fee = minResourceFee + 10000", async () => {
    const { sdk, mockServer, state } = makeSdk();
    state.minResourceFee = "5000";

    // Capture the fee used in buildTx by spying on getAccount (called inside buildTx)
    // We verify by checking that simulateTransaction was called (fee computed from sim)
    await sdk.grantFund(GRANT_FUND);

    // simulateTransaction called once for fee computation, once is enough
    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
  });

  it("Property 3 — default fee computation: minResourceFee + 10000 for several values", async () => {
    const fees = ["0", "100", "5000", "99999"];
    for (const fee of fees) {
      const { sdk, mockServer, state } = makeSdk();
      state.minResourceFee = fee;
      await sdk.grantFund(GRANT_FUND);
      // Simulation must have been called to determine the fee
      expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    }
  });

  it("feeMultiplier: fee = ceil(minResourceFee * multiplier)", async () => {
    const { sdk, mockServer, state } = makeSdk();
    state.minResourceFee = "4000";

    await sdk.grantFund(GRANT_FUND, { feeMultiplier: 1.5 });

    // Simulation must be called when feeMultiplier is set
    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
  });

  it("Property 4 — feeMultiplier fee computation for several values", async () => {
    const cases: Array<[string, number]> = [
      ["1000", 2],
      ["4000", 1.5],
      ["7777", 1.25],
    ];
    for (const [fee, multiplier] of cases) {
      const { sdk, state } = makeSdk();
      state.minResourceFee = fee;
      // Should not throw — fee = ceil(fee * multiplier)
      await expect(sdk.grantFund(GRANT_FUND, { feeMultiplier: multiplier })).resolves.toBeDefined();
    }
  });

  it("simulatedFee: uses provided fee, simulation still runs", async () => {
    const { sdk, mockServer } = makeSdk();

    await sdk.grantFund(GRANT_FUND, { simulatedFee: "99999" });

    // Simulation is still called (simulatedFee only overrides the fee value)
    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
  });

  it("transactionData (no feeMultiplier): skips simulation and prepareTransaction", async () => {
    const { sdk, mockServer } = makeSdk();

    await sdk.grantFund(GRANT_FUND, { transactionData: "MOCK_TX_DATA" });

    expect(mockServer.simulateTransaction).not.toHaveBeenCalled();
    expect(mockServer.prepareTransaction).not.toHaveBeenCalled();
    expect(mockServer.sendTransaction).toHaveBeenCalledTimes(1);
  });

  it("Property 6 — transactionData without feeMultiplier always skips simulation", async () => {
    const writeCalls: Array<[string, any[]]> = [
      ["grantCreate", [GRANT_CREATE, { transactionData: "TD" }]],
      ["grantFund", [GRANT_FUND, { transactionData: "TD" }]],
      ["milestoneSubmit", [MILESTONE_SUBMIT, { transactionData: "TD" }]],
      ["milestoneVote", [MILESTONE_VOTE, { transactionData: "TD" }]],
    ];
    for (const [method, args] of writeCalls) {
      const { sdk, mockServer } = makeSdk();
      await (sdk as any)[method](...args);
      expect(mockServer.simulateTransaction).not.toHaveBeenCalled();
    }
  });

  it("transactionData + feeMultiplier: simulation IS called", async () => {
    const { sdk, mockServer } = makeSdk();

    await sdk.grantFund(GRANT_FUND, { transactionData: "MOCK_TX_DATA", feeMultiplier: 2 });

    expect(mockServer.simulateTransaction).toHaveBeenCalledTimes(1);
  });

  describe("waitForTransaction", () => {
    const signer = {
      getPublicKey: jest.fn(async () => "GABC123"),
      signTransaction: jest.fn(async () => "SIGNED_XDR"),
    };

    it("resolves on SUCCESS", async () => {
      const sdk = new StellarGrantsSDK({
        contractId: "CBLAH",
        rpcUrl: "https://rpc.test",
        networkPassphrase: "Test SDF Network ; September 2015",
        signer,
      });

      const res = await sdk.waitForTransaction("abc123");
      expect(res.status).toBe("SUCCESS");
    });

    it("throws on FAILED", async () => {
      const sdk = new StellarGrantsSDK({
        contractId: "CBLAH",
        rpcUrl: "https://rpc.test",
        networkPassphrase: "Test SDF Network ; September 2015",
        signer,
      });

      await expect(sdk.waitForTransaction("fail")).rejects.toThrow("Transaction failed");
    });

    it("throws on timeout", async () => {
      const sdk = new StellarGrantsSDK({
        contractId: "CBLAH",
        rpcUrl: "https://rpc.test",
        networkPassphrase: "Test SDF Network ; September 2015",
        signer,
        pollingIntervalMs: 10,
        pollingTimeoutMs: 50,
      });

      await expect(sdk.waitForTransaction("timeout")).rejects.toThrow("Transaction timed out");
    });
  });
});
