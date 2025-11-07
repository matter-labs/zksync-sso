# Session Support for Web SDK - Implementation Plan

## Status Summary (Updated Nov 6, 2025)

- ✅ **Phase 1**: Rust/WASM FFI Layer - COMPLETED
  - Session types, deploy-with-session, post-deploy installation
  - FFI tests: 4/4 passing
- ✅ **Phase 2**: TypeScript/Web SDK Updates - COMPLETED
  - Types, helpers, unit tests, e2e tests for installation
  - Unit tests: 8/8 passing
  - E2E tests: 4/4 passing (including session install flows)
- ✅ **Phase 3**: Vue Component Updates - COMPLETED
  - SessionConfig component created and integrated
  - Deploy and post-deploy UI functional
- ⏳ **Phase 4**: Session Transaction Sending - NEXT PRIORITY
  - Need: FFI functions, TS wrappers, UI updates, e2e tests
- ⏳ **Phase 5**: Testing & Validation - PARTIAL
  - Installation flows validated ✅
  - Transaction sending and limit enforcement pending Phase 4

**Recent Achievements**:

- Both session install flows (deploy-with-session and post-deploy) implemented
  and tested
- End-to-end Playwright tests passing for session installation
- Reusable SessionConfig Vue component extracted
- Documentation updated for contracts.json configuration
- All core session infrastructure in place for Web SDK

**Next Steps**:

Phase 4 implementation to enable sending transactions signed by session keys,
with proper limit enforcement and validation.

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

**Status**: COMPLETED (Nov 6, 2025)

Core API wiring, TypeScript interfaces, session helpers, unit tests, and e2e
tests for session installation flows all completed.

Actionable checklist:

- [x] Export `SessionPayload` and `TransferPayload` from the WASM bundle in
  `bundler.ts`
- [x] Define and export TS interfaces (`SessionConfig`, `UsageLimit`,
  `TransferSpec`)
- [x] Wire `add_session_to_account` and deploy-with-session into the Web SDK
  surface with friendly wrappers
- [x] Add browser/e2e tests for both session install flows (deploy-with-session
  and post-deploy) — added in `examples/demo-app/tests/web-sdk-test.spec.ts`
- [x] Update docs and `contracts.json` handling to surface `sessionValidator`
  address — documented in `examples/demo-app/README.md`
- [ ] Add browser/e2e tests for session transaction sending (prepare/submit) —
  deferred to Phase 4

### 2.1 Export Types from bundler.ts ✅

Exported WASM types and helper functions:

```typescript
export const {
  // ... existing exports ...
  SessionPayload,
  TransferPayload,
  add_session_to_account,
} = wasm;

// Re-export session helpers
export { toSessionPayload, deployAccountWithSession, addSessionToAccount } from './session';
```

### 2.2 Create TypeScript Types ✅

**File**: `packages/sdk-platforms/web/src/types.ts`

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

### 2.3 Session Helper Functions ✅

**File**: `packages/sdk-platforms/web/src/session.ts`

Implemented helper functions for ergonomic session usage:

- `toSessionPayload(spec: SessionConfig)` - Converts TS config to WASM payload
- `deployAccountWithSession(...)` - Deploy account with optional session
- `addSessionToAccount(...)` - Install session on existing account

### 2.4 Unit Tests ✅

**File**: `packages/sdk-platforms/web/src/session.test.ts`

Unit tests covering session payload conversion and wrapper invocations.
Result: All tests passing (8 tests total in web SDK).

### 2.5 E2E Tests ✅

**File**: `examples/demo-app/tests/web-sdk-test.spec.ts`

Browser e2e tests covering:

- Deploy with session at deploy time
- Install session post-deploy

Result: All tests passing (4/4 in ERC-4337 test suite).

### 2.6 Documentation ✅

**File**: `examples/demo-app/README.md`

Documented `contracts.json` configuration including `sessionValidator` field
required for session flows.

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

**Status**: PENDING

This phase will add the ability to send transactions signed by session keys.

### 4.1 Add Session Transaction Functions to FFI

Add Rust/WASM functions for session-signed transactions:

```rust
#[wasm_bindgen]
pub fn prepare_session_user_operation(...) -> js_sys::Promise

#[wasm_bindgen]
pub fn submit_session_user_operation(...) -> js_sys::Promise
```

### 4.2 Create TypeScript Wrappers

Add convenience wrappers in `packages/sdk-platforms/web/src/session.ts`:

- `prepareSessionTransaction(...)`
- `submitSessionTransaction(...)`
- `sendTransactionWithSession(...)` - High-level wrapper combining both

### 4.3 Update TransactionSender Component

**File**: `examples/demo-app/components/TransactionSender.vue`

Add session signing mode:

- Radio button option for "Session Key" alongside EOA and Passkey
- Session key private key input (or load from config)
- UI to show session limits and remaining allowances

### 4.4 Add E2E Tests

**File**: `examples/demo-app/tests/web-sdk-test.spec.ts`

Add test case:

- Deploy account with session
- Fund account
- Send transaction using session key signature
- Verify transaction succeeds and limits are enforced

## Phase 5: Testing & Validation

**Status**: PARTIAL - Session installation flows validated; transaction sending
pending Phase 4

### Test Cases

