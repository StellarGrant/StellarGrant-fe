export interface MockServerState {
    simulationError?: string;
    simulationResult?: any;
    minResourceFee?: string;
    sendStatus?: "PENDING" | "ERROR";
    sendErrorResult?: string;
    events?: any[];
    getEventsError?: Error;
}

/**
 * Creates a mock rpc.Server instance with configurable state.
 * Pass the returned `state` object to control per-test behaviour.
 */
export function makeMockServer(): { server: any; state: MockServerState } {
    const state: MockServerState = {
        sendStatus: "PENDING",
    };

    const server = {
        getAccount: jest.fn(async () => ({ accountId: "GABC123TESTPUBLICKEY", sequence: "1" })),

        simulateTransaction: jest.fn(async () => {
            if (state.simulationError) {
                return { error: state.simulationError };
            }
            return {
                result: { retval: state.simulationResult ?? { _mock: "retval" } },
                minResourceFee: state.minResourceFee ?? "1000",
            };
        }),

        prepareTransaction: jest.fn(async (tx: any) => tx),

        sendTransaction: jest.fn(async () => {
            if (state.sendStatus === "ERROR") {
                return { status: "ERROR", errorResult: state.sendErrorResult ?? "unknown error" };
            }
            return { status: "PENDING", hash: "mockhash123" };
        }),

        getEvents: jest.fn(async () => {
            if (state.getEventsError) throw state.getEventsError;
            return { events: state.events ?? [] };
        }),
    };

    return { server, state };
}
