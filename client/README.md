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

## Public API

### `StellarGrantsSDK`

- **`grantCreate(input)`**: create a grant.
- **`grantFund(input)`**: fund a grant.
- **`milestoneSubmit(input)`**: submit milestone proof hash.
- **`milestoneVote(input)`**: vote approve/reject on a milestone.
- **`grantGet(grantId)`**: read grant details.
- **`milestoneGet(grantId, milestoneIdx)`**: read milestone details.

## Errors

- **`StellarGrantsError`**: base SDK error with `code` and optional `details`.
- **`SorobanRevertError`**: thrown when the contract reverts (code `SOROBAN_REVERT`).
- **`parseSorobanError(error)`**: converts raw RPC failures into typed errors.

## Examples

See `client/examples/` for copy/paste scripts:

- `create-grant.ts`
- `vote-on-milestone.ts`

> These examples are intended as starting points. You must provide a real signer implementation.

