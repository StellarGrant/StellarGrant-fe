# Design Document: sdk-unit-tests

## Overview

This document describes the design for a comprehensive unit test suite for the `StellarGrantsSDK` client library. The suite replaces all network I/O with Jest mocks, covers every public method and internal code path, and is structured to run fast and deterministically in CI.

The existing test file (`client/tests/sdk.test.ts`) has only 3 tests. This suite expands it into a full coverage suite organized by module, using a shared mock factory and Jest fake timers for polling tests.

---

## Architecture

```
client/
  tests/
    sdk.test.ts              ← existing (will be replaced/expanded)
    __mocks__/
      stellar-sdk.mock.ts    ← reusable mock factory for @stellar/stellar-sdk
    helpers/
      mockSigner.ts          ← reusable Mock_Signer factory
  src/
    StellarGrantsSDK.ts      ← subject under test
    errors/
      StellarGrantsError.ts  ← subject under test
      parseSorobanError.ts   ← subject under test
    wallets/
      AlbedoAdapter.ts       ← subject under test
      FreighterAdapter.ts    ← subject under test
```

All test files live under `client/tests/`. The Jest config already points `roots` at `<rootDir>/tests`, so no config changes are needed.

---

## Components and Interfaces

### Mock_Server

A Jest mock class that replaces `rpc.Server`. It exposes a mutable state object so individual tests can configure responses:

```typescript
interface MockServerState {
  simulationResult?: any;       // returned by simulateTransaction
  simulationError?: string;     // if set, simulation returns { error }
  sendStatus?: string;          // "PENDING" | "ERROR"
  sendErrorResult?: string;     // populated when sendStatus === "ERROR"
  events?: any[];               // returned by getEvents
  getEventsError?: Error;       // if set, getEvents throws
}
```

Methods:
- `getAccount()` → returns a stub account
- `simulateTransaction()` → returns `{ error }` or `{ result: { retval } }`
- `prepareTransaction(tx)` → returns `tx` unchanged
- `sendTransaction()` → returns `{ status, hash?, errorResult? }`
- `getEvents(req)` → returns `{ events }` or throws

### Mock_Signer

A simple object implementing `WalletAdapter`:

```typescript
const mockSigner = {
  getPublicKey: jest.fn(async () => "GABC123TESTPUBLICKEY"),
  signTransaction: jest.fn(async () => "SIGNED_XDR_STRING"),
};
```

### SDK Factory Helper

A helper function that creates a `StellarGrantsSDK` instance wired to the Mock_Server and Mock_Signer:

```typescript
function makeSdk(overrides?: Partial<StellarGrantsSDKConfig>): StellarGrantsSDK
```

---

## Data Models

### Test Fixtures

Canonical input fixtures used across tests:

```typescript
const GRANT_CREATE_INPUT: GrantCreateInput = {
  owner: "GOWNER123",
  title: "Test Grant",
  description: "A test grant",
  budget: 1000000n,
  deadline: 9999999n,
  milestoneCount: 3,
};

const GRANT_FUND_INPUT: GrantFundInput = {
  grantId: 1,
  token: "GCTOKEN123",
  amount: 500000n,
};

const MILESTONE_SUBMIT_INPUT: MilestoneSubmitInput = {
  grantId: 1,
  milestoneIdx: 0,
  proofHash: "abc123hash",
};

const MILESTONE_VOTE_INPUT: MilestoneVoteInput = {
  grantId: 1,
  milestoneIdx: 0,
  approve: true,
};
```

### Mock Event Shape

```typescript
interface MockEvent {
  id: string;
  pagingToken?: string;
  topic?: string[];   // base64-encoded ScVal strings
  value?: any;
}
```

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Error wrapping invariant

*For any* error thrown inside `invokeRead` or `invokeWrite`, the error that propagates to the caller SHALL be an instance of `StellarGrantsError` (or a subclass).

**Validates: Requirements 5.5**

---

### Property 2: parseSorobanError classification

*For any* `Error` object whose lowercased message contains "revert" or "txfailed", `parseSorobanError` SHALL return a `SorobanRevertError`; for any other `Error` it SHALL return a `StellarGrantsError` with code `"RPC_ERROR"`; for any non-`Error` value it SHALL return a `StellarGrantsError` with code `"UNKNOWN_RPC_ERROR"`.

