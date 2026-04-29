# Requirements Document

## Introduction

This feature adds a comprehensive unit test suite for the StellarGrantsSDK client library. The suite must cover all public methods, all internal code paths, error handling, wallet adapters, and event subscription logic — without relying on a live Stellar network. All tests run under Jest with ts-jest and use Jest mocks to replace the `@stellar/stellar-sdk` RPC layer.

## Glossary

- **SDK**: The `StellarGrantsSDK` class in `client/src/StellarGrantsSDK.ts`.
- **RPC_Server**: The `rpc.Server` instance used by the SDK to communicate with a Soroban RPC endpoint.
- **Signer**: An object implementing `WalletAdapter` (`getPublicKey`, `signTransaction`).
- **Simulation**: The result of calling `RPC_Server.simulateTransaction`, which may contain a `result.retval` or an `error` field.
- **invokeWrite**: The private SDK method that builds, simulates (optionally), prepares, signs, and sends a write transaction.
- **invokeRead**: The private SDK method that builds, simulates, and parses the result of a read-only transaction.
- **StellarGrantsError**: The base typed error class with `code` and optional `details`.
- **SorobanRevertError**: A subclass of `StellarGrantsError` for contract revert / txFailed conditions.
- **AlbedoAdapter**: A `WalletAdapter` implementation that delegates to `window.albedo`.
- **FreighterAdapter**: A `WalletAdapter` implementation that delegates to `window.freighterApi`.
- **Test_Suite**: The Jest test files located under `client/tests/`.
- **Mock_Server**: A Jest mock of `rpc.Server` that returns controlled responses without network I/O.
- **Mock_Signer**: A Jest mock of `WalletAdapter` that returns deterministic keys and signed XDR strings.

---

## Requirements

### Requirement 1: Mock Infrastructure

**User Story:** As a developer, I want a reusable mock infrastructure for the RPC server and signer, so that every test can set up predictable conditions without duplicating boilerplate.

#### Acceptance Criteria

1. THE Test_Suite SHALL provide a Mock_Server that replaces `rpc.Server` and exposes configurable return values for `simulateTransaction`, `prepareTransaction`, `sendTransaction`, `getAccount`, and `getEvents`.
2. THE Test_Suite SHALL provide a Mock_Signer that implements `WalletAdapter` and returns a fixed public key and a fixed signed XDR string.
3. WHEN a test configures Mock_Server to return a simulation error, THE Mock_Server SHALL include an `error` field in the simulation response.
4. WHEN a test configures Mock_Server to return a send ERROR status, THE Mock_Server SHALL include `status: "ERROR"` and an `errorResult` field in the send response.
5. THE Test_Suite SHALL reset all mock state between tests using Jest's `clearMocks` or `beforeEach` hooks.

---

### Requirement 2: Read Method Coverage

**User Story:** As a developer, I want unit tests for all read methods of the SDK, so that I can verify correct argument encoding and result parsing without a live network.

#### Acceptance Criteria

1. WHEN `grantGet` is called with a grant ID, THE SDK SHALL call `RPC_Server.simulateTransaction` exactly once with a transaction that encodes the grant ID as a `u32` ScVal.
2. WHEN `milestoneGet` is called with a grant ID and milestone index, THE SDK SHALL call `RPC_Server.simulateTransaction` exactly once with a transaction that encodes both arguments as `u32` ScVals.
3. WHEN `simulateTransaction` is called directly with a method name and args, THE SDK SHALL call `RPC_Server.simulateTransaction` and return the raw simulation object.
4. WHEN a simulation returns a `result.retval`, THE SDK SHALL return the native-converted value from `scValToNative`.
5. WHEN a simulation returns no `result.retval`, THE SDK SHALL return `null`.
6. WHEN a simulation returns an `error` field, THE SDK SHALL throw a `StellarGrantsError`.

---

### Requirement 3: Write Method Coverage

**User Story:** As a developer, I want unit tests for all write methods of the SDK, so that I can verify correct argument encoding, signing, and transaction sending without a live network.

#### Acceptance Criteria

