export { getRpcClient, rpcClient, networkPassphraseConfig, getHorizonClient, horizonClient } from "./client";
export { ContractClient, contractClient } from "./contract";
export { fetchContractEvents, decodeEvent } from "./events";
export type { ContractEvent } from "./events";
export { BatchBuilder } from "./batchBuilder";
export type { BatchOperation, BatchResult } from "./batchBuilder";
export {
  getGrantBalances,
  getGrantXlmBalance,
  getGrantTokenBalance,
  listenToBalanceChanges,
  parseBalanceToStroops,
  formatStroops,
} from "./balances";
export type {
  GrantBalance,
  GrantBalances,
  BalanceChangeListenerOptions,
} from "./balances";

// Multi-signature transaction support
export {
  buildUnsignedTransaction,
  combineSignatures,
  submitSignedXdr,
  isValidTransactionXdr,
  MultiSigTracker,
} from "./multisig";
export type {
  TransactionXdr,
  SignerStatus,
  SignerEntry,
  MultiSigStatus,
  BuildUnsignedTxOptions,
  SubmitOptions,
} from "./multisig";

// Grant search, filter, and sort utilities (Issue #253)
export {
  StellarGrantsSDK,
  stellarGrantsSDK,
  grantListAll,
  grantFilterByCategory,
  grantSearchByTitle,
  sortGrants,
  queryGrants,
} from "./sdk";
export type {
  PaginationOptions,
  GrantListResult,
  SortOptions,
  SortDirection,
  GrantSortField,
  QueryGrantsOptions,
  GrantWithMilestones,
} from "./sdk";