**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

---

### Property 3: Fee computation invariant

*For any* `minResourceFee` value returned by simulation and any `feeMultiplier` value, the fee used in the transaction SHALL equal `ceil(minResourceFee * feeMultiplier)`.

**Validates: Requirements 4.2**

---

### Property 4: Default fee computation invariant

*For any* `minResourceFee` value returned by simulation when no `feeMultiplier` is provided, the fee used in the transaction SHALL equal `minResourceFee + 10000`.

**Validates: Requirements 4.1**

---

### Property 5: Signer always called for write operations

*For any* write method call (`grantCreate`, `grantFund`, `milestoneSubmit`, `milestoneVote`) that does not result in a simulation error, `Signer.signTransaction` SHALL be called exactly once.

**Validates: Requirements 3.1, 3.3**

---

### Property 6: Simulation skip when transactionData provided (no feeMultiplier)

*For any* call to a write method with `options.transactionData` set and `options.feeMultiplier` absent, `RPC_Server.simulateTransaction` SHALL NOT be called.

**Validates: Requirements 4.4**

---

### Property 7: Event callback filtering

*For any* set of events returned by `getEvents` and any `eventName` filter, the callback SHALL be invoked exactly for those events whose topic decodes to the given event name, and not for others.

**Validates: Requirements 9.4, 9.5**

---

### Property 8: Unsubscribe stops polling

*For any* subscription, calling the returned unsubscribe function SHALL cause no further callback invocations even if `getEvents` would return more events.

**Validates: Requirements 9.7**

---

### Property 9: StellarGrantsError instanceof chain

*For any* `SorobanRevertError` instance, `instanceof StellarGrantsError` SHALL be `true` and `instanceof SorobanRevertError` SHALL be `true`.

**Validates: Requirements 6.4**

---

### Property 10: AlbedoAdapter public key caching

*For any* `AlbedoAdapter` instance, calling `getPublicKey` N times SHALL result in `window.albedo.publicKey` being called exactly once regardless of N (when N ≥ 1).

**Validates: Requirements 7.2**

---

## Error Handling

### Simulation errors

`ensureSimulationSuccess` checks `simulation.error` and throws `StellarGrantsError`. This is caught by the `try/catch` in `invokeRead` and `invokeWrite`, which re-throws via `parseSorobanError`.

### Send errors

After `sendTransaction`, if `sent.status === "ERROR"`, a `StellarGrantsError` is thrown directly (before `parseSorobanError`). The outer `catch` then wraps it again — since `StellarGrantsError` is already an `Error` without "revert"/"txfailed" in its message, `parseSorobanError` returns a new `StellarGrantsError` with code `"RPC_ERROR"`.

### Event polling errors

`subscribeToEvents` catches errors from `getEvents` with a `console.warn` and continues polling. Tests should verify the callback is not called and polling continues.

### Wallet adapter errors

Both adapters throw plain `Error` objects when `window.albedo` / `window.freighterApi` is unavailable or returns unexpected values. Tests mock `window` globals.

---

## Testing Strategy

### Framework

- **Jest 29** with **ts-jest** (already configured in `jest.config.js`)
- **testEnvironment: "node"** — wallet adapter tests mock `window` manually via `global`
- No additional libraries needed; Jest's built-in fake timers handle `setTimeout` polling

### Test File Organization

```
client/tests/
  sdk.test.ts                  ← StellarGrantsSDK public methods + invokeWrite options
  errors.test.ts               ← StellarGrantsError, SorobanRevertError, parseSorobanError
  wallets/
    albedo.test.ts             ← AlbedoAdapter
    freighter.test.ts          ← FreighterAdapter
  events.test.ts               ← subscribeToEvents polling, filtering, unsubscribe
```

### Dual Testing Approach

**Unit tests** cover:
- Specific input/output examples for each public method
- Edge cases: null retval, missing `window.albedo`, network mismatch
- Error conditions: simulation error, send ERROR status, non-Error thrown

**Property tests** are not applicable here in the traditional PBT sense because the SDK's correctness depends on mock interactions rather than data transformations over large input spaces. The "properties" above are verified as parameterized unit tests (e.g., testing fee computation with several `minResourceFee` and `feeMultiplier` values).