1. WHEN `grantCreate` is called with valid input, THE SDK SHALL call `Signer.signTransaction` with the prepared transaction XDR and the configured network passphrase.
2. WHEN `grantFund` is called with valid input, THE SDK SHALL call `RPC_Server.sendTransaction` and return the send result.
3. WHEN `milestoneSubmit` is called with valid input, THE SDK SHALL call `Signer.signTransaction` exactly once.
4. WHEN `milestoneVote` is called with valid input, THE SDK SHALL call `RPC_Server.sendTransaction` exactly once.
5. WHEN `RPC_Server.sendTransaction` returns `status: "ERROR"`, THE SDK SHALL throw a `StellarGrantsError`.

---

### Requirement 4: invokeWrite Option Paths

**User Story:** As a developer, I want unit tests for all option paths of the write invocation logic, so that fee overrides and pre-built transaction data are handled correctly.

#### Acceptance Criteria

1. WHEN `invokeWrite` is called without options, THE SDK SHALL simulate the transaction to determine the fee and add 10000 to `minResourceFee`.
2. WHEN `invokeWrite` is called with `feeMultiplier`, THE SDK SHALL set the fee to `ceil(minResourceFee * feeMultiplier)`.
3. WHEN `invokeWrite` is called with `simulatedFee` and no `feeMultiplier`, THE SDK SHALL use `simulatedFee` as the transaction fee without simulating.
4. WHEN `invokeWrite` is called with `transactionData` and no `feeMultiplier`, THE SDK SHALL skip simulation and call `RPC_Server.prepareTransaction` with the provided data.
5. WHEN `invokeWrite` is called with both `transactionData` and `feeMultiplier`, THE SDK SHALL still simulate to compute the fee using the multiplier.

---

### Requirement 5: Error Handling

**User Story:** As a developer, I want unit tests for all error handling paths, so that I can verify that raw RPC failures are converted to typed errors consistently.

#### Acceptance Criteria

1. WHEN `parseSorobanError` receives an `Error` whose message contains "revert", THE Parser SHALL return a `SorobanRevertError` with a humanized message.
2. WHEN `parseSorobanError` receives an `Error` whose message contains "txfailed" (case-insensitive), THE Parser SHALL return a `SorobanRevertError`.
3. WHEN `parseSorobanError` receives an `Error` that does not contain "revert" or "txfailed", THE Parser SHALL return a `StellarGrantsError` with code `"RPC_ERROR"`.
4. WHEN `parseSorobanError` receives a non-`Error` value, THE Parser SHALL return a `StellarGrantsError` with code `"UNKNOWN_RPC_ERROR"`.
5. WHEN a read or write operation throws any error, THE SDK SHALL wrap it through `parseSorobanError` before re-throwing.

---

### Requirement 6: StellarGrantsError and SorobanRevertError Constructors

**User Story:** As a developer, I want unit tests for the error class constructors, so that I can verify that error instances carry the correct properties.

#### Acceptance Criteria

1. WHEN `StellarGrantsError` is constructed with a message, THE StellarGrantsError SHALL have `name === "StellarGrantsError"`, the provided message, and `code === "STELLAR_GRANTS_ERROR"` by default.
2. WHEN `StellarGrantsError` is constructed with a custom code and details, THE StellarGrantsError SHALL expose those values on `code` and `details`.
3. WHEN `SorobanRevertError` is constructed, THE SorobanRevertError SHALL have `name === "SorobanRevertError"` and `code === "SOROBAN_REVERT"`.
4. THE SorobanRevertError SHALL be an instance of both `SorobanRevertError` and `StellarGrantsError`.

---

### Requirement 7: AlbedoAdapter

**User Story:** As a developer, I want unit tests for the AlbedoAdapter, so that I can verify it correctly delegates to `window.albedo` and caches the public key.

#### Acceptance Criteria

