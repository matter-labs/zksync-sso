# Guardian Recovery - Manual Testing Guide

This guide walks through manually testing the guardian recovery feature in the
ZKsync SSO auth-server.

## Prerequisites

1. **Local blockchain running**:

   ```bash
   # Start Anvil (in one terminal)
   cd packages/erc4337-contracts
   pnpm anvil
   ```

2. **Deploy contracts**:

   ```bash
   # Deploy contracts (in another terminal)
   cd packages/erc4337-contracts
   forge script script/Deploy.s.sol --broadcast \
     --rpc-url http://localhost:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```

   This will automatically update `packages/auth-server/stores/local-node.json`
   with the deployed addresses.

3. **Start bundler**:

   ```bash
   cd packages/erc4337-contracts
   pnpm bundler
   ```

4. **Start auth-server**:

   ```bash
   pnpm nx dev auth-server
   ```

## Test Scenarios

### Scenario 1: Basic Guardian Flow (SSO Account as Guardian)

This tests the most common case where a user wants another SSO account to be
their guardian.

#### Step 1: Create Primary Account

1. Navigate to `http://localhost:3000`
2. Click "Create Account" or "Sign Up"
3. Create a passkey when prompted (use name like "Primary User")
4. Complete account creation
5. **Save the account address** from the dashboard (e.g., `0xABC...123`)

#### Step 2: Create Guardian Account

1. Open a new **incognito/private browser window**
2. Navigate to `http://localhost:3000`
3. Create a new passkey (use name like "Guardian User")
4. Complete account creation
5. **Save this guardian account address** (e.g., `0xDEF...456`)

#### Step 3: Propose Guardian

1. Go back to the **first browser window** (Primary User)
2. Navigate to Dashboard → Settings
3. Find the "Guardian Recovery" section
4. Click "Add Guardian"
5. Enter the guardian address from Step 2 (`0xDEF...456`)
6. Click "Propose Guardian"
7. Wait for transaction to confirm
8. **Expected**: Guardian should appear in "Pending Guardians" list

#### Step 4: Confirm Guardian

1. Copy the confirmation link shown (or construct it):

   ```text
   http://localhost:3000/recovery/guardian/confirm-guardian?accountAddress=0xABC...123&guardianAddress=0xDEF...456
   ```

2. Open this link in the **incognito window** (Guardian User)
3. The page should detect it's an SSO account
4. Click "Confirm Guardian"
5. Wait for transaction to confirm
6. **Expected**: "Guardian Confirmed" success message

#### Step 5: Verify Guardian is Active

1. Go back to **Primary User window**
2. Refresh the Settings page
3. **Expected**: Guardian should now appear in "Active Guardians" list with
   status "Ready"

---

### Scenario 2: External Wallet as Guardian

This tests using a connected external wallet (like MetaMask) as a guardian.

#### Step 1: Create Account to Guard

1. Navigate to `http://localhost:3000`
2. Create an SSO account with passkey
3. **Save the account address**

#### Step 2: Propose External Wallet as Guardian

1. In Settings → Guardian Recovery
2. Click "Add Guardian"
3. Enter an external wallet address (e.g., from MetaMask)
4. Click "Propose Guardian"
5. Wait for confirmation

#### Step 3: Confirm with External Wallet

1. Copy the confirmation link
2. Open link in a new browser window
3. Click "Connect Wallet" button
4. Connect the wallet that matches the guardian address
5. **Expected**: "Wallet Connected" status appears
6. Click "Confirm Guardian"
7. Sign the transaction with the wallet
8. **Expected**: Guardian confirmed successfully

---

### Scenario 3: Guardian Status Checks

Test that the system correctly identifies guardian account types.

#### Check SSO Account

1. Create two SSO accounts (as in Scenario 1)
2. Navigate to confirmation page with guardian's address
3. **Expected**: Page should show "ZKsync SSO Account" badge
4. **Expected**: "Confirm Guardian" button appears automatically

#### Check Non-SSO Account

1. Create one SSO account
2. Use a regular Ethereum address as guardian
3. Navigate to confirmation page
4. **Expected**: Page shows "Standard Account"
5. **Expected**: "Connect Wallet" button appears
6. **Expected**: Must connect wallet before confirming

