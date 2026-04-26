import { WalletAdapter } from "../types";

export class AlbedoAdapter implements WalletAdapter {
  private publicKeyCache: string | null = null;
  private network: "testnet" | "public" = "testnet";

  constructor(networkPassphrase?: string) {
    if (networkPassphrase?.includes("Public")) {
      this.network = "public";
    }
  }

  async getPublicKey(): Promise<string> {
    if (this.publicKeyCache) return this.publicKeyCache;
    const albedo = (window as any).albedo;
    if (!albedo) throw new Error("Albedo is not installed or available");

    const response = await albedo.publicKey({});
    this.publicKeyCache = response.pubkey;
    return response.pubkey;
  }

  async signTransaction(txXdr: string, networkPassphrase: string): Promise<string> {
    const albedo = (window as any).albedo;
    if (!albedo) throw new Error("Albedo is not installed or available");

    let network = "testnet";
    if (networkPassphrase.includes("Public")) {
      network = "public";
    }

    const response = await albedo.tx({
      xdr: txXdr,
      network,
    });

    return response.signed_envelope_xdr;
  }
}
