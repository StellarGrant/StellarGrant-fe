/**
 * Stellar RPC Client Singleton
 *
 * Centralized RPC client configuration for Stellar network interactions.
 */

import { rpc } from "@stellar/stellar-sdk";

const rpcUrl =
  process.env.NEXT_PUBLIC_STELLAR_RPC_URL || "https://soroban-testnet.stellar.org";
const networkPassphrase =
  process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || "Test SDF Network ; September 2015";

// Create RPC client singleton
export const rpcClient = new rpc.Server(rpcUrl, {
  allowHttp: rpcUrl.startsWith("http://"),
});

export const networkPassphraseConfig = networkPassphrase;

export function getRpcClient(): rpc.Server {
  return rpcClient;
}
