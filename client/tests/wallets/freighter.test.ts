import { FreighterAdapter } from "../../src/wallets/FreighterAdapter";

const TESTNET_PASSPHRASE = "Test SDF Network ; September 2015";
const MAINNET_PASSPHRASE = "Public Global Stellar Network ; September 2015";

function makeFreighterWindow(overrides?: {
    setAllowed?: jest.Mock;
    getPublicKey?: jest.Mock;
    signTransaction?: jest.Mock;
    getNetworkDetails?: jest.Mock;
}) {
    return {
        freighterApi: {
            setAllowed: overrides?.setAllowed ?? jest.fn(async () => true),
            getPublicKey: overrides?.getPublicKey ?? jest.fn(async () => "GFREIGHTER_KEY"),
            signTransaction: overrides?.signTransaction ?? jest.fn(async () => "FREIGHTER_SIGNED_XDR"),
            ...(overrides?.getNetworkDetails !== undefined
                ? { getNetworkDetails: overrides.getNetworkDetails }
                : {}),
        },
    };
}

afterEach(() => {
    delete (global as any).window;
});

describe("FreighterAdapter.getPublicKey", () => {
    it("returns public key when setAllowed is truthy", async () => {
        (global as any).window = makeFreighterWindow();
        const adapter = new FreighterAdapter();

        const key = await adapter.getPublicKey();

        expect(key).toBe("GFREIGHTER_KEY");
    });

    it("throws when setAllowed returns falsy", async () => {
        (global as any).window = makeFreighterWindow({
            setAllowed: jest.fn(async () => false),
        });
        const adapter = new FreighterAdapter();

        await expect(adapter.getPublicKey()).rejects.toThrow("Freighter not allowed or installed");
    });

    it("throws when getPublicKey returns null", async () => {
        (global as any).window = makeFreighterWindow({
            getPublicKey: jest.fn(async () => null),
        });
        const adapter = new FreighterAdapter();

        await expect(adapter.getPublicKey()).rejects.toThrow("No public key returned by Freighter");
    });

    it("throws when getPublicKey returns undefined", async () => {
        (global as any).window = makeFreighterWindow({
            getPublicKey: jest.fn(async () => undefined),
        });
        const adapter = new FreighterAdapter();

        await expect(adapter.getPublicKey()).rejects.toThrow("No public key returned by Freighter");
    });
});

describe("FreighterAdapter.signTransaction", () => {
    it("calls signTransaction with TESTNET network when no getNetworkDetails", async () => {
        (global as any).window = makeFreighterWindow();
        const adapter = new FreighterAdapter();

        const result = await adapter.signTransaction("TX_XDR", TESTNET_PASSPHRASE);

        expect((global as any).window.freighterApi.signTransaction).toHaveBeenCalledWith("TX_XDR", {
            network: "TESTNET",
            networkPassphrase: TESTNET_PASSPHRASE,
        });
        expect(result).toBe("FREIGHTER_SIGNED_XDR");
    });

    it("calls signTransaction with PUBLIC network for mainnet passphrase when no getNetworkDetails", async () => {
        (global as any).window = makeFreighterWindow();
        const adapter = new FreighterAdapter();

        await adapter.signTransaction("TX_XDR", MAINNET_PASSPHRASE);

        expect((global as any).window.freighterApi.signTransaction).toHaveBeenCalledWith("TX_XDR", {
            network: "PUBLIC",
            networkPassphrase: MAINNET_PASSPHRASE,
        });
    });

    it("uses network from getNetworkDetails when passphrase matches", async () => {
        (global as any).window = makeFreighterWindow({
            getNetworkDetails: jest.fn(async () => ({
                networkPassphrase: TESTNET_PASSPHRASE,
                network: "TESTNET",
            })),
        });
        const adapter = new FreighterAdapter();

        await adapter.signTransaction("TX_XDR", TESTNET_PASSPHRASE);

        expect((global as any).window.freighterApi.signTransaction).toHaveBeenCalledWith("TX_XDR", {
            network: "TESTNET",
            networkPassphrase: TESTNET_PASSPHRASE,
        });
    });

    it("throws when getNetworkDetails returns mismatched passphrase", async () => {
        (global as any).window = makeFreighterWindow({
            getNetworkDetails: jest.fn(async () => ({
                networkPassphrase: MAINNET_PASSPHRASE,
                network: "PUBLIC",
            })),
        });
        const adapter = new FreighterAdapter();

        await expect(adapter.signTransaction("TX_XDR", TESTNET_PASSPHRASE)).rejects.toThrow(
            TESTNET_PASSPHRASE,
        );
    });

    it("returns string result directly", async () => {
        (global as any).window = makeFreighterWindow({
            signTransaction: jest.fn(async () => "DIRECT_STRING_XDR"),
        });
        const adapter = new FreighterAdapter();

        const result = await adapter.signTransaction("TX_XDR", TESTNET_PASSPHRASE);

        expect(result).toBe("DIRECT_STRING_XDR");
    });

    it("throws when signTransaction returns falsy", async () => {
        (global as any).window = makeFreighterWindow({
            signTransaction: jest.fn(async () => null),
        });
        const adapter = new FreighterAdapter();

        await expect(adapter.signTransaction("TX_XDR", TESTNET_PASSPHRASE)).rejects.toThrow(
            "Failed to sign transaction with Freighter",
        );
    });
});
