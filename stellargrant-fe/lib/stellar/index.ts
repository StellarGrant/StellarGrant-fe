export { getRpcClient, rpcClient, networkPassphraseConfig } from "./client";
export { ContractClient, contractClient } from "./contract";
export { fetchContractEvents, decodeEvent } from "./events";
export type { ContractEvent } from "./events";

// Resilient event subscription (Issue #254)
export { subscribeToEvents } from "./subscription";
export type {
  EventHandler,
  StatusHandler,
  SubscriptionStatus,
  SubscribeOptions,
  Subscription,
} from "./subscription";