1. ✅ Deploy without session - existing flow (e2e passing)
2. ✅ Deploy with session - new flow (e2e passing)
3. ✅ Install session post-deploy (e2e passing)
4. ⏳ Send transaction with session - pending Phase 4
5. ⏳ Session expiration enforcement - pending Phase 4
6. ⏳ Fee limit enforcement - pending Phase 4
7. ⏳ Value limit enforcement - pending Phase 4
8. ⏳ Multiple transfers within limits - pending Phase 4
9. ⏳ Session with passkey + EOA - pending Phase 4

### contracts.json Update ✅

Documented in `examples/demo-app/README.md`. The `sessionValidator` field is now
included in deployment scripts and properly loaded by the demo app.

## Implementation Summary

### Completed Work (Phases 1-3)

#### Phase 1: Rust/WASM FFI Layer

- ✅ Session payload types (SessionPayload, TransferPayload)
- ✅ DeployAccountConfig extended with session_validator_address
- ✅ deploy_account supports optional session payload
- ✅ add_session_to_account for post-deployment installation
- ✅ FFI tests covering both installation flows (4/4 passing)

#### Phase 2: TypeScript/Web SDK

- ✅ Exported WASM session types to bundler
- ✅ TypeScript interfaces (SessionConfig, UsageLimit, TransferSpec, LimitType)
- ✅ Helper functions (toSessionPayload, deployAccountWithSession,
  addSessionToAccount)
- ✅ Unit tests for session conversion and wrappers (8/8 passing)
- ✅ E2E tests for both installation flows (4/4 passing)
- ✅ Documentation of contracts.json configuration

#### Phase 3: Vue Components

- ✅ SessionConfig.vue component with v-model support
- ✅ Session state management in demo page
- ✅ Integration with deploy flow
- ✅ Post-deploy session installation UI

### Remaining Work (Phases 4-5)

#### Phase 4: Session Transaction Sending

- ⏳ FFI functions for session-signed transactions
- ⏳ TypeScript wrappers for session transaction flow
- ⏳ TransactionSender component updates for session key option
- ⏳ E2E tests for session-signed transactions

#### Phase 5: Additional Testing

- ⏳ Session limit enforcement tests
- ⏳ Session expiration tests
- ⏳ Multi-validator scenarios (session + passkey + EOA)

### Files Modified/Created

**Rust/WASM**:

- `packages/sdk-platforms/rust/zksync-sso-erc4337-ffi-web/src/lib.rs`

**TypeScript/Web SDK**:

- `packages/sdk-platforms/web/src/bundler.ts`
- `packages/sdk-platforms/web/src/types.ts`
- `packages/sdk-platforms/web/src/session.ts` (new)
- `packages/sdk-platforms/web/src/session.test.ts` (new)
- `packages/sdk-platforms/web/src/index.ts`

**Demo App**:

- `examples/demo-app/components/SessionConfig.vue` (new)
- `examples/demo-app/pages/web-sdk-test.vue`
- `examples/demo-app/tests/web-sdk-test.spec.ts`
- `examples/demo-app/README.md`

**Documentation**:

- `packages/sdk-platforms/web/SESSION_WEB_SDK_PLAN.md` (this file)

## Implementation Order

1. ✅ **Phase 1** - Rust FFI layer *(COMPLETED)*
   - Added WASM types (SessionPayload, TransferPayload)
   - Updated deploy_account with optional session support
   - Added conversion logic with robust error handling
   - Comprehensive tests (4/4 passing)

2. ✅ **Phase 2** - TypeScript exports *(COMPLETED)*
   - Exported WASM types to bundler
   - Defined TS interfaces for ergonomic usage
   - Created helper wrappers (toSessionPayload, deployAccountWithSession,
     addSessionToAccount)
   - Unit and e2e tests passing

3. ✅ **Phase 3** - Vue UI *(COMPLETED)*
   - Created reusable SessionConfig component
   - Updated deployment flow to support sessions
   - Added post-deploy session installation UI

4. ⏳ **Phase 4** - Transaction sending *(NEXT)*
   - Add session signing functions to FFI
   - TypeScript wrappers for session transactions
   - Update TransactionSender component for session key option
   - E2E tests for session-signed transactions

5. ⏳ **Phase 5** - Integration testing *(PENDING)*
   - Session limit enforcement tests
   - Session expiration tests
   - Multi-validator scenario tests

## Key Design Decisions

1. **Stateless Design**: All session data passed explicitly, no global state
2. **Limit Types**: Support unlimited, lifetime, and allowance limits
3. **Security**: Session keys are separate from EOA/passkey signers
4. **Compatibility**: Works alongside EOA and passkey authentication
5. **Flexibility**: Multiple transfers per session, each with own limits
6. **Dual Installation Paths**: Support both deploy-with-session and
   post-deployment installation
7. **Component Reusability**: SessionConfig component follows established
   patterns (PasskeyConfig, WalletConfig)

## References

- Core session types:
  `packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/`
- Existing Rust SDK tests:
  `packages/sdk-platforms/rust/zksync-sso/crates/sdk/src/client/session/`
- Swift integration:
  `packages/sdk-platforms/swift/ZKsyncSSOIntegration/Sources/ZKsyncSSOIntegration/Actions/DeployAccount.swift`
- Web SDK session helpers:
  `packages/sdk-platforms/web/src/session.ts`
- SessionConfig component:
  `examples/demo-app/components/SessionConfig.vue`