1. WHEN `AlbedoAdapter.getPublicKey` is called and `window.albedo` is available, THE AlbedoAdapter SHALL call `window.albedo.publicKey` and return the `pubkey` field.
2. WHEN `AlbedoAdapter.getPublicKey` is called a second time, THE AlbedoAdapter SHALL return the cached value without calling `window.albedo.publicKey` again.
3. WHEN `AlbedoAdapter.getPublicKey` is called and `window.albedo` is not defined, THE AlbedoAdapter SHALL throw an error.
4. WHEN `AlbedoAdapter.signTransaction` is called with a testnet passphrase, THE AlbedoAdapter SHALL call `window.albedo.tx` with `network: "testnet"` and return `signed_envelope_xdr`.
5. WHEN `AlbedoAdapter.signTransaction` is called with a passphrase containing "Public", THE AlbedoAdapter SHALL call `window.albedo.tx` with `network: "public"`.
6. WHEN `AlbedoAdapter.signTransaction` is called and `window.albedo` is not defined, THE AlbedoAdapter SHALL throw an error.

---

### Requirement 8: FreighterAdapter

**User Story:** As a developer, I want unit tests for the FreighterAdapter, so that I can verify it correctly delegates to `window.freighterApi` and handles network mismatch errors.

#### Acceptance Criteria

1. WHEN `FreighterAdapter.getPublicKey` is called and `window.freighterApi.setAllowed` returns truthy, THE FreighterAdapter SHALL call `window.freighterApi.getPublicKey` and return the result.
2. WHEN `FreighterAdapter.getPublicKey` is called and `window.freighterApi.setAllowed` returns falsy, THE FreighterAdapter SHALL throw an error.
3. WHEN `FreighterAdapter.getPublicKey` is called and `window.freighterApi.getPublicKey` returns null or undefined, THE FreighterAdapter SHALL throw an error.
4. WHEN `FreighterAdapter.signTransaction` is called and `getNetworkDetails` is not available, THE FreighterAdapter SHALL call `window.freighterApi.signTransaction` with the correct network string.
5. WHEN `FreighterAdapter.signTransaction` is called and `getNetworkDetails` returns a passphrase that does not match the expected passphrase, THE FreighterAdapter SHALL throw an error containing the expected passphrase.
6. WHEN `FreighterAdapter.signTransaction` is called and `getNetworkDetails` returns a matching passphrase, THE FreighterAdapter SHALL call `window.freighterApi.signTransaction` with the network from `getNetworkDetails`.
7. WHEN `FreighterAdapter.signTransaction` returns a string, THE FreighterAdapter SHALL return that string directly.

---

### Requirement 9: subscribeToEvents

**User Story:** As a developer, I want unit tests for the event subscription logic, so that I can verify polling, event filtering, cursor advancement, and unsubscription.

#### Acceptance Criteria

1. WHEN `subscribeToEvents` is called, THE SDK SHALL call `RPC_Server.getEvents` with a filter for the configured contract ID.
2. WHEN `subscribeToEvents` is called with a `startLedger` option and no cursor has been set, THE SDK SHALL include `startLedger` in the first `getEvents` request.
3. WHEN `RPC_Server.getEvents` returns events, THE SDK SHALL invoke the callback for each event.
4. WHEN `subscribeToEvents` is called with an `eventName` option, THE SDK SHALL only invoke the callback for events whose topic matches the event name.
5. WHEN `subscribeToEvents` is called with an `eventName` option, THE SDK SHALL not invoke the callback for events whose topic does not match.
6. WHEN events are returned, THE SDK SHALL advance the cursor to the last event's `id` or `pagingToken` for subsequent polls.
7. WHEN the unsubscribe function returned by `subscribeToEvents` is called, THE SDK SHALL stop polling and not invoke the callback for subsequent events.
8. IF `RPC_Server.getEvents` throws an error, THE SDK SHALL log a warning and continue polling without crashing.

---

### Requirement 10: CI Compatibility and Test Performance

**User Story:** As a developer, I want the test suite to run quickly and reliably in CI environments, so that feedback loops are short and builds are stable.

#### Acceptance Criteria

1. THE Test_Suite SHALL complete all tests without making any real network requests.
2. THE Test_Suite SHALL complete all tests in under 30 seconds on standard CI hardware.
3. THE Test_Suite SHALL use Jest fake timers for any tests that involve `setTimeout`-based polling.
4. THE Test_Suite SHALL achieve at least 80% line coverage across `StellarGrantsSDK.ts`, `parseSorobanError.ts`, `StellarGrantsError.ts`, `AlbedoAdapter.ts`, and `FreighterAdapter.ts`.
