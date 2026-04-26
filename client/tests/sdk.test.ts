import { StellarGrantsSDK } from "../src/StellarGrantsSDK";
import { parseSorobanError } from "../src/errors/parseSorobanError";
import {
  UnauthorizedError,
  GrantNotFoundError,
  DeadlinePassedError,
  InsufficientBalanceError,
  CONTRACT_ERROR_CODES,
  getErrorName,
  getErrorClass,
  SorobanRevertError,
} from "../src/errors/StellarGrantsError";

jest.mock("@stellar/stellar-sdk", () => {
  class MockServer {
    static simulationError: string | null = null;
    constructor() {}
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
  }

  return {
    rpc: { Server: MockServer },
    Contract: class {
      constructor() {}
      call(method: string, ...args: unknown[]) {
        return { method, args };
      }
    },
    TransactionBuilder: class {
      static fromXDR() {
        return { from: "xdr" };
      }
      constructor() {}
      addOperation() {
        return this;
      }
      setTimeout() {
        return this;
      }
      build() {
        return { toXDR: () => "TX_XDR" };
      }
    },
    nativeToScVal: (value: unknown) => ({ value }),
    scValToNative: () => ({ ok: true }),
    xdr: {},
  };
});

describe("StellarGrantsSDK", () => {
  const signer = {
    getPublicKey: jest.fn(async () => "GABC123"),
    signTransaction: jest.fn(async () => "SIGNED_XDR"),
  };

  it("calls write wrappers without stellar-cli dependency", async () => {
    const sdk = new StellarGrantsSDK({
      contractId: "CBLAH",
      rpcUrl: "https://rpc.test",
      networkPassphrase: "Test SDF Network ; September 2015",
      signer,
    });

    const result = await sdk.grantFund({
      grantId: 1,
      token: "GCTOKEN",
      amount: 1000n,
    });

    expect(result).toEqual({ status: "PENDING", hash: "abc123" });
    expect(signer.signTransaction).toHaveBeenCalled();
  });

  it("provides read wrapper response parsing", async () => {
    const sdk = new StellarGrantsSDK({
      contractId: "CBLAH",
      rpcUrl: "https://rpc.test",
      networkPassphrase: "Test SDF Network ; September 2015",
      signer,
    });

    const grant = await sdk.grantGet(7);
    expect(grant).toEqual({ ok: true });
  });

  it("parses generic Soroban revert errors", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: grant not active"));
    expect(parsed.name).toBe("SorobanRevertError");
    expect(parsed.message).toContain("grant not active");
  });

  it("parses error code 2 (Unauthorized) from error message", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: Error(Code2)"));
    expect(parsed).toBeInstanceOf(UnauthorizedError);
    expect(parsed.name).toBe("UnauthorizedError");
    expect(parsed.message).toContain("Unauthorized");
    expect((parsed as any).details).toHaveProperty("code", 2);
  });

  it("parses error code 1 (GrantNotFound) from error message", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: Error(Code1)"));
    expect(parsed).toBeInstanceOf(GrantNotFoundError);
    expect(parsed.name).toBe("GrantNotFoundError");
    expect(parsed.message).toContain("Grant not found");
  });

  it("parses error code 5 (DeadlinePassed) from error message", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: error code: 5"));
    expect(parsed).toBeInstanceOf(DeadlinePassedError);
    expect(parsed.name).toBe("DeadlinePassedError");
    expect(parsed.message).toContain("Deadline has passed");
  });

  it("parses error code 32 (InsufficientBalance) from error message", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: code 32"));
    expect(parsed).toBeInstanceOf(InsufficientBalanceError);
    expect(parsed.name).toBe("InsufficientBalanceError");
    expect(parsed.message).toContain("Insufficient token balance");
  });

  it("falls back to generic SorobanRevertError for unknown error codes", () => {
    const parsed = parseSorobanError(new Error("HostError: txFailed: revert: Error(Code999)"));
    expect(parsed).toBeInstanceOf(SorobanRevertError);
    expect(parsed.name).toBe("SorobanRevertError");
  });

  it("exports CONTRACT_ERROR_CODES with correct values", () => {
    expect(CONTRACT_ERROR_CODES.GrantNotFound).toBe(1);
    expect(CONTRACT_ERROR_CODES.Unauthorized).toBe(2);
    expect(CONTRACT_ERROR_CODES.DeadlinePassed).toBe(5);
    expect(CONTRACT_ERROR_CODES.InsufficientBalance).toBe(32);
    expect(CONTRACT_ERROR_CODES.ContractPaused).toBe(33);
  });

  it("getErrorName returns correct name for error code", () => {
    expect(getErrorName(1)).toBe("GrantNotFound");
    expect(getErrorName(2)).toBe("Unauthorized");
    expect(getErrorName(5)).toBe("DeadlinePassed");
    expect(getErrorName(32)).toBe("InsufficientBalance");
    expect(getErrorName(999)).toBeUndefined();
  });

  it("getErrorClass returns correct class for error code", () => {
    const ErrorClass = getErrorClass(2);
    expect(ErrorClass).toBe(UnauthorizedError);
    expect(new ErrorClass()).toBeInstanceOf(SorobanRevertError);
  });
});
