# Session Support for Web SDK - Implementation Plan

## Overview
Add session functionality to the Web SDK to enable gasless transactions within defined limits. Sessions allow users to authorize specific operations (transfers, contract calls) with limits on fees and values, signed by a separate session key.

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

## Phase 1: Rust/WASM FFI Layer Updates

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

### 1.4 Add Session Support to Core Deploy Function ⚠️
**Status**: BLOCKED - Requires core changes

The core `deploy_account` function in `zksync-sso-erc4337-core` does not currently support sessions during deployment. This needs to be added before we can complete the FFI implementation.

**Required Changes**:
1. Update `DeployAccountParams` in core to include `session_spec: Option<SessionSpec>`
2. Add session module installation logic in core deploy function
3. Encode session initialization data
4. Add session module to modules array

**Workaround**: For now, we can:
- Add the WASM bindings for session configuration
- Document the limitation
- Implement session addition post-deployment (similar to passkey addition)

### 1.5 Alternative: Add Session Post-Deployment 🔄
**Status**: RECOMMENDED APPROACH

Instead of adding session during deployment, we can add a function to install a session module after deployment:

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

### 1.6 Session Transaction Functions (TODO)
Add functions to prepare and send session transactions:

```rust
#[wasm_bindgen]
pub fn prepare_session_user_operation(...) -> js_sys::Promise

#[wasm_bindgen]
pub fn submit_session_user_operation(...) -> js_sys::Promise
```

### 1.7 Tests (TODO)
Add comprehensive tests for session functionality.

## Phase 2: TypeScript/Web SDK Updates

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
  }>;
}
```

## Phase 3: Vue Component Updates

### 3.1 Add Session State
```typescript
const sessionConfig = ref({
  enabled: false,
  signerPrivateKey: anvilPrivateKeys[2], // Different from deployer
  expiresAt: Math.floor(Date.now() / 1000) + 86400,
  feeLimit: "0.1",
  transfers: [{
    to: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    valueLimit: "0.1",
  }],
});
```

### 3.2 Create SessionConfig Component
UI component for configuring session parameters.

### 3.3 Update deployAccount Function
Integrate session payload creation and pass to WASM.

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

- Core session types: `packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/`
- Existing Rust SDK tests: `packages/sdk-platforms/rust/zksync-sso/crates/sdk/src/client/session/`
- Swift integration: `packages/sdk-platforms/swift/ZKsyncSSOIntegration/Sources/ZKsyncSSOIntegration/Actions/DeployAccount.swift`
