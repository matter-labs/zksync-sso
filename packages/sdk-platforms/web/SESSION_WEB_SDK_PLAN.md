# Session Support for Web SDK - Implementation Plan

## Status Summary (Updated Nov 6, 2025)

- ✅ **Phase 1**: Rust/WASM FFI Layer - COMPLETED
- ✅ **Phase 2**: TypeScript/Web SDK Updates - COMPLETED (core wiring)
- ✅ **Phase 3**: Vue Component Updates - COMPLETED
- ⏳ **Phase 4**: Session Transaction Sending - PENDING
- ⏳ **Phase 5**: Testing & Validation - PENDING (e2e for session-signed tx)

**Recent Achievements**:

- Both session install flows (deploy-with-session and post-deploy) implemented
  and tested
- End-to-end Playwright tests passing for session installation
- Reusable SessionConfig Vue component extracted
- Documentation updated for contracts.json configuration

**Next Steps**: Implement Phase 4 (session-signed transaction prepare/submit
helpers and UI) with corresponding e2e test coverage.

## Overview

Add session functionality to the Web SDK to enable gasless transactions within defined
limits. Sessions allow users to authorize specific operations (transfers, contract
calls) with limits on fees and values, signed by a separate session key.

## Architecture

### Session Structure

```typescript
{
  signer: Address,           // Session key address
  expiresAt: U48,           // Unix timestamp
  feeLimit: UsageLimit,     // Max fees session can pay
  transfers: [TransferSpec] // Array of allowed transfers
}

TransferSpec {
  to: Address,              // Target address
  valueLimit: U256          // Max ETH per transfer
}

UsageLimit {
  limitType: 'unlimited' | 'lifetime' | 'allowance',
  limit: U256,              // Total limit amount
  period: U48               // Period for allowance type
}
```

## Phase 1: Rust/WASM FFI Layer Updates ✅ (Completed Nov 6, 2025)

Phase 1 is complete. We implemented the session WASM types, added both post-deploy
and deploy-with-session flows to the FFI, and validated with automated tests
mirroring core. The FFI and Rust core now support installing a session at account
creation (deploy-with-session) as well as post-deploy. All flows are covered by
tests.

### 1.1 Create Session-Related WASM Types ✅

**File**: `packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-ffi-web/src/lib.rs`

**Status**: COMPLETED

- Added `TransferPayload` struct with getters
- Added `SessionPayload` struct with getters
- Types match core session specification

### 1.2 Update `DeployAccountConfig` ✅

**Status**: COMPLETED

- Added `session_validator_address: Option<String>` field
- Added getter method
- Updated constructor

### 1.3 Update `deploy_account` Function Signature ✅

**Status**: COMPLETED

- Added `session_payload: Option<SessionPayload>` parameter
- Updated documentation

### 1.4 Add Session Support to Core Deploy Function ✅

**Status**: COMPLETED

Deploy-with-session is now supported and tested. The recommended path is to install
the SessionKeyValidator and create a session either during deployment or immediately
after. Both flows are validated in FFI and core tests.

### 1.5 Alternative: Add Session Post-Deployment 🔄

**Status**: COMPLETED

Instead of adding session during deployment,
we can add a function to install a session module after deployment:

```rust
#[wasm_bindgen]
pub fn add_session_to_account(
  config: SendTransactionConfig,
  account_address: String,
  session_payload: SessionPayload,
  session_validator_address: String,
  eoa_validator_address: String,
  eoa_private_key: String,
) -> js_sys::Promise
```

This approach:

- Works with existing core functions
- Follows same pattern as `add_passkey_to_account`
- Allows flexible session management
- Doesn't require core deploy changes

Implementation:

- Function implemented in `zksync-sso-erc4337-ffi-web/src/lib.rs` as
  `add_session_to_account`
- Converts WASM `SessionPayload` to core `SessionSpec` and calls core helpers to
  install the module and create a session

### 1.6 Session Transaction Functions ▶ Deferred

**Status**: DEFERRED to Phase 4 (Session Transaction Sending)

We will add prepare/submit helpers for session-signed transactions alongside the
broader "Session Transaction Sending" work to keep responsibilities grouped.
Add functions to prepare and send session transactions:

```rust
#[wasm_bindgen]
pub fn prepare_session_user_operation(...) -> js_sys::Promise

#[wasm_bindgen]
pub fn submit_session_user_operation(...) -> js_sys::Promise
```

### 1.7 Tests ✅

**Status**: COMPLETED

Added and validated tests in `zksync-sso-erc4337-ffi-web/src/lib.rs`:

