import { WalletAdapter } from "../../src/types";

export type MockSigner = {
    getPublicKey: jest.Mock<Promise<string>>;
    signTransaction: jest.Mock<Promise<string>>;
} & WalletAdapter;

export function makeMockSigner(): MockSigner {
    return {
        getPublicKey: jest.fn(async () => "GABC123TESTPUBLICKEY"),
        signTransaction: jest.fn(async (_txXdr: string, _passphrase: string) => "SIGNED_XDR_STRING"),
    };
}
