import { makeSdk } from "./helpers/sdkFactory";

jest.mock("@stellar/stellar-sdk", () => {
  const actual = jest.requireActual("@stellar/stellar-sdk");
  return {
    ...actual,
    Contract: jest.fn().mockImplementation(() => ({
      call: jest.fn(),
    })),
  };
});

describe("WebSocket event subscription", () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("uses polling when RPC URL is http (not WebSocket)", async () => {
    const { sdk, mockServer } = makeSdk({ rpcUrl: "https://rpc.test.mock" });
    mockServer.getEvents = jest.fn().mockResolvedValue({ events: [] });

    const callback = jest.fn();
    const unsubscribe = sdk.subscribeToEvents(callback);

    await Promise.resolve(); // Allow async polling to start
    expect(mockServer.getEvents).toHaveBeenCalled();
    unsubscribe();
  });

  it("normalizes event payload shape", async () => {
    const { sdk, mockServer } = makeSdk({ rpcUrl: "https://rpc.test.mock" });
    const rawEvent = {
      id: "event-123",
      type: "contract",
      contractId: "CD...",
      topic: [],
      value: { _scval: "value" },
      ledger: 100,
      timestamp: 1234567890,
    };

    mockServer.getEvents = jest.fn().mockResolvedValue({ events: [rawEvent] });

    const callback = jest.fn();
    sdk.subscribeToEvents(callback);

    await Promise.resolve(); // Allow async polling to complete

    expect(callback).toHaveBeenCalledWith(
      expect.objectContaining({
        id: "event-123",
        type: "contract",
        contractId: "CD...",
        ledger: 100,
        timestamp: 1234567890,
      })
    );
  });

  it("unsubscribes correctly from polling", async () => {
    const { sdk, mockServer } = makeSdk({ rpcUrl: "https://rpc.test.mock" });
    mockServer.getEvents = jest.fn().mockResolvedValue({ events: [] });

    const callback = jest.fn();
    const unsubscribe = sdk.subscribeToEvents(callback);

    await Promise.resolve();
    unsubscribe();
    jest.runAllTimers();

    // After unsubscribe, getEvents should not be called again
    expect(mockServer.getEvents).toHaveBeenCalledTimes(1);
  });
});
