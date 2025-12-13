# Paymaster Implementation Plan

## Overview

Add General Paymaster support to the ZKsync SSO auth server and SDK, enabling
gas sponsorship for both regular transactions and session creation. Paymaster
sponsorship works transparently without requiring UI changes.

## Requirements

### 1. Paymaster Type

- **General Paymaster** (simple sponsorship model)
- Uses existing `createGeneralPaymaster()` handler from SDK
- TestPaymaster contract already exists at
  `packages/erc4337-contracts/src/test/TestPaymaster.sol`

### 2. UI Requirements

- **No extra display** — Paymaster works transparently in the background
- No fee comparison UI, no sponsorship badges
- Existing fee displays remain unchanged

### 3. Session Integration

- **No automatic session + paymaster coupling**
- Sessions and paymasters are independent features
- **Can create sessions WITH paymaster enabled** — session creation transaction
  can be sponsored
- Session fee limits remain independent of paymaster (track actual gas spent,
  not sponsored amounts)

### 4. Test Requirements

- **Use unfunded accounts** to prove paymaster sponsorship works
- Transactions that would fail without paymaster must succeed with it
- Fund test paymaster in `beforeAll()` hook, reuse single instance across tests
- Add negative test case: verify unfunded account fails without paymaster

### 5. Configuration Options

- **Support both**: paymaster address (simple) and handler function (advanced)
- Address-only → automatically use General Paymaster handler
- Custom handler → supports future ZyFi/custom integrations

## Architecture Changes

### SDK-4337 Connector

```typescript
// packages/sdk-4337/src/connector/index.ts
export interface ZksyncSsoConnectorParameters {
  // ... existing params
  paymaster?: Address | CustomPaymasterHandler;
}
```

### Client Creation Flow

```
zksyncSsoConnector({ paymaster: "0x..." })
  └→ getClient({ paymasterHandler })
     └→ createSmartAccountClient({ paymasterHandler })
        └→ bundler uses handler in getTransactionWithPaymasterData()
```

### Test Architecture

```
beforeAll:
  1. Deploy TestPaymaster contract
  2. Fund paymaster with 10 ETH from rich account
  3. Export paymaster address to test config

Tests:
  1. Create unfunded account (skip fundAccount() call)
  2. Verify balance = 0
  3. Send transaction WITH paymaster → Success
  4. Send transaction WITHOUT paymaster → Fail (insufficient funds)
  5. Create session WITH paymaster → Success
  6. Execute session transaction WITH paymaster → Success
```

## Implementation Tasks

### 1. SDK-4337 Connector Enhancement

- [ ] Add `paymaster?: Address | CustomPaymasterHandler` to connector parameters
- [ ] Convert address to handler using `createGeneralPaymaster()` if needed
- [ ] Pass handler through to client creation

**Files**:

- `packages/sdk-4337/src/connector/index.ts`
- `packages/sdk-4337/src/connector/types.ts`

### 2. Client Creation Threading

- [ ] Accept `paymasterHandler` parameter in `getClient()`
- [ ] Thread handler to `createSmartAccountClient()`
- [ ] Ensure handler is applied in bundler's transaction preparation

**Files**:

- `packages/sdk-4337/src/client-auth-server/index.ts`
- `packages/sdk-4337/src/client-auth-server/client.ts`

### 3. Demo-App Paymaster Deployment

- [ ] Create deployment script for TestPaymaster
- [ ] Export deployed address to `contracts-anvil.json`
- [ ] Add funding helper in test utilities

**Files**:

- `examples/demo-app/scripts/deploy-paymaster.ts` (new)
- `examples/demo-app/contracts-anvil.json`

### 4. Demo-App Connector Configuration

- [ ] Add paymaster address constant
- [ ] Create `zksyncConnectorWithPaymaster` instance
- [ ] Optionally add UI toggle for testing (button to connect with/without
      paymaster)

**Files**:

- `examples/demo-app/pages/index.vue`

### 5. E2E Test Suite: Paymaster Flows

