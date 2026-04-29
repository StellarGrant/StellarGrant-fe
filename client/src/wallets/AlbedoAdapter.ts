import { WalletAdapter } from "../types";

type AlbedoNetwork = "testnet" | "public";

export class AlbedoAdapter implements WalletAdapter {
  private publicKeyCache: string | null = null;
  private network: AlbedoNetwork = "testnet";

  constructor(networkPassphrase?: string) {
    this.network = this.resolveNetwork(networkPassphrase ?? "");
  }

  async getPublicKey(): Promise<string> {
    if (this.publicKeyCache) return this.publicKeyCache;
    const albedo = (window as any).albedo;
    if (!albedo) throw new Error("Albedo is not installed or available");

    const response = await this.executeWithPopupGuard<{ pubkey?: string }>(() => albedo.publicKey({}));
    if (!response?.pubkey) {
      throw new Error("Albedo did not return a public key.");
    }

    this.publicKeyCache = response.pubkey;
    return response.pubkey;
  }

  async signTransaction(txXdr: string, networkPassphrase: string): Promise<string> {
    const albedo = (window as any).albedo;
    if (!albedo) throw new Error("Albedo is not installed or available");

    const network = this.resolveNetwork(networkPassphrase || this.network);

    const response = await this.executeWithPopupGuard<{ signed_envelope_xdr?: string }>(() => albedo.tx({
      xdr: txXdr,
      network,
    }));

    if (!response?.signed_envelope_xdr) {
      throw new Error("Albedo did not return a signed transaction envelope.");
    }

    return response.signed_envelope_xdr;
  }

  private resolveNetwork(networkPassphrase: string): AlbedoNetwork {
    if (networkPassphrase.includes("Public")) {
      return "public";
    }
    // Albedo prompts currently support testnet/public. Futurenet and custom
    // passphrases are mapped to testnet for compatibility.
    return "testnet";
  }

  private async executeWithPopupGuard<T>(cb: () => Promise<T>): Promise<T> {
    try {
      return await cb();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (/popup|blocked|denied|closed|cancel/i.test(message)) {
        throw new Error(
          "Albedo popup was blocked or closed. Enable popups for this site and try again.",
        );
      }
      throw error;
    }
  }
}
