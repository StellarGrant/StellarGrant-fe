# StellarGrants Client SDK

TypeScript SDK for interacting with the StellarGrants Soroban contract via RPC simulation + transaction submission.

## Getting Started

### Install

```bash
npm install @stellargrants/client-sdk
```

### Create an SDK instance

```ts
import { StellarGrantsSDK } from "@stellargrants/client-sdk";

const sdk = new StellarGrantsSDK({
  contractId: process.env.CONTRACT_ID!,
  rpcUrl: process.env.RPC_URL!,
  networkPassphrase: process.env.NETWORK_PASSPHRASE!,
  signer: {
    async getPublicKey() {
      // return active wallet public key
      return "G...";
    },
    async signTransaction(txXdr, networkPassphrase) {
      // sign using a wallet (Freighter / Albedo / custom signer)
      return txXdr;
    },
  },
});
```

## Configuration

### `StellarGrantsSDKConfig`

- **`contractId`**: Soroban contract id (hex string).
- **`rpcUrl`**: Soroban RPC endpoint URL.
- **`networkPassphrase`**: Network passphrase (e.g. Futurenet / Testnet / Mainnet).
- **`signer`**: An implementation of `StellarGrantsSigner`.
- **`defaultFee`** (optional): Fee in stroops as a string (defaults to `"100"`).

### `StellarGrantsSigner`

- **`getPublicKey()`**: returns the active Stellar address used as the transaction source.
- **`signTransaction(txXdr, networkPassphrase)`**: must return a signed transaction XDR string.

### Wallet Adapters

Built-in adapters:

- `FreighterAdapter`
- `AlbedoAdapter`
- `WalletConnectAdapter`
- `XBullAdapter`
- `createPreferredWalletAdapter(networkPassphrase)` for automatic Freighter -> Albedo fallback.

Example:

```ts
import { createPreferredWalletAdapter, StellarGrantsSDK } from "@stellargrants/client-sdk";

const networkPassphrase = "Test SDF Network ; September 2015";
const signer = createPreferredWalletAdapter(networkPassphrase);

const sdk = new StellarGrantsSDK({
  contractId: process.env.CONTRACT_ID!,
  rpcUrl: process.env.RPC_URL!,
  networkPassphrase,
  signer,
});
```

## Public API

### `StellarGrantsSDK`

- **`grantCreate(input)`**: create a grant.
- **`grantFund(input)`**: fund a grant.
- **`milestoneSubmit(input)`**: submit milestone proof hash.
- **`milestoneVote(input)`**: vote approve/reject on a milestone.
- **`grantGet(grantId)`**: read grant details.
- **`milestoneGet(grantId, milestoneIdx)`**: read milestone details.

### Dynamic Fee Estimation

`estimateFees()` now uses dynamic network load data from Horizon when
`horizonUrl` (or `feeStatsEndpoint`) is configured:

```ts
const sdk = new StellarGrantsSDK({
  contractId,
  rpcUrl,
  networkPassphrase,
  signer,
  horizonUrl: "https://horizon-testnet.stellar.org",
});

const fees = await sdk.estimateFees("grant_create", []);
console.log(fees.source, fees.networkLoad, fees.modifiers);
```

When fee stats are unavailable, the SDK safely falls back to static defaults.

### IPFS Metadata Schema Validation

`uploadMetadataToIPFS()` validates metadata locally before upload to prevent
storing malformed payloads and to avoid unnecessary on-chain calls.

Built-in schemas:

- `grant`
- `milestone`

```ts
import { uploadMetadataToIPFS } from "@stellargrants/client-sdk";

await uploadMetadataToIPFS(
  {
    title: "Open-source educational grant",
    description: "Funding curriculum and mentorship",
  },
  {
    pinataJwt: process.env.PINATA_JWT,
    metadataSchema: "grant",
  },
);
```

Invalid payloads throw `MetadataValidationError` with field-level details.

## CLI

The SDK package ships with a developer CLI available through `npx`:

```bash
npx @stellargrants/client-sdk init
npx @stellargrants/client-sdk grant-status 1 --format json
npx @stellargrants/client-sdk fund-grant 1 --token CTOKEN... --amount 1000000
```

Supported commands:

- `init` - write a starter `.env`
- `grant-status` - query grant state
- `fund-grant` - fund a grant with local secret-key signing

## Errors

- **`StellarGrantsError`**: base SDK error with `code` and optional `details`.
- **`SorobanRevertError`**: thrown when the contract reverts (code `SOROBAN_REVERT`).
- **`parseSorobanError(error)`**: converts raw RPC failures into typed errors.

## Examples

See `client/examples/` for copy/paste scripts:

- `create-grant.ts`
- `vote-on-milestone.ts`

> These examples are intended as starting points. You must provide a real signer implementation.