---

### Scenario 4: Remove Guardian

Test removing an active guardian.

#### Step 1: Set up Guardian

1. Follow Scenario 1 to add and confirm a guardian

#### Step 2: Remove Guardian

1. In Primary Account Settings
2. Find the active guardian in the list
3. Click "Remove Guardian" button
4. Confirm the transaction
5. **Expected**: Guardian removed from list
6. **Expected**: Guardian no longer appears in guardian list

---

### Scenario 5: Multiple Guardians

Test managing multiple guardians for one account.

1. Create one primary SSO account
2. Create two guardian SSO accounts (in separate incognito windows)
3. Propose both as guardians from primary account
4. Confirm both guardians (using their respective browser sessions)
5. **Expected**: Both guardians appear as "Active" in Settings
6. Remove one guardian
7. **Expected**: One guardian remains active

---

## Testing Edge Cases

### Invalid Guardian Address

1. Try to propose an invalid address (not checksummed, wrong format)
2. **Expected**: Validation error prevents submission

### Duplicate Guardian

1. Add and confirm a guardian
2. Try to add the same guardian address again
3. **Expected**: Should handle gracefully (check current behavior)

### Self as Guardian

1. Try to add your own account address as guardian
2. **Expected**: Should be prevented or handled appropriately

### Confirming Already Confirmed Guardian

1. Confirm a guardian
2. Try to access the confirmation link again
3. **Expected**: Show "Guardian already confirmed" message

---

## Checking Contract State

You can verify the guardian state on-chain using cast:

```bash
# Check if address is a guardian for an account
cast call <GUARDIAN_EXECUTOR_ADDRESS> \
  "guardianStatusFor(address,address)" \
  <ACCOUNT_ADDRESS> <GUARDIAN_ADDRESS> \
  --rpc-url http://localhost:8545

# Get all guardians for an account
cast call <GUARDIAN_EXECUTOR_ADDRESS> \
  "getGuardians(address)" \
  <ACCOUNT_ADDRESS> \
  --rpc-url http://localhost:8545
```

---

## Troubleshooting

### Guardian Executor Address Not Set

- **Symptom**: Error "Recovery contract address not configured"
- **Fix**: Ensure `local-node.json` has valid `guardianExecutor` and `recovery`
  addresses
- **Check**: Both should point to the GuardianExecutor contract address

### Transactions Failing

- **Check**: Anvil is running and bundler is connected
- **Check**: Account has sufficient ETH/balance
- **Check**: Paymaster is configured (if using one)

### SSO Account Not Detected

- **Symptom**: External wallet required even for SSO accounts
- **Check**: `useIsSsoAccount` composable is working
- **Check**: Account was created with the current auth-server instance

### Cannot Connect to Confirmation Page

- **Symptom**: 404 on `/recovery/guardian/...` routes
- **Fix**: Ensure `pages/recovery` folder exists (not `recovery.disabled`)
- **Check**: Run `pnpm nx dev auth-server` to rebuild routes

---

## Expected User Flow Summary

```text
Primary User                          Guardian User
     |                                      |
     | 1. Propose Guardian                  |
     |------------------------------------->|
     |                                      |
     |    2. Share confirmation link        |
     |------------------------------------->|
     |                                      |
     |                              3. Open link
     |                              4. Confirm Guardian
     |                                      |
     |<-------------------------------------|
     | 5. Guardian now active               |
     |                                      |
```

The key insight is that:

1. **Primary user** initiates by proposing the guardian
2. **Guardian** must actively confirm (sign transaction) to accept the role
3. Both actions require on-chain transactions
4. Once confirmed, guardian can initiate recovery if primary user loses access

---

## Future Testing: Recovery Execution

> **Note**: Full recovery testing (initiating and executing recovery) requires
> additional implementation. This guide covers the guardian management portion
> only.

Recovery execution would involve:

1. Guardian initiates recovery for a lost account
2. Wait for timelock period
3. Execute recovery to change account ownership
4. These flows are planned for future implementation
