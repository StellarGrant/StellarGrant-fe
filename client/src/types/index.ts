export type StellarGrantsSigner = {
  getPublicKey(): Promise<string>;
  signTransaction(txXdr: string, networkPassphrase: string): Promise<string>;
};

export type StellarGrantsSDKConfig = {
  contractId: string;
  rpcUrl: string;
  networkPassphrase: string;
  signer: StellarGrantsSigner;
  defaultFee?: string;
  pollingIntervalMs?: number;
  pollingTimeoutMs?: number;
};

export type GrantCreateInput = {
  owner: string;
  title: string;
  description: string;
  budget: bigint;
  deadline: bigint;
  milestoneCount: number;
};

export type GrantFundInput = {
  grantId: number;
  token: string;
  amount: bigint;
};

export type MilestoneSubmitInput = {
  grantId: number;
  milestoneIdx: number;
  proofHash: string;
};

export type MilestoneVoteInput = {
  grantId: number;
  milestoneIdx: number;
  approve: boolean;
};
