# Implementation Plan: sdk-unit-tests

## Overview

Expand the existing `client/tests/sdk.test.ts` into a full unit test suite covering all public SDK methods, all `invokeWrite` option paths, error handling, wallet adapters, and event subscription logic. All tests use Jest mocks — no live network required.

## Tasks

- [x] 1. Set up shared mock infrastructure
  - Create `client/tests/helpers/mockSigner.ts` exporting a `makeMockSigner()` factory that returns a Jest mock implementing `WalletAdapter` with `getPublicKey` returning `"GABC123TESTPUBLICKEY"` and `signTransaction` returning `"SIGNED_XDR_STRING"`
  - Create `client/tests/helpers/mockServer.ts` exporting a `makeMockServer()` factory with configurable state for `simulateTransaction`, `prepareTransaction`, `sendTransaction`, `getAccount`, and `getEvents`
  - Create `client/tests/helpers/sdkFactory.ts` exporting a `makeSdk(overrides?)` helper that wires a `StellarGrantsSDK` instance to the mock server and mock signer
  - _Requirements: 1.1, 1.2, 1.5_

- [x] 2. Implement error class and parseSorobanError tests
  - [x] 2.1 Write tests for `StellarGrantsError` and `SorobanRevertError` constructors
    - Verify `name`, `message`, `code`, and `details` fields for both classes
    - Verify `SorobanRevertError` is `instanceof StellarGrantsError`
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - [x] 2.2 Write property test for SorobanRevertError instanceof chain
    - **Property 9: SorobanRevertError instanceof chain**
    - **Validates: Requirements 6.4**
  - [x] 2.3 Write tests for `parseSorobanError` all branches
    - Test with Error containing "revert" → `SorobanRevertError`
    - Test with Error containing "txFailed" (mixed case) → `SorobanRevertError`
    - Test with Error containing "REVERT" (uppercase) → `SorobanRevertError`
    - Test with plain Error (no keywords) → `StellarGrantsError` with code `"RPC_ERROR"`
    - Test with non-Error string → `StellarGrantsError` with code `"UNKNOWN_RPC_ERROR"`
    - Test with non-Error object → `StellarGrantsError` with code `"UNKNOWN_RPC_ERROR"`
    - _Requirements: 5.1, 5.2, 5.3, 5.4_
  - [x] 2.4 Write property test for parseSorobanError classification rule
    - **Property 2: parseSorobanError classification rule**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4**

- [x] 3. Checkpoint — Ensure error tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement read method tests
  - [x] 4.1 Write tests for `grantGet`
    - Verify `simulateTransaction` is called exactly once
    - Verify the returned value equals the `scValToNative` result
    - Verify `null` is returned when `result.retval` is absent
    - Verify a `StellarGrantsError` is thrown when simulation returns an `error` field
    - _Requirements: 2.1, 2.4, 2.5, 2.6_
  - [x] 4.2 Write property test for read methods calling simulateTransaction exactly once
    - **Property 1: Read methods call simulateTransaction exactly once**
    - **Validates: Requirements 2.1, 2.2**
  - [x] 4.3 Write tests for `milestoneGet`
    - Verify `simulateTransaction` is called exactly once with both args
    - Verify the returned value equals the `scValToNative` result
    - _Requirements: 2.2, 2.4_
  - [x] 4.4 Write tests for `simulateTransaction` (public method)
    - Verify it calls `RPC_Server.simulateTransaction` and returns the raw simulation object
    - Verify it throws when simulation has an error field
    - _Requirements: 2.3_
  - [x] 4.5 Write property test for error wrapping invariant on read operations
    - **Property 11: Error wrapping invariant**
    - **Validates: Requirements 5.5**

- [x] 5. Implement write method tests
  - [x] 5.1 Write tests for `grantCreate`
    - Verify `Signer.signTransaction` is called exactly once with the transaction XDR and network passphrase
    - Verify `RPC_Server.sendTransaction` is called
    - Verify the send result is returned
    - _Requirements: 3.1_
  - [x] 5.2 Write tests for `grantFund`
    - Verify `RPC_Server.sendTransaction` is called and result is returned
    - _Requirements: 3.2_
  - [x] 5.3 Write tests for `milestoneSubmit`
    - Verify `Signer.signTransaction` is called exactly once
    - _Requirements: 3.3_
  - [x] 5.4 Write tests for `milestoneVote`
    - Verify `RPC_Server.sendTransaction` is called exactly once
    - _Requirements: 3.4_
  - [x] 5.5 Write tests for send ERROR status handling
    - Configure Mock_Server to return `{ status: "ERROR", errorResult: "some error" }`
    - Verify a `StellarGrantsError` is thrown
    - _Requirements: 3.5_
  - [x] 5.6 Write property test for all write methods calling signTransaction exactly once
    - **Property 5: All write methods call signTransaction exactly once on success**
    - **Validates: Requirements 3.1, 3.3**