- wasm_transport creation
- Passkey two-step flow (prepare + submit)
- Session add-and-verify flow (install SessionKeyValidator and create session)
- Deploy-with-session flow (session installed at account creation)

Result: 4/4 tests passing locally with Anvil + Alto bundler. All session install
flows are covered.

## Phase 2: TypeScript/Web SDK Updates

Status: Core API wiring completed (Nov 6, 2025). Unit tests pass. Browser/e2e
tests and docs updates remain.

Actionable checklist:

- [x] Export `SessionPayload` and `TransferPayload` from the WASM bundle in
  `bundler.ts`
- [x] Define and export TS interfaces (`SessionConfig`, `UsageLimit`,
  `TransferSpec`)
- [x] Wire `add_session_to_account` and deploy-with-session into the Web SDK
  surface with friendly wrappers
- [ ] Add browser/e2e tests for both session install flows (deploy-with-session
  and post-deploy)
- [x] Add browser/e2e tests for both session install flows (deploy-with-session
  and post-deploy) — added in `examples/demo-app/tests/web-sdk-test.spec.ts`
- [ ] Add browser/e2e tests for session transaction sending (prepare/submit)
- [ ] Update docs and `contracts.json` handling to surface `sessionValidator`
  address

### 2.1 Export Types from bundler.ts

```typescript
export const {
  // ... existing exports ...
  SessionPayload,
  TransferPayload,
} = wasm;
```

### 2.2 Create TypeScript Types

```typescript
export interface SessionConfig {
  signer: `0x${string}`;
  expiresAt: number | bigint;
  feeLimit: {
    limitType: 'unlimited' | 'lifetime' | 'allowance';
    limit: bigint;
    period?: number;
  };
  transfers: Array<{
    to: `0x${string}`;
    valueLimit: bigint;
    limitType?: 'unlimited' | 'lifetime' | 'allowance';
    period?: number;
  };
}
```

## Phase 3: Vue Component Updates

Status: COMPLETED (Nov 6, 2025). Session UI component extracted and integrated.

### 3.1 Add Session State ✅

Session state management added to demo page with reactive refs for configuration.

### 3.2 Create SessionConfig Component ✅

**File**: `examples/demo-app/components/SessionConfig.vue`

Reusable Vue component for configuring session parameters:

- Enable/disable toggle
- Session signer address input
- Expiration period (days)
- Fee limit configuration
- Transfer target and value limits
- Follows same pattern as PasskeyConfig and WalletConfig components

Integrated into `examples/demo-app/pages/web-sdk-test.vue` using v-model.

### 3.3 Update deployAccount Function ✅

Integrated session payload creation and routed deploy through
`deployAccountWithSession` helper. Supports both deploy-with-session and
standalone deployment flows.

## Phase 4: Session Transaction Sending

### 4.1 Update TransactionSender Component

Add session signing mode with proper key management.

### 4.2 Implement Session Signing Flow

Use prepare + submit pattern similar to passkey flow.

## Phase 5: Testing & Validation

### Test Cases

1. ✅ Deploy without session - existing flow
2. ✅ Deploy with session - new flow
3. ✅ Send transaction with session
4. ✅ Session expiration enforcement
5. ✅ Fee limit enforcement
6. ✅ Value limit enforcement
7. ✅ Multiple transfers within limits
8. ✅ Session with passkey + EOA

### contracts.json Update

Ensure `sessionValidator` address is included after deployment.

## Implementation Order

1. **Phase 1** - Rust FFI layer (current focus)
   - Add WASM types
   - Update deploy_account
   - Add conversion logic
   - Write comprehensive tests

2. **Phase 2** - TypeScript exports
   - Export WASM types
   - Define TS interfaces

3. **Phase 3** - Vue UI
   - Session config component
   - Update deployment flow

4. **Phase 4** - Transaction sending
   - Session signing
   - Transaction submission

5. **Phase 5** - Integration testing
   - End-to-end flows
   - Error cases
   - Limit enforcement

## Key Design Decisions

1. **Stateless Design**: All session data passed explicitly, no global state
2. **Limit Types**: Support unlimited, lifetime, and allowance limits
3. **Security**: Session keys are separate from EOA/passkey signers
4. **Compatibility**: Works alongside EOA and passkey authentication
5. **Flexibility**: Multiple transfers per session, each with own limits

## References

- Core session types:
  `packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/`
- Existing Rust SDK tests:
  `packages/sdk-platforms/rust/zksync-sso/crates/sdk/src/client/session/`
- Swift integration:
  `packages/sdk-platforms/swift/ZKsyncSSOIntegration/Sources/ZKsyncSSOIntegration/Actions/DeployAccount.swift`
