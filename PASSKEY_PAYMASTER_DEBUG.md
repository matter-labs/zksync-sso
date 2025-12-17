# Passkey Paymaster Transaction Debugging Guide

## Problem Statement

The **"Send with Paymaster (Passkey)"** button fails with AA23 error (signature
validation failure) during gas estimation, while:

- ✅ **EOA + Paymaster** works correctly (confirmed: balance changed 0.1 → 0.09
  ETH)
- ✅ **Passkey without Paymaster** works correctly
- ✅ **Rust integration test** with passkey + paymaster passes all assertions
- ❌ **Vue button** with passkey + paymaster consistently fails with AA23

## Working Reference: Rust Test

**File:**
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/paymaster/paymaster_test.rs`

**Function:** `test_passkey_eth_send_with_paymaster_no_fees()` (lines 232-420)

**Key characteristics:**

- Deploys account with EOA signer first
- Funds account from rich wallet
- Installs WebAuthn validator module
- Adds passkey with `origin_domain: "https://example.com"`
- Creates paymaster with:
  `PaymasterParams::default_paymaster(paymaster_address)`
  - This sets `data: Bytes::default()` (**empty bytes**, not a flow type)
- Calls `passkey_send_transaction()` which internally uses `send_user_op()`
- Validates: sender only loses transfer amount (no gas fees paid by sender)
- **Test assertion:** `sender_delta == amount` (only 1000 wei transferred, no
  gas)

## Failing Implementation: Vue Button

**File:** `examples/demo-app/pages/web-sdk-test.vue`

**Function:** `sendWithPasskeyPaymaster()` (lines 1449-1559)

### Initial Issue Found & Fixed

Changed paymaster data parameter from `"0x05"` to `null`:

```typescript
// BEFORE (changed from this):
const paymaster = new PaymasterParams(paymasterAddress, "0x05", null, null);

// AFTER (changed to this):
const paymaster = new PaymasterParams(paymasterAddress, null, null, null);
```

Also removed unused `paymasterFlow` field from `passkeyConfig`:

```typescript
// Removed this line:
paymasterFlow: "0x05",
```

## Error Details

### Error Message

```text
RPC request failed:
Error: Execution error: execution reverted: custom error 0x65c8fd4d:
AA23 reverted
```

### Error Code Meaning

- **AA23** = Account's `validateAccountSignature()` reverted during validation
- This is **NOT** a paymaster error (which would be AA33)
- Occurs during gas estimation (`eth_call`) with state overrides
- Not during actual transaction submission

### Root Cause Hypothesis

The AA23 error during gas estimation suggests one of these:

1. **Account state issue**: The passkey hasn't been registered properly or
   WebAuthn validator isn't installed as a module
2. **Stub signature mismatch**: The stub signature used during gas estimation
   doesn't match what the validator expects
3. **Nonce state issue**: If account already used the passkey for other
   transactions, nonce validation might fail
4. **State override problem**: The state override in the bundler's gas
   estimation call doesn't properly simulate the account state

## Flow Comparison: Rust vs Vue

### Rust Flow (Working)

```text
1. Deploy account (with EOA signer)
2. Fund account
3. Add WebAuthn validator module to account
4. Add passkey to account
5. Send transaction:
   - Prepare UserOperation with stub signature
   - Get hash from EntryPoint
   - Sign with passkey
   - Submit with real signature
6. Bundler processes:
   - Gas estimation (simulateValidation)
   - User operation submission
```

**Paymaster fields:**

- `address`: Set
- `data`: Empty bytes (Bytes::default())
- `verification_gas_limit`: None
- `post_op_gas_limit`: None
- Calculated in `prepare_passkey_user_operation`:
  - `verification_gas_limit`: 3M (higher for paymaster)
  - `post_op_gas_limit`: 1M (auto-set default)

### Vue Flow (Failing)

```text
1. Deploy account (button: "Deploy Account")
2. Fund account (button: "Fund Smart Account")
3. Load validator address (automatic on load or button:
   "Load WebAuthn Validator")
4. Register passkey (button: "Register Passkey")
5. Fund paymaster (button: "Fund Paymaster")
6. Send transaction (button: "Send with Paymaster (Passkey)"):
   - Load contracts and defaults
   - Build SendTransactionConfig
   - Create PaymasterParams
   - Call prepare_passkey_user_operation
   - Sign with passkey
   - Submit signed UserOperation
```

**Paymaster fields:**

- `address`: Set from `paymasterTx.value.paymaster || contracts.testPaymaster`
- `data`: null (converted to empty in normalize_paymaster_params)
- `verification_gas_limit`: null
- `post_op_gas_limit`: null

## Code Changes Made

### Change 1: PaymasterParams Data Field

**File:** `examples/demo-app/pages/web-sdk-test.vue` (line 1493)

```typescript
// Changed from:
const paymaster = new PaymasterParams(paymasterAddress, "0x05", null, null);

