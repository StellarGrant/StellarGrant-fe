StellarGrants Protocol
======================

This repository contains three tightly-coupled parts of the StellarGrants Protocol:

1. **Smart contracts** (Soroban, Rust) in `stellargrant-contracts/`
2. **Frontend** (Next.js, zero-backend) in `stellargrant-fe/`
3. **Backend API middleware** (Express, TypeScript) in `api/`

The overall goal is milestone-based grant management on Stellar:
grant creation and funding are handled on-chain, and the frontend reads state directly from Stellar RPC while wallets sign and submit transactions.

----

## Repository Layout

`stellargrant-contracts/`
Smart contracts written in Rust for Soroban.

Typical responsibilities:
- Build and test contracts (compile to WASM)
- Enforce Rust formatting and linting (`cargo fmt`, `cargo clippy`)
- Deploy/invoke the contract using Stellar CLI

`stellargrant-fe/`
Next.js frontend that interacts with the deployed contract via Stellar RPC and wallet extensions.

Typical responsibilities:
- Provide UI flows for grant lifecycle actions (create, fund, submit milestones, approve/vote)
- Read contract state via RPC
- Sign and submit transactions through supported wallets (for example: Freighter and other Stellar wallet providers)

`api/`
Express API middleware for cached grant reads and signature-verified write operations.

Typical responsibilities:
- Cache grant data fetched from Soroban (via contract client adapters)
- Expose backend endpoints for grant listing and metadata
- Validate Stellar signatures for write-intent endpoints (for example milestone proof submission)

----

## Architecture (High Level)

### On-chain: Soroban contract (Rust)
The protocol is implemented as a Soroban smart contract with a modular code structure (for example: contract logic, types, storage helpers, events, and unit tests).

Core concepts:
- **Grants**: milestone-based funding opportunities
- **Milestones**: deliverables that unlock payouts
- **Escrow**: secure holding of funds until milestone approval
- **Voting / approvals**: DAO-style governance for milestone acceptance
- **Events**: emitted on state changes to support off-chain indexing

### Off-chain: Next.js frontend (zero-backend)
The frontend follows a **zero-backend** approach:
- No custom API server in this repository
- The browser/frontend directly reads from Stellar RPC
- Wallets sign transactions in the user's browser, and the frontend submits them to the network

----

## Prerequisites

### For Smart Contracts
- Rust `>= 1.78`
- `wasm32-unknown-unknown` target (install via `rustup`)
- Stellar CLI (for deploy/invoke), installed via `cargo install`

### For the Frontend
- Node.js `>= 18`
- npm (or another package manager, but this repo is currently configured for npm via `package-lock.json`)

----

## Quick Start (Local)

You can work on contracts and frontend independently.

### Smart Contracts: Formatting, Linting, and Build Checks

From the repository root:

```bash
cd stellargrant-contracts
rustup target add wasm32-unknown-unknown

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo check --workspace --all-targets --target wasm32-unknown-unknown
```

### Smart Contracts: Build and Test

From `stellargrant-contracts`, you can build and test the contract workspace using Cargo.

```bash
cd stellargrant-contracts
cargo test
```

If your development workflow uses the contract directory's Makefile (optional), you can use it to build WASM with the project's Soroban tooling:

```bash
cd contracts/stellar-grants
make build
make test
```

### Smart Contracts: Code Coverage

You can run test coverage locally using `cargo-tarpaulin`.

1. Install `cargo-tarpaulin`:
   ```bash
   cargo install cargo-tarpaulin
   ```
2. Run coverage targeting the library logic:
   ```bash
   cd stellargrant-contracts
   cargo tarpaulin --workspace --lib --target x86_64-unknown-linux-gnu --engine llvm --out Xml
   ```
   *Note: Our `.tarpaulin.toml` is configured to exclude test files automatically.*

### Frontend: Install and Run


```bash
cd stellargrant-fe
npm ci
npm run dev
```

The dev server will be available at `http://localhost:3000` (default Next.js behavior).

----

## Frontend Configuration (Environment Variables)

The frontend is configured via environment variables (create a local `.env.local` file and do not commit it).

Required values for typical development:
- `NEXT_PUBLIC_STELLAR_RPC_URL`
  - Stellar RPC endpoint for Soroban reads/writes (example: testnet RPC)
- `NEXT_PUBLIC_NETWORK_PASSPHRASE`
  - Network passphrase for signing transactions
- `NEXT_PUBLIC_CONTRACT_ID`
  - Deployed contract identifier on the selected network

Optional values (depending on features you use):
- Token contract identifiers:
  - `NEXT_PUBLIC_NATIVE_TOKEN`
  - `NEXT_PUBLIC_USDC_TOKEN`
- Milestone proof upload / IPFS gateway:
  - `NEXT_PUBLIC_IPFS_GATEWAY`
- Optional analytics:
  - `NEXT_PUBLIC_POSTHOG_KEY`
  - `NEXT_PUBLIC_POSTHOG_HOST`

Security note:
- Anything prefixed with `NEXT_PUBLIC_` is exposed to browser clients.
- Do not put secrets into `NEXT_PUBLIC_` variables.

----

## Contract Deployment (Testnet / Mainnet)

After building the contract WASM, deploy it using Stellar CLI.

Example flow (testnet):

```bash
cd stellargrant-contracts/contracts/stellar-grants
make build

stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellar_grants.wasm \
  --network testnet \
  --source-account YOUR_SECRET_KEY
```

Example flow (mainnet):

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/stellar_grants.wasm \
  --network mainnet \
  --source-account YOUR_SECRET_KEY
```

Important:
- Replace `YOUR_SECRET_KEY` with credentials stored securely in your local environment.
- Treat deployment and initialization carefully and follow the contract's expected initialization procedure (if required).

----

## CI / GitHub Actions

This repo includes a GitHub Actions workflow at `.github/workflows/ci.yml` that runs on every `push` and `pull_request`.

What it checks:
- **Contracts job**
  - `cargo fmt` (format check)
  - `cargo clippy` (deny warnings)
  - `cargo check` for the contract workspace
- **Frontend job**
  - `npm ci`
  - `npm run lint`
  - `npm run build`

If you add new code, make sure both sides compile/lint cleanly so your PR passes CI.

----

## Contributing

General expectations:
- Keep smart contract code formatted (`cargo fmt`)
- Keep contract warnings-free (`cargo clippy -D warnings`)
- Keep frontend code lint- and build-clean

More guidance:
- `stellargrant-contracts/ContributionGuide.md` (contract-specific contribution rules)
- `stellargrant-fe/CONTRIBUTING.md` (frontend-specific contribution rules, if present in your working copy)

----

## Security

Before deploying changes to real networks:
- Run tests and clippy checks locally and via CI
- Review access control and arithmetic safety in contract changes
- Avoid committing any secrets (private keys, API tokens, etc.)

Report vulnerabilities via GitHub Security Advisories or by contacting the maintainers.

----

## License

MIT License.

