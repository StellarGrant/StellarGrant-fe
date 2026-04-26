import { WalletAdapter } from "../types";

export class FreighterAdapter implements WalletAdapter {
  async getPublicKey(): Promise<string> {
    const isAllowed = await (window as any).freighterApi?.setAllowed();
    if (!isAllowed) {
      throw new Error("Freighter not allowed or installed");
    }
    const result = await (window as any).freighterApi?.getPublicKey();
    if (result) return result;
    throw new Error("No public key returned by Freighter");
  }

  async signTransaction(txXdr: string, networkPassphrase: string): Promise<string> {
    // network: "TESTNET" | "PUBLIC" | "FUTURENET"
    // freighter expects network string instead of passphrase mostly, but accepts both in newer versions
    let network = "TESTNET";
    if (networkPassphrase.includes("Public")) {
      network = "PUBLIC";
    }

    const { getNetworkDetails } = (window as any).freighterApi || {};
    if (getNetworkDetails) {
      const details = await getNetworkDetails();
      if (details.networkPassphrase !== networkPassphrase) {
        throw new Error(`Freighter is on wrong network. Expected: ${networkPassphrase}`);
      }
      network = details.network;
    }

    const signed = await (window as any).freighterApi?.signTransaction(txXdr, {
      network,
      networkPassphrase,
    });
    
    // Some versions return raw xdr or an object
    if (!signed) throw new Error("Failed to sign transaction with Freighter");
    // Depending on Freighter api standard (v1 vs v2)
    // usually signTransaction resolves to a signed XDR string
    if (typeof signed === "string") return signed;
    if (signed instanceof Object && Buffer.isBuffer(signed)) return signed.toString('base64');
    // If wrapping object
    return signed as string;
  }
}