- [x] 6. Implement invokeWrite option path tests
  - [x] 6.1 Write test for default fee computation (no options)
    - Configure Mock_Server to return `minResourceFee: "5000"` from simulation
    - Verify the fee passed to `buildTx` equals `5000 + 10000 = 15000`
    - _Requirements: 4.1_
  - [x] 6.2 Write property test for default fee computation
    - **Property 3: Default fee = minResourceFee + 10000**
    - **Validates: Requirements 4.1**
  - [x] 6.3 Write test for `feeMultiplier` option
    - Configure `minResourceFee: "4000"` and `feeMultiplier: 1.5`
    - Verify fee equals `Math.ceil(4000 * 1.5) = 6000`
    - _Requirements: 4.2_
  - [x] 6.4 Write property test for feeMultiplier fee computation
    - **Property 4: feeMultiplier fee = ceil(minResourceFee * feeMultiplier)**
    - **Validates: Requirements 4.2**
  - [x] 6.5 Write test for `simulatedFee` option (no feeMultiplier)
    - Verify the fee used equals the provided `simulatedFee` string
    - Verify `simulateTransaction` is still called (fee override only affects fee value, not simulation skip)
    - _Requirements: 4.3_
  - [x] 6.6 Write test for `transactionData` option (no feeMultiplier)
    - Verify `RPC_Server.simulateTransaction` is NOT called
    - Verify `RPC_Server.prepareTransaction` is NOT called (skipped when transactionData provided)
    - _Requirements: 4.4_
  - [x] 6.7 Write property test for transactionData skipping simulation
    - **Property 6: transactionData (without feeMultiplier) skips simulation**
    - **Validates: Requirements 4.4**
  - [x] 6.8 Write test for `transactionData` + `feeMultiplier` combination
    - Verify `RPC_Server.simulateTransaction` IS called (feeMultiplier forces simulation)
    - _Requirements: 4.5_

- [x] 7. Checkpoint — Ensure SDK core tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Implement AlbedoAdapter tests
  - [x] 8.1 Write tests for `AlbedoAdapter.getPublicKey`
    - Mock `global.window.albedo` with a `publicKey` jest.fn returning `{ pubkey: "GTEST" }`
    - Verify `getPublicKey` returns `"GTEST"`
    - Verify calling `getPublicKey` twice only calls `window.albedo.publicKey` once (caching)
    - Verify an error is thrown when `window.albedo` is undefined
    - _Requirements: 7.1, 7.2, 7.3_
  - [x] 8.2 Write property test for AlbedoAdapter public key caching
    - **Property 10: AlbedoAdapter public key caching**
    - **Validates: Requirements 7.2**
  - [x] 8.3 Write tests for `AlbedoAdapter.signTransaction`
    - Verify `window.albedo.tx` is called with `network: "testnet"` for testnet passphrase
    - Verify `window.albedo.tx` is called with `network: "public"` for passphrase containing "Public"
    - Verify `signed_envelope_xdr` is returned
    - Verify an error is thrown when `window.albedo` is undefined
    - _Requirements: 7.4, 7.5, 7.6_

- [x] 9. Implement FreighterAdapter tests
  - [x] 9.1 Write tests for `FreighterAdapter.getPublicKey`
    - Mock `global.window.freighterApi` with `setAllowed` returning truthy and `getPublicKey` returning `"GFREIGHTER"`
    - Verify `getPublicKey` returns `"GFREIGHTER"`
    - Verify an error is thrown when `setAllowed` returns falsy
    - Verify an error is thrown when `getPublicKey` returns null
    - _Requirements: 8.1, 8.2, 8.3_
  - [x] 9.2 Write tests for `FreighterAdapter.signTransaction`
    - Test without `getNetworkDetails`: verify `signTransaction` is called with correct network string
    - Test with `getNetworkDetails` returning matching passphrase: verify network from details is used
    - Test with `getNetworkDetails` returning mismatched passphrase: verify error is thrown
    - Verify string return value is passed through directly
    - _Requirements: 8.4, 8.5, 8.6, 8.7_

- [x] 10. Implement subscribeToEvents tests
  - [x] 10.1 Write tests for basic event polling
    - Use `jest.useFakeTimers()` to control `setTimeout`
    - Verify `getEvents` is called with a filter containing the contract ID
    - Verify the callback is invoked once per event returned
    - _Requirements: 9.1, 9.3_
  - [x] 10.2 Write property test for getEvents always called with contract ID filter
    - **Property 12: getEvents always called with contract ID filter**
    - **Validates: Requirements 9.1**
  - [x] 10.3 Write property test for callback called once per event
    - **Property 13: Callback called once per event**
    - **Validates: Requirements 9.3**
  - [x] 10.4 Write test for `startLedger` option
    - Verify `startLedger` is included in the first `getEvents` request when no cursor is set
    - _Requirements: 9.2_
  - [x] 10.5 Write tests for `eventName` filtering
    - Mock events with matching and non-matching topics (base64-encoded ScVal strings)
    - Verify callback is only called for matching events
    - _Requirements: 9.4, 9.5_
  - [x] 10.6 Write property test for event callback filtering
    - **Property 7: Event callback filtering by eventName**
    - **Validates: Requirements 9.4, 9.5**
  - [x] 10.7 Write test for cursor advancement
    - Verify the cursor is updated to the last event's `id` after a poll
    - Verify subsequent polls use `pagination.cursor` instead of `startLedger`
    - _Requirements: 9.6_
  - [x] 10.8 Write test for unsubscribe
    - Call unsubscribe, advance fake timers, verify callback is not called again
    - _Requirements: 9.7_
  - [x] 10.9 Write property test for unsubscribe stopping polling
    - **Property 8: Unsubscribe stops all future callback invocations**
    - **Validates: Requirements 9.7**
  - [x] 10.10 Write test for getEvents error resilience
    - Configure Mock_Server to throw on `getEvents`
    - Verify callback is not called and no unhandled rejection occurs
    - _Requirements: 9.8_

- [x] 11. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- All tasks are required for comprehensive coverage
- Each task references specific requirements for traceability
- Use `jest.useFakeTimers()` / `jest.runAllTimersAsync()` for all polling tests
- Wallet adapter tests mock `window` via `(global as any).window = { albedo: ..., freighterApi: ... }` and restore in `afterEach`
- The `@stellar/stellar-sdk` module mock in `sdk.test.ts` should be extracted to a shared `jest.mock` factory used across all test files