// To:
const paymaster = new PaymasterParams(paymasterAddress, null, null, null);
```

**Reason:** Match the Rust test which uses
`PaymasterParams::default_paymaster()` (empty data)

### Change 2: Remove Unused paymasterFlow

**File:** `examples/demo-app/pages/web-sdk-test.vue` (lines 711-719)

```typescript
// Removed:
paymasterFlow: "0x05",
```

**Reason:** This field was not being used; the paymaster data is passed directly
in PaymasterParams constructor

## Testing Status

### Setup Checklist

- [ ] Deploy Account
- [ ] Fund Account (0.1 ETH)
- [ ] Load WebAuthn Validator Address
- [ ] Register Passkey (critical - installs WebAuthn validator module)
- [ ] Fund Paymaster (1 ETH)
- [ ] Verify Paymaster Balance shows > 0

### Current Behavior

- ✅ Steps 1-5 complete successfully
- ✅ Console logs show addresses are loaded
- ❌ "Send with Paymaster (Passkey)" fails with AA23
- Error occurs during bundler's gas estimation (eth_call simulateValidation)

## Areas to Investigate

### 1. Module Installation Verification

Check if WebAuthn validator is actually installed as a module:

```bash
# In browser console, after "Register Passkey" succeeds:
# Check account for WebAuthn module at contracts.webauthnValidator
# Call: isModuleInstalled(account, webauthnValidator)
```

### 2. Stub Signature Format

The stub signature used for gas estimation might not match the expected format.

**File:**
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-ffi-web/src/lib.rs`

- `stub_signature_passkey_core()` generates stub signature (line ~1328)
- Format must match WebAuthnValidator's expected signature structure

### 3. Gas Estimation State Override

The bundler's gas estimation includes account state overrides:

```text
state_diff: {
  [storage_slot]: 0x00...010559c5
}
```

**Questions:**

- Is this state slot correct for the account being estimated?
- Does it represent the validator/passkey being installed?
- Is the state override sufficient for validation?

### 4. Nonce State

If the account has already made transactions:

```typescript
// Check nonce before sending
const nonce = await getAccountNonce(accountAddress, entryPoint);
console.log("Current nonce:", nonce);
```

### 5. Signature Validation Logic

The WebAuthnValidator might have specific requirements:

- Expected signature format (from `stub_signature_passkey_core`)
- Validation might fail if account state isn't properly simulated
- Validator code location:
  `packages/erc4337-contracts/contracts/validators/ WebAuthnValidator.sol`

## WASM FFI Flow Details

**File:**
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/ zksync-sso-erc4337-ffi-web/src/lib.rs`

### Paymaster Normalization (lines 340-379)

```rust
fn normalize_paymaster_params(pm: Option<PaymasterParams>) -> Option<PaymasterParamsCore> {
    // ...
    let data_bytes = if pm.data().is_empty() {
        Bytes::default()
    } else {
        match hex::decode(pm.data().trim_start_matches("0x")) {
            Ok(bytes) => Bytes::from(bytes),
            Err(_) => Bytes::default(),  // ⚠️ SILENT FAILURE
        }
    };
    // ...
}
```

**Note:** Hex decoding failures silently fall back to empty bytes. This is why
"0x05" vs null shouldn't matter, but might indicate other issues.

### UserOperation Preparation (lines 1186-1500)

```text
1. Parse addresses and values
2. Create transport and provider
3. Normalize paymaster params
4. Encode call data
5. Get nonce with key
6. Create stub signature
7. Build AlloyPackedUserOperation with fixed gas:
   - call_gas_limit: 2M
   - verification_gas_limit: 3M (if paymaster)
   - pre_verification_gas: 1M
8. Pack gas limits and fees
9. Build EntryPoint::PackedUserOperation
10. Get hash from EntryPoint (on-chain call)
11. Return hash + prepared UserOperation (JSON)
```

## Debugging Commands

### Browser Console

```typescript
// After Register Passkey succeeds:
const deploymentResult = ... // from Vue component state
const contracts = await loadContracts()
const publicClient = await createPublicClient(contracts)

// Check if WebAuthn validator is installed
const isInstalled = await publicClient.call({
  account: deploymentResult.address,
  to: contracts.entryPoint,
  // Call isModuleInstalled()
})

// Check account nonce
const nonce = await publicClient.call({
  account: deploymentResult.address,
  to: contracts.entryPoint,
  // Call getNonce()
})

