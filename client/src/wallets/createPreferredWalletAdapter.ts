import { WalletAdapter } from "../types";
import { AlbedoAdapter } from "./AlbedoAdapter";
import { FreighterAdapter } from "./FreighterAdapter";

function hasFreighter(): boolean {
  return typeof window !== "undefined" && Boolean((window as any).freighterApi);
}

function hasAlbedo(): boolean {
  return typeof window !== "undefined" && Boolean((window as any).albedo);
}

/**
 * Creates a signer with seamless Freighter -> Albedo fallback.
 */
export function createPreferredWalletAdapter(networkPassphrase?: string): WalletAdapter {
  if (hasFreighter()) {
    return new FreighterAdapter();
  }
  if (hasAlbedo()) {
    return new AlbedoAdapter(networkPassphrase);
  }
  throw new Error(
    "No supported wallet detected. Install Freighter or Albedo and refresh.",
  );
}