- [ ] Add `beforeAll` hook to deploy and fund paymaster
- [ ] Helper: `createUnfundedAccount()` (skip funding step)
- [ ] Test: "Unfunded account transaction fails without paymaster"
- [ ] Test: "Unfunded account transaction succeeds with paymaster"
- [ ] Test: "Unfunded account creates session with paymaster"
- [ ] Test: "Session transaction uses paymaster successfully"

**Files**:

- `examples/demo-app/tests/create-account.spec.ts` (add new test suite)
- Or create new file: `examples/demo-app/tests/paymaster.spec.ts`

## Key Technical Details

### Paymaster Handler Flow

```typescript
// SDK already has infrastructure:
export type CustomPaymasterHandler = (
  args: CustomPaymasterParameters,
) => Promise<CustomPaymasterHandlerResponse>;

// Response includes:
interface CustomPaymasterHandlerResponse {
  paymaster: Address;
  paymasterInput: Hex;
  // Optional fee overrides:
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
}
```

### General Paymaster Handler

```typescript
// Already implemented in packages/sdk-4337/src/handlers/general.ts
export function createGeneralPaymaster(
  paymaster: Address,
): CustomPaymasterHandler {
  return async (_): Promise<CustomPaymasterHandlerResponse> => {
    return {
      paymaster,
      paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
    };
  };
}
```

### TestPaymaster Contract

```solidity
// Already exists at packages/erc4337-contracts/src/test/TestPaymaster.sol
contract TestPaymaster is IPaymaster {
  function validatePaymasterUserOp(...) external pure returns (...) {
    return ("", 0); // Always approves
  }

  function postOp(...) external {}

  receive() external payable {} // Can receive funds
}
```

## Test Validation Strategy

### Proof of Sponsorship

1. **Initial State**: Create account with balance = 0 ETH
2. **Without Paymaster**: Attempt 0.1 ETH transfer → Expect failure
   (insufficient funds)
3. **With Paymaster**: Same transfer → Expect success
4. **Post-Transaction**: Verify balance still ≈ 0 ETH (paymaster covered gas)

### Session + Paymaster Flow

1. **Create Session**: Unfunded account + paymaster → Session created
   successfully
2. **Execute Transaction**: Use session to send ETH → Success without passkey
3. **Balance Check**: Account balance remains at 0 (all gas sponsored)

### Error Detection

```typescript
// Check for insufficient funds error
catch (error) {
  const errorMessage = error.cause?.details || error.message;
  expect(errorMessage).toMatch(/insufficient funds|insufficient balance/i);
}
```

## Acceptance Criteria

- [ ] Connector accepts `paymaster: Address` parameter
- [ ] Connector accepts `paymaster: CustomPaymasterHandler` parameter
- [ ] Unfunded account (0 ETH) can execute transactions with paymaster
- [ ] Session creation works with paymaster on unfunded account
- [ ] Unfunded account transactions fail without paymaster (negative test)
- [ ] No breaking changes to existing flows (paymaster is optional)
- [ ] E2E tests pass consistently on local anvil
- [ ] No UI changes required (transparent operation)

## Future Enhancements (Out of Scope)

- [ ] UI indicator for paymaster sponsorship status
- [ ] Fee comparison display (with/without sponsorship)
- [ ] ZyFi paymaster integration for conditional sponsorship
- [ ] Paymaster selection UI (multiple paymasters)
- [ ] Session-level paymaster configuration (inherit from session)
- [ ] Analytics/tracking of sponsored transaction volume

## References

- **TestPaymaster Contract**:
  `packages/erc4337-contracts/src/test/TestPaymaster.sol`
- **Paymaster Handlers**: `packages/sdk-4337/src/handlers/`
- **Current E2E Tests**: `examples/demo-app/tests/create-account.spec.ts`
- **Demo App**: `examples/demo-app/pages/index.vue`
- **SDK Connector**: `packages/sdk-4337/src/connector/index.ts`

---

**Created**: December 12, 2025  
**Status**: Ready for Implementation
