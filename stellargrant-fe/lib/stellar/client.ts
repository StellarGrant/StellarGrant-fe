/**
 * Stellar RPC & Horizon Client Singletons
 *
 * Centralized network client configuration for Stellar interactions.
 * - Soroban RPC: contract invocations and ledger state
 * - Horizon: account balances, trustlines, and transaction history
 */

import { rpc, Horizon } from "@stellar/stellar-sdk";

const rpcUrl =
  process.env.NEXT_PUBLIC_STELLAR_RPC_URL || "https://soroban-testnet.stellar.org";
const networkPassphrase =
  process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || "Test SDF Network ; September 2015";
const horizonUrl =
  process.env.NEXT_PUBLIC_HORIZON_URL || "https://horizon-testnet.stellar.org";

// Soroban RPC client singleton
export const rpcClient = new rpc.Server(rpcUrl, {
  allowHttp: rpcUrl.startsWith("http://"),
});

// Horizon client singleton (provides account balances and trustlines)
export const horizonClient = new Horizon.Server(horizonUrl, {
  allowHttp: horizonUrl.startsWith("http://"),
});

export const networkPassphraseConfig = networkPassphrase;

export function getRpcClient(): rpc.Server {
  return rpcClient;
}

export function getHorizonClient(): Horizon.Server {
  return horizonClient;
}
