export { StellarGrantsSDK, CONTRACT_INTERFACE_VERSION } from "./StellarGrantsSDK";
export * from "./types";
export * from "./errors/StellarGrantsError";
export * from "./errors/parseSorobanError";
export * from "./wallets";
export { parseSorobanError } from "./errors/parseSorobanError";
export { SorobanRevertError, StellarGrantsError } from "./errors/StellarGrantsError";
export type {
  GrantCreateInput,
  GrantFundInput,
  MilestoneSubmitInput,
  MilestoneVoteInput,
  StellarGrantsSDKConfig,
  WalletAdapter,
  WriteOptions,
  FeePriority,
  FeeEstimate,
} from "./types";
export { EventParser } from "./events";
export type {
  ParsedEvent,
  GrantCreatedData,
  MilestoneSubmittedData,
  GrantFundedData,
  MilestoneVotedData,
} from "./events";
export {
  xdrToBase64,
  xdrFromBase64,
  appendSignature,
  computeSignatureWeight,
  meetsThreshold,
  PendingXdrStore,
} from "./utils/transactions";
export type { AccountSigner, AccountThresholds } from "./utils/transactions";
export { isNativeXLM, toAssetScVal, NATIVE_XLM_ADDRESS } from "./utils/assets";
export { uploadMetadataToIPFS, fetchMetadataFromIPFS } from "./ipfs";
export {
  GRANT_METADATA_SCHEMA,
  MILESTONE_METADATA_SCHEMA,
  IPFS_METADATA_SCHEMAS,
  inferMetadataSchemaName,
  validateMetadataAgainstSchema,
} from "./metadataSchemas";
export { MetadataValidationError } from "./errors/MetadataValidationError";
export type {
  AllowanceResult,
  AllowanceCheckResult,
  IpfsUploadConfig,
  IpfsUploadResult,
} from "./types";

// Vue composables (optional - requires Vue as peer dependency)
export {
  useStellarGrants,
  useGrants,
  useGrant,
  provideStellarGrants,
} from "./composables";
export type {
  StellarGrantsContext,
  UseGrantsOptions,
  UseGrantsResult,
  UseGrantOptions,
  UseGrantResult,
} from "./composables";