// Get current account balance
const balance = await publicClient.getBalance({
  address: deploymentResult.address
})
```

### Anvil Logs

Watch the Anvil terminal (Terminal "a") during "Send with Paymaster (Passkey)":

- Should show `eth_call` with state overrides
- Look for the account address in the calls
- Check if simulateValidation succeeds or fails
- Note the exact error message

### Network Tab

Check the bundler RPC calls in browser DevTools:

1. `bundler_estimateUserOperationGas` - This is where AA23 occurs
2. Look at the full UserOperation being sent:
   - Verify paymaster address is set
   - Verify paymaster data field (should be empty or minimal)
   - Verify signature (should be stub signature for gas estimation)

## Hypotheses to Test

### Hypothesis 1: Module Not Installed

After clicking "Register Passkey", the WebAuthn validator should be installed.

**Test:**

```typescript
// Check IsModuleInstalledParams
const isInstalled = await isModuleInstalled({
  module: Module::webauthn_validator(webauthnValidator),
  account: accountAddress,
  provider
})
console.log("WebAuthn validator installed:", isInstalled)
```

**Expected:** `true`

### Hypothesis 2: Passkey Not Registered

The passkey credentials might not be stored properly.

**Test:** Check the account's storage for passkey after "Register Passkey":

```typescript
// Get account's passkey registry state
const passkeyData = await publicClient.getStorageAt({
  address: accountAddress,
  slot: ... // depends on storage layout
})
```

### Hypothesis 3: Stub Signature Issues

The stub signature format might be incorrect when paymaster is present.

**Test:** Compare stub signatures from:

- Passkey without paymaster (working) - see logs
- Passkey with paymaster (failing) - see logs
- Should be identical

### Hypothesis 4: Gas Limits Too Low

The verification gas limit might not be enough for paymaster + account
validation.

**Test:** Increase gas limits in `prepare_passkey_user_operation`:

```rust
let verification_gas_limit = if paymaster_params.is_some() {
    U256::from(5_000_000u64) // Try higher
} else {
    U256::from(2_000_000u64)
};
```

### Hypothesis 5: Paymaster Data Causing State Changes

Empty paymaster data might affect validation in unexpected ways.

**Test:** Try explicit empty bytes vs null:

```typescript
const paymaster = new PaymasterParams(paymasterAddress, "", null, null);
// vs
const paymaster = new PaymasterParams(paymasterAddress, null, null, null);
```

## Files to Review

### Core Logic

- [lib.rs](packages/sdk-platforms/rust/zksync-sso-erc4337/crates/
  zksync-sso-erc4337-ffi-web/src/lib.rs) - `prepare_passkey_user_operation()`,
  `normalize_paymaster_params()`
- [send.rs](packages/sdk-platforms/rust/zksync-sso-erc4337/crates/
  zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/ send.rs) -
  `send_user_op()`
- [passkey.rs](packages/sdk-platforms/rust/zksync-sso-erc4337/crates/
  zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/
  send/passkey.rs) - `passkey_send_transaction()`

### Smart Contracts

- [WebAuthnValidator.sol](packages/erc4337-contracts/contracts/validators/
  WebAuthnValidator.sol) - Signature validation logic
- [TestPaymaster.sol](packages/erc4337-contracts/contracts/paymaster/
  TestPaymaster.sol) - Paymaster validation

### Vue Components

- [web-sdk-test.vue](examples/demo-app/pages/web-sdk-test.vue) -
  `sendWithPasskeyPaymaster()` (line 1449)
- [TransactionSender.vue](examples/demo-app/components/ TransactionSender.vue) -
  Paymaster checkbox and submission

## Next Steps for Debugging

1. **Enable verbose logging:**

   - Add `console.log()` statements in `prepare_passkey_user_operation()` (Rust
     WASM)
   - Log the prepared UserOperation before signing
   - Log the final paymaster field values

2. **Test with reduced scope:**

   - Try sending without paymaster to confirm passkey works
   - Try EOA with paymaster to confirm paymaster works
   - Combine both to identify interaction issue

3. **Compare UserOperations:**

   - Print full UserOp from working EOA + paymaster
   - Print full UserOp from failing Passkey + paymaster
   - Diff the two to find structural differences

4. **Check state machine:**

   - Ensure "Register Passkey" actually installed the module
   - Verify account state includes registered passkey
   - Confirm nonce is correct before sending

5. **Bundler debugging:**
   - Add logging to bundler gas estimation
   - See exactly which validation step fails
   - Get the full revert reason instead of just "AA23"

## Collaboration Points

**For other developers:**

- The fix has been applied but issue persists
- Need help understanding the stub signature format
- Need help interpreting Anvil/bundler error messages
- May need to add logging to Rust WASM code

**Current Status:**

- Code changes: ✅ Applied
- Testing: ❌ Still fails with AA23
- Next: Deep dive into stub signature and state overrides
- May need to add logging to Rust WASM code

**Current Status:**

- Code changes: ✅ Applied
- Testing: ❌ Still fails with same AA23 error
- Root cause: ❓ Unknown - likely account state or signature validation

---

**Last Updated:** December 17, 2025 **File:**
`/home/colinbellmore/Documents/zksync-sso/PASSKEY_PAYMASTER_DEBUG.md`