### Fake Timers

`subscribeToEvents` uses `setTimeout` for polling. Tests use `jest.useFakeTimers()` and `jest.runAllTimersAsync()` / `jest.advanceTimersByTimeAsync()` to control the polling loop without real delays.

### Coverage Target

Jest coverage is configured with `collectCoverageFrom` targeting:
- `src/StellarGrantsSDK.ts`
- `src/errors/*.ts`
- `src/wallets/*.ts`

Target: ≥ 80% line coverage on all listed files.

### Running Tests

```bash
cd client
npm test
# or with coverage:
npx jest --coverage --runInBand
```


---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Read methods call simulateTransaction exactly once

*For any* valid input to `grantGet` or `milestoneGet`, calling the method SHALL result in `RPC_Server.simulateTransaction` being called exactly once.

**Validates: Requirements 2.1, 2.2**

---

### Property 2: parseSorobanError classification rule

*For any* `Error` whose lowercased message contains "revert" or "txfailed", `parseSorobanError` SHALL return a `SorobanRevertError`. *For any* `Error` whose message does not contain those keywords, it SHALL return a `StellarGrantsError` with code `"RPC_ERROR"`. *For any* non-`Error` value, it SHALL return a `StellarGrantsError` with code `"UNKNOWN_RPC_ERROR"`.

**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

---

### Property 3: Default fee = minResourceFee + 10000

*For any* `minResourceFee` value returned by simulation when no `feeMultiplier` option is provided, the fee used in the built transaction SHALL equal `Number(minResourceFee) + 10000`.

**Validates: Requirements 4.1**

---

### Property 4: feeMultiplier fee = ceil(minResourceFee * feeMultiplier)

*For any* `minResourceFee` value and any `feeMultiplier` value, the fee used in the built transaction SHALL equal `Math.ceil(Number(minResourceFee) * feeMultiplier)`.

**Validates: Requirements 4.2**

---

### Property 5: All write methods call signTransaction exactly once on success

*For any* valid input to `grantCreate`, `grantFund`, `milestoneSubmit`, or `milestoneVote` that does not result in a simulation or send error, `Signer.signTransaction` SHALL be called exactly once.

**Validates: Requirements 3.1, 3.3**

---

### Property 6: transactionData (without feeMultiplier) skips simulation

*For any* write method call where `options.transactionData` is provided and `options.feeMultiplier` is absent, `RPC_Server.simulateTransaction` SHALL NOT be called.

**Validates: Requirements 4.4**

---

### Property 7: Event callback filtering by eventName

*For any* list of events returned by `getEvents` and any `eventName` filter, the callback SHALL be invoked exactly for those events whose topic decodes to the given event name, and SHALL NOT be invoked for events whose topic does not match.

**Validates: Requirements 9.4, 9.5**

---

### Property 8: Unsubscribe stops all future callback invocations

*For any* active subscription, calling the returned unsubscribe function SHALL cause zero additional callback invocations even if `getEvents` subsequently returns events.

**Validates: Requirements 9.7**

---

### Property 9: SorobanRevertError instanceof chain

*For any* `SorobanRevertError` instance, `instanceof StellarGrantsError` SHALL be `true` and `instanceof SorobanRevertError` SHALL be `true`.

**Validates: Requirements 6.4**

---

### Property 10: AlbedoAdapter public key caching

*For any* `AlbedoAdapter` instance, calling `getPublicKey` N times (N ≥ 1) SHALL result in `window.albedo.publicKey` being called exactly once.

**Validates: Requirements 7.2**

---

### Property 11: Error wrapping invariant

*For any* error thrown inside `invokeRead` or `invokeWrite`, the error that propagates to the caller SHALL be an instance of `StellarGrantsError` (or a subclass).

**Validates: Requirements 5.5**

---

### Property 12: getEvents always called with contract ID filter

*For any* call to `subscribeToEvents` with any contract ID, the `getEvents` request SHALL include a filter containing that exact contract ID.

**Validates: Requirements 9.1**

---

### Property 13: Callback called once per event

*For any* list of N events returned by `getEvents` (with no `eventName` filter), the callback SHALL be invoked exactly N times.

**Validates: Requirements 9.3**
