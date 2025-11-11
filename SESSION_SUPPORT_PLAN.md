# Session Support Implementation Plan for SDK-4337

## Overview

This plan outlines the implementation of session-based smart account support in
`sdk-4337` (ERC-4337 compatible) mirroring the existing architecture in `sdk`
(ZKsync native). The key challenge is ensuring WASM compatibility, particularly
avoiding time-related functions that don't work in browser environments.

## Architecture Context

### Existing Implementations

1. **sdk (ZKsync Native)**: Full session support with
   `createZksyncSessionClient`
2. **sdk-4337 (ERC-4337)**: Currently only supports EOA and Passkey signers, no
   sessions
3. **Rust Core**: Session logic exists in `zksync-sso-erc4337-core` with
   time-dependent validation
4. **Web SDK (WASM)**: Exposes Rust functions but currently limited by time
   encoding issues

## Phase 1: Rust Core - WASM-Compatible Session Functions

### Goal

Create WASM-friendly session functions that avoid time-based validation during
signing (defer to on-chain validation).

### Status: ✅ In Progress (Tasks 1.1 and 1.4 Complete)

### Tasks

#### 1.1: Create Session Signature Generation (No Time Validation) ✅ COMPLETE

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/`

**✅ Created**: `signature_wasm.rs`

**Implemented Functions**:

```rust
// WASM-safe period ID calculation (no std::time calls)
pub fn get_period_id_no_validation(
    limit: &UsageLimit,
    current_timestamp: Option<u64>,
) -> Uint<48, 1>

// Session signature generation WITHOUT timestamp validation
pub fn session_signature_no_validation(
    private_key_hex: &str,
    session_validator: Address,
    session_spec: &SessionSpec,
    hash: FixedBytes<32>,
    current_timestamp: Option<u64>,
) -> eyre::Result<Bytes>
```

**Why**: The existing `signature.rs` may call time functions through
`get_period_id`. We need a version that:

- ✅ Accepts an optional timestamp parameter (caller provides it)
- ✅ Computes period_ids without time validation
- ✅ Defers all validation to on-chain execution

**Test Coverage** ✅:

```rust
#[test]
fn test_get_period_id_no_validation_with_timestamp()
#[test]
fn test_get_period_id_no_validation_without_timestamp()
#[test]
fn test_get_period_id_no_validation_lifetime_limit()
#[test]
fn test_session_signature_no_validation_generates_valid_signature()
#[test]
fn test_session_signature_no_validation_with_timestamp()
#[test]
fn test_session_signature_matches_expected_encoding_format()
```

#### 1.2: Create Deploy With Session Support

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/deploy.rs`

**Modify**: Add optional session module installation during deployment

```rust
pub struct DeployAccountParams<P: Provider + Send + Sync + Clone> {
    // ... existing fields ...

    // NEW: Optional session configuration
    pub session_config: Option<SessionDeployConfig>,
}

pub struct SessionDeployConfig {
    pub session_validator: Address,
    pub initial_sessions: Vec<SessionSpec>, // Sessions to install during deployment
}
```

**Why**: Currently deployment only supports EOA and WebAuthn validators. We need
to:

1. Install session validator module during deployment
2. Optionally create initial sessions in the same transaction
3. Return session validator address in deployment result

**Test Coverage**:

```rust
#[tokio::test]
async fn test_deploy_account_with_session_validator()
#[tokio::test]
async fn test_deploy_account_with_session_and_initial_session()
#[tokio::test]
async fn test_deploy_account_with_eoa_and_session_validators()
```

#### 1.3: Keyed Nonce Utilities

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/send.rs`

**Already exists**: `keyed_nonce()` function ✅

- Review existing implementation
- Ensure it's exposed for FFI

**Test Coverage** (already exists):

```rust
#[test]
async fn test_keyed_nonce()
#[test]
async fn test_send_transaction_session()
```

#### 1.4: Encode Session UserOperation ✅ COMPLETE

**✅ Created**: `encode.rs` in session directory

**Implemented Functions**:

```rust
// Encode UserOperation call data for session transaction
pub fn encode_session_user_operation(
    target: Address,
    value: U256,
    data: Bytes,
) -> Bytes

// Generate stub signature for gas estimation
pub fn generate_session_stub_signature(
    session_validator: Address,
    session_spec: &SessionSpec,
    current_timestamp: Option<u64>,
) -> Bytes
```

**Why**: Similar to `encode_execute_call_data` for EOA, we need session-specific
encoding.

**Test Coverage** ✅:

```rust
#[test]
fn test_encode_session_user_operation()
#[test]
fn test_encode_session_user_operation_empty_data()
#[test]
fn test_generate_session_stub_signature_has_correct_format()
#[test]
fn test_generate_session_stub_signature_with_timestamp()
#[test]
fn test_generate_session_stub_signature_size_consistency()
#[test]
fn test_stub_signature_matches_real_signature_size()
```

---

## Phase 2: FFI/WASM Bindings

### Goal

Expose Rust session functions to JavaScript/TypeScript via WASM.

### Tasks

#### 2.1: Add Session Functions to FFI

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-ffi-web/src/lib.rs`

**New Exports**:

```rust
#[wasm_bindgen]
pub fn encode_session_signature(
    private_key_hex: &str,
    session_validator: &str,
    session_spec: JsValue, // SessionSpec as JSON
    hash: &str,
) -> Result<String, JsValue>

#[wasm_bindgen]
pub fn generate_session_stub_signature(
    session_validator: &str,
    session_spec: JsValue, // SessionSpec as JSON
) -> Result<String, JsValue>

#[wasm_bindgen]
pub fn compute_keyed_nonce(
    session_signer_address: &str,
) -> Result<String, JsValue>

#[wasm_bindgen]
pub fn encode_add_session_call_data(
    session_validator: &str,
    session_spec: JsValue, // SessionSpec as JSON
) -> Result<String, JsValue>
```

**Why**: These are the building blocks needed by the TypeScript SDK.

**Test Coverage**: WASM tests in Node.js

```typescript
// packages/sdk-platforms/web/tests/session.test.ts
test("encode_session_signature produces valid signature");
test("generate_session_stub_signature matches expected format");
test("compute_keyed_nonce produces correct value");
```

#### 2.2: Update bundler.ts Exports

**Location**: `packages/sdk-platforms/web/src/bundler.ts`

**Add**:

```typescript
export const {
  // ... existing exports ...

  // ===== SESSION FUNCTIONS =====
  encode_session_signature, // Sign UserOp with session key
  generate_session_stub_signature, // Generate stub sig for gas estimation
  compute_keyed_nonce, // Calculate session-keyed nonce
  encode_add_session_call_data, // Encode createSession call
} = wasm;
```

**Why**: Make session functions available to sdk-4337.

#### 2.3: Add Session Types

**Location**: `packages/sdk-platforms/web/src/types.ts`

**Add**:

```typescript
export type SessionSpec = {
  signer: string; // Address
  expiresAt: string; // U48 as string
  feeLimit: UsageLimit;
  callPolicies: CallPolicy[];
  transferPolicies: TransferPolicy[];
};

export type UsageLimit = {
  limitType: number; // 0=Unlimited, 1=Lifetime, 2=Allowance
  limit: string; // U256 as string
  period: string; // U256 as string
};

export type TransferPolicy = {
  target: string;
  maxValuePerUse: string;
  valueLimit: UsageLimit;
};

export type CallPolicy = {
  target: string;
  selector: string;
  maxValuePerUse: string;
  valueLimit: UsageLimit;
  constraints: Constraint[];
};

export type Constraint = {
  index: string;
  condition: number;
  refValue: string;
  limit: UsageLimit;
};
```

---

## Phase 3: SDK-4337 TypeScript Implementation

### Goal

Create session client and account abstractions in sdk-4337 mirroring the sdk
implementation.

### Tasks

#### 3.1: Session Account Implementation

**Location**: `packages/sdk-4337/src/client/session/account.ts` (NEW)

**Implementation**:

```typescript
import type { Address, Hash, Hex, PublicClient, Transport, Chain } from "viem";
import { toSmartAccount, type ToSmartAccountReturnType } from "viem/account-abstraction";
import {
  encode_session_signature,
  generate_session_stub_signature,
  compute_keyed_nonce,
  encode_execute_call_data,
  type SessionSpec,
} from "zksync-sso-web-sdk/bundler";

export type ToSessionSmartAccountParams<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
> = {
  client: PublicClient<TTransport, TChain>;
  sessionPrivateKey: Hash;
  address: Address;
  sessionValidatorAddress: Address;
  sessionSpec: SessionSpec;
};

export async function toSessionSmartAccount<
  TTransport extends Transport = Transport,
  TChain extends Chain = Chain,
>({
  client,
  sessionPrivateKey,
  address,
  sessionValidatorAddress,
  sessionSpec,
}: ToSessionSmartAccountParams<TTransport, TChain>): Promise<ToSmartAccountReturnType> {
  return toSmartAccount({
    client,
    entryPoint: { /* ... */ },

    async getNonce() {
      // Use keyed nonce for sessions
      const keyedNonceStr = compute_keyed_nonce(sessionSpec.signer);
      const keyedNonce = BigInt(keyedNonceStr);

      // Call EntryPoint.getNonce(sender, keyedNonce)
      // ...
    },

    async encodeCalls(calls) {
      // Only single calls supported
      if (calls.length !== 1) throw new Error("Batch not supported");

      const call = calls[0];
      return encode_execute_call_data(
        call.to,
        (call.value ?? 0n).toString(),
        call.data ?? "0x",
      ) as Hex;
    },

    async getStubSignature() {
      return generate_session_stub_signature(
        sessionValidatorAddress,
        JSON.stringify(sessionSpec), // Serialize to JSON for Rust
      ) as Hex;
    },

    async signUserOperation(userOp) {
      const hash = /* compute userOpHash */;
      return encode_session_signature(
        sessionPrivateKey,
        sessionValidatorAddress,
        JSON.stringify(sessionSpec),
        hash,
      ) as Hex;
    },

    // ... other methods
  });
}
```

**Why**: This is the core account abstraction that handles session-specific
signing.

**Test Coverage**:

```typescript
// packages/sdk-4337/src/client/session/account.test.ts
describe("toSessionSmartAccount", () => {
  it("should create session account with correct address");
  it("should compute keyed nonce correctly");
  it("should generate valid stub signature");
  it("should sign UserOperation with session key");
  it("should encode single calls correctly");
  it("should throw on batch calls");
});
```

#### 3.2: Session Deployment Support

**Location**: `packages/sdk-4337/src/client/actions/deploy.ts`

**Modify**:

```typescript
export type DeploySmartAccountParams = {
  contracts: {
    factory: Address;
    eoaValidator?: Address;
    webauthnValidator?: Address;
    sessionValidator?: Address; // NEW
  };

  eoaSigners?: Address[];
  passkeySigners?: Array<{
    /* ... */
  }>;

  // NEW: Session configuration
  sessionConfig?: {
    initialSessions?: SessionSpec[]; // Sessions to create during deployment
  };

  userId?: string;
  accountId?: Hex;
};
```

**Update Rust Call**:

```typescript
const callData = encode_deploy_account_call_data(
  accountIdBytes,
  eoaSignersJson,
  passkeySignersJson,
  sessionConfigJson, // NEW parameter
) as Hex;
```

**Why**: Enable session installation during account deployment.

**Test Coverage**:

```typescript
describe("deploySmartAccount with sessions", () => {
  it("should deploy account with session validator");
  it("should deploy account with initial sessions");
  it("should deploy with EOA, passkey, and session validators");
});
```

#### 3.3: Create Session Action

**Location**: `packages/sdk-4337/src/client/actions/createSession.ts` (NEW)

**Implementation**:

```typescript
import type { Address, Client, Hash, Hex } from "viem";
import {
  encode_add_session_call_data,
  type SessionSpec,
} from "zksync-sso-web-sdk/bundler";
import { sendUserOperation } from "viem/account-abstraction";

export type CreateSessionParams = {
  account: Address;
  sessionSpec: SessionSpec;
  sessionValidator: Address;
};

export async function createSession<TClient extends Client>(
  client: TClient,
  params: CreateSessionParams,
): Promise<{ userOpHash: Hash }> {
  const { account, sessionSpec, sessionValidator } = params;

  // Encode the createSession call
  const callData = encode_add_session_call_data(
    sessionValidator,
    JSON.stringify(sessionSpec),
  ) as Hex;

  // Send as UserOperation
  const userOpHash = await sendUserOperation(client, {
    calls: [
      {
        to: account,
        data: callData,
        value: 0n,
      },
    ],
  });

  return { userOpHash };
}
```

**Why**: Allow adding new sessions to existing accounts.

**Test Coverage**:

```typescript
describe("createSession", () => {
  it("should create new session on existing account");
  it("should create session with transfer policies");
  it("should create session with call policies");
  it("should create session with fee limits");
});
```

#### 3.4: Client Index Exports

**Location**: `packages/sdk-4337/src/client/index.ts`

**Add**:

```typescript
export { toSessionSmartAccount } from "./session/account.js";
export type { ToSessionSmartAccountParams } from "./session/account.js";
```

**Location**: `packages/sdk-4337/src/client/actions/index.ts`

**Add**:

```typescript
export { createSession } from "./createSession.js";
export type { CreateSessionParams } from "./createSession.js";
```

---

## Phase 4: Integration Tests

### Goal

Comprehensive testing across all layers to ensure compatibility.

### Test Hierarchy

#### 4.1: Rust Core Tests (No Network)

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/`

**Tests** (in each module):

```rust
// signature_wasm.rs
#[test]
fn test_session_signature_no_validation()

// encode.rs
#[test]
fn test_encode_session_user_operation()
#[test]
fn test_generate_session_stub_signature()

// send.rs (already exists)
#[test]
fn test_keyed_nonce()

// deploy.rs
#[test]
fn test_deploy_with_session_validator()
```

**Success Criteria**: All Rust tests pass with `cargo test`

#### 4.2: Rust Integration Tests (With Anvil/Bundler)

**Location**:
`packages/sdk-platforms/rust/zksync-sso-erc4337/crates/zksync-sso-erc4337-core/src/erc4337/account/modular_smart_account/session/send.rs`

**Tests** (already exists, review/update):

```rust
#[tokio::test]
async fn test_send_transaction_session() {
  // 1. Deploy account
  // 2. Fund account
  // 3. Install session validator module
  // 4. Create session
  // 5. Send transaction using session signature
  // 6. Verify transaction success
}
```

**New Test**:

```rust
#[tokio::test]
async fn test_deploy_with_session_and_transact() {
  // 1. Deploy account WITH session validator installed
  // 2. Fund account
  // 3. Create session
  // 4. Send transaction using session
  // 5. Verify success
}
```

**Success Criteria**: Tests pass with actual network/bundler interaction

#### 4.3: WASM FFI Tests (Node.js)

**Location**: `packages/sdk-platforms/web/tests/session.test.ts` (NEW)

**Tests**:

```typescript
import { describe, it, expect } from "vitest";
import {
  encode_session_signature,
  generate_session_stub_signature,
  compute_keyed_nonce,
  encode_add_session_call_data,
  type SessionSpec,
} from "../src/bundler";

describe("Session WASM Functions", () => {
  const mockSessionSpec: SessionSpec = {
    signer: "0x...",
    expiresAt: "2088558400",
    feeLimit: {
      limitType: 1,
      limit: "1000000000000000000",
      period: "0",
    },
    callPolicies: [],
    transferPolicies: [
      {
        target: "0x...",
        maxValuePerUse: "1",
        valueLimit: {
          limitType: 0,
          limit: "0",
          period: "0",
        },
      },
    ],
  };

  it("should compute keyed nonce", () => {
    const nonce = compute_keyed_nonce(mockSessionSpec.signer);
    expect(nonce).toBeDefined();
    expect(nonce).toMatch(/^0x[0-9a-fA-F]+$/);
  });

  it("should generate session stub signature", () => {
    const sig = generate_session_stub_signature(
      "0x...", // session validator
      JSON.stringify(mockSessionSpec),
    );
    expect(sig).toBeDefined();
    expect(sig).toMatch(/^0x[0-9a-fA-F]+$/);
  });

  it("should encode session signature", () => {
    const sig = encode_session_signature(
      "0x...", // private key
      "0x...", // session validator
      JSON.stringify(mockSessionSpec),
      "0x...", // hash
    );
    expect(sig).toBeDefined();
    expect(sig).toMatch(/^0x[0-9a-fA-F]+$/);
  });
});
```

**Success Criteria**: All WASM bindings work correctly in Node.js

#### 4.4: SDK-4337 Unit Tests

**Location**: `packages/sdk-4337/src/client/session/account.test.ts`

**Tests**: See section 3.1 above

**Success Criteria**: All TypeScript unit tests pass

#### 4.5: Web SDK Integration Test

**Location**: `examples/demo-app/pages/web-sdk-test.vue`

**New Components**:

##### Component 1: SessionConfig.vue

```vue
<template>
  <div class="bg-indigo-50 p-4 rounded-lg mb-4 border border-indigo-200">
    <h2 class="text-xl font-semibold mb-3">Session Configuration</h2>

    <label class="flex items-center mb-2">
      <input
        type="checkbox"
        v-model="config.enabled"
        class="mr-2"
      />
      Enable Session Support
    </label>

    <div
      v-if="config.enabled"
      class="space-y-2"
    >
      <div>
        <label class="block text-sm font-medium mb-1"
          >Session Validator Address</label
        >
        <input
          v-model="config.validatorAddress"
          type="text"
          placeholder="0x..."
          class="w-full p-2 border rounded"
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1"
          >Session Signer (auto-generated)</label
        >
        <input
          :value="config.sessionSigner"
          type="text"
          readonly
          class="w-full p-2 border rounded bg-gray-100"
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1"
          >Expires At (timestamp)</label
        >
        <input
          v-model.number="config.expiresAt"
          type="number"
          class="w-full p-2 border rounded"
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Max Fee (wei)</label>
        <input
          v-model="config.feeLimit"
          type="text"
          class="w-full p-2 border rounded"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { privateKeyToAccount } from "viem/accounts";
import { generatePrivateKey } from "viem/accounts";

const config = defineModel<SessionConfig>();

// Auto-generate session signer on enable
watch(
  () => config.value.enabled,
  (enabled) => {
    if (enabled && !config.value.sessionPrivateKey) {
      const privateKey = generatePrivateKey();
      config.value.sessionPrivateKey = privateKey;
      const account = privateKeyToAccount(privateKey);
      config.value.sessionSigner = account.address;
    }
  },
);
</script>
```

##### Component 2: SessionTransactionSender.vue

```vue
<template>
  <div class="bg-teal-50 p-4 rounded-lg mb-4 border border-teal-200">
    <h2 class="text-xl font-semibold mb-3">Send Session Transaction</h2>

    <div class="space-y-2">
      <div>
        <label class="block text-sm font-medium mb-1">Target Address</label>
        <input
          v-model="target"
          type="text"
          placeholder="0x..."
          class="w-full p-2 border rounded"
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Value (ETH)</label>
        <input
          v-model="value"
          type="text"
          placeholder="0.001"
          class="w-full p-2 border rounded"
        />
      </div>

      <button
        @click="sendTransaction"
        :disabled="loading"
        class="px-4 py-2 bg-teal-600 text-white rounded hover:bg-teal-700 disabled:opacity-50"
      >
        {{ loading ? "Sending..." : "Send Session Transaction" }}
      </button>
    </div>

    <div
      v-if="result"
      class="mt-4 p-3 bg-green-50 border border-green-200 rounded"
    >
      <p class="font-medium">Success!</p>
      <p class="text-sm">UserOp Hash: {{ result.userOpHash }}</p>
    </div>

    <div
      v-if="error"
      class="mt-4 p-3 bg-red-50 border border-red-200 rounded"
    >
      <p class="font-medium text-red-600">Error</p>
      <p class="text-sm">{{ error }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { createPublicClient, http, parseEther } from "viem";
import { toSessionSmartAccount } from "zksync-sso-4337/client";
import { createBundlerClient } from "viem/account-abstraction";

const props = defineProps<{
  accountAddress: string;
  sessionConfig: SessionConfig;
}>();

const target = ref("");
const value = ref("0.001");
const loading = ref(false);
const result = ref(null);
const error = ref("");

async function sendTransaction() {
  loading.value = true;
  error.value = "";
  result.value = null;

  try {
    // Load contracts
    const response = await fetch("/contracts.json");
    const contracts = await response.json();

    // Create clients
    const publicClient = createPublicClient({
      chain: {
        /* ... */
      },
      transport: http(contracts.rpcUrl),
    });

    const bundlerClient = createBundlerClient({
      transport: http(contracts.bundlerUrl),
      chain: {
        /* ... */
      },
    });

    // Create session account
    const sessionAccount = await toSessionSmartAccount({
      client: publicClient,
      sessionPrivateKey: props.sessionConfig.sessionPrivateKey,
      address: props.accountAddress,
      sessionValidatorAddress: props.sessionConfig.validatorAddress,
      sessionSpec: {
        signer: props.sessionConfig.sessionSigner,
        expiresAt: props.sessionConfig.expiresAt.toString(),
        feeLimit: {
          limitType: 1, // Lifetime
          limit: props.sessionConfig.feeLimit,
          period: "0",
        },
        callPolicies: [],
        transferPolicies: [
          {
            target: target.value,
            maxValuePerUse: parseEther(value.value).toString(),
            valueLimit: {
              limitType: 0, // Unlimited
              limit: "0",
              period: "0",
            },
          },
        ],
      },
    });

    // Send transaction
    const userOpHash = await bundlerClient.sendUserOperation({
      account: sessionAccount,
      calls: [
        {
          to: target.value,
          value: parseEther(value.value),
          data: "0x",
        },
      ],
    });

    result.value = { userOpHash };
  } catch (err) {
    error.value = err.message;
  } finally {
    loading.value = false;
  }
}
</script>
```

##### Update web-sdk-test.vue

**Add to template**:

```vue
<!-- Session Configuration (Optional) -->
<SessionConfig v-model="sessionConfig" />

<!-- Send Session Transaction (if session-enabled deployment) -->
<SessionTransactionSender
  v-if="deploymentResult && sessionConfig.enabled"
  :account-address="deploymentResult.address"
  :session-config="sessionConfig"
/>
```

**Add to script**:

```typescript
const sessionConfig = ref({
  enabled: false,
  validatorAddress: "",
  sessionPrivateKey: "",
  sessionSigner: "",
  expiresAt: 2088558400, // Year 2036
  feeLimit: "1000000000000000000", // 1 ETH
});

// Load session validator address on mount
onMounted(async () => {
  const response = await fetch("/contracts.json");
  if (response.ok) {
    const contracts = await response.json();
    sessionConfig.value.validatorAddress = contracts.sessionValidator;
  }
});
```

**Success Criteria**:

1. Deploy account with session validator ✅
2. Create session via UI ✅
3. Send transaction using session key ✅
4. Verify transaction success ✅

---

## Phase 5: Documentation & Examples

### 5.1: API Documentation

**Location**: `packages/sdk-4337/README.md`

**Add Section**:

```markdown
### Session-Based Smart Accounts

Session keys allow temporary, limited access to a smart account without exposing
the main signer.

#### Creating a Session Account

\`\`\`typescript import { toSessionSmartAccount } from "zksync-sso-4337/client";
import { createPublicClient, http } from "viem";

const publicClient = createPublicClient({ chain: mainnet, transport: http(), });

const sessionAccount = await toSessionSmartAccount({ client: publicClient,
sessionPrivateKey: "0x...", // Temporary session key address: "0x...", // Your
smart account address sessionValidatorAddress: "0x...", // Session validator
contract sessionSpec: { signer: "0x...", // Session key address expiresAt:
"2088558400", // Expiry timestamp feeLimit: { limitType: 1, // Lifetime limit
limit: "1000000000000000000", // 1 ETH period: "0", }, callPolicies: [], //
Function call restrictions transferPolicies: [{ // ETH transfer restrictions
target: "0x...", maxValuePerUse: "1000000000000000", // 0.001 ETH per
tx valueLimit: { limitType: 0, limit: "0", period: "0" }, }], }, });

// Use with bundler const userOpHash = await bundlerClient.sendUserOperation({
account: sessionAccount, calls: [{ to: "0x...", value: 1000000n, data: "0x" }], });
\`\`\`

#### Creating a New Session

\`\`\`typescript import { createSession } from "zksync-sso-4337/client/actions";

const { userOpHash } = await createSession(publicClient, { account: "0x...",
sessionValidator: "0x...", sessionSpec: { /_ ... _/ }, }); \`\`\`
```

### 5.2: Migration Guide

**Location**: `packages/sdk-4337/MIGRATION.md` (NEW)

**Content**:

```markdown
# Migrating from sdk (ZKsync Native) to sdk-4337 (ERC-4337)

## Session Clients

### ZKsync Native (sdk)

\`\`\`typescript import { createZksyncSessionClient } from
"zksync-sso/client/session";

const client = createZksyncSessionClient({ chain: zkSyncSepoliaTestnet,
transport: http(), address: "0x...", sessionKey: "0x...", sessionConfig: { /_
... _/ }, contracts: { session: "0x..." }, }); \`\`\`

### ERC-4337 (sdk-4337)

\`\`\`typescript import { toSessionSmartAccount } from "zksync-sso-4337/client";
import { createBundlerClient } from "viem/account-abstraction";

const account = await toSessionSmartAccount({ client: publicClient,
sessionPrivateKey: "0x...", address: "0x...", sessionValidatorAddress: "0x...",
sessionSpec: { /_ ... _/ }, });

const bundlerClient = createBundlerClient({ account, // ... }); \`\`\`

## Key Differences

1. **Account Model**: sdk-4337 uses viem's `account-abstraction` package
2. **Bundler Required**: Transactions go through a bundler, not direct RPC
3. **Nonce Management**: Uses keyed nonces (session-specific)
4. **Gas Estimation**: Requires stub signatures for accurate gas estimates
```

---

## Test Validation Matrix

| Test Layer           | Location                              | Must Pass Before | Success Criteria                   |
| -------------------- | ------------------------------------- | ---------------- | ---------------------------------- |
| **Rust Unit**        | `zksync-sso-erc4337-core/...`         | Phase 1 complete | `cargo test` passes                |
| **Rust Integration** | `zksync-sso-erc4337-core/.../send.rs` | Phase 1 complete | Can deploy + transact with session |
| **WASM FFI**         | `sdk-platforms/web/tests/`            | Phase 2 complete | Node.js tests pass                 |
| **TS Unit**          | `sdk-4337/src/client/session/`        | Phase 3 complete | Vitest passes                      |
| **E2E Integration**  | `demo-app/pages/web-sdk-test.vue`     | Phase 4 complete | Full flow works in browser         |

## Critical Issues to Address

### 1. Time Encoding in WASM

**Problem**: Rust `std::time` functions don't work in WASM  
**Solution**:

- Accept timestamp as parameter (caller provides)
- OR compute period IDs without time validation
- Defer all validation to on-chain execution

**Verification**: WASM tests must pass without time-related errors

### 2. JSON Serialization for WASM

**Problem**: Rust structs need JSON bridge to JS  
**Solution**:

- Use `serde_json` for SessionSpec serialization
- TS passes `JSON.stringify(sessionSpec)` to Rust
- Rust deserializes with `serde_json::from_str`

**Verification**: Round-trip serialization tests

### 3. Gas Estimation

**Problem**: Session signatures are large, need accurate gas estimates  
**Solution**:

- Implement `generate_session_stub_signature` that matches real signature size
- Use in `getStubSignature()` for viem account

**Verification**: Gas estimates must be within 10% of actual usage

### 4. Keyed Nonce Calculation

**Problem**: Sessions use non-sequential nonces  
**Solution**:

- Use `compute_keyed_nonce(sessionSigner)` to derive nonce key
- Pass to `EntryPoint.getNonce(sender, key)`

**Verification**: Nonce must match Rust implementation exactly

---

## Success Metrics

- [ ] All Rust tests pass
- [ ] All WASM tests pass
- [ ] All TypeScript tests pass
- [ ] Can deploy account with session validator via web-sdk-test
- [ ] Can create session via web-sdk-test
- [ ] Can send transaction using session key via web-sdk-test
- [ ] Documentation complete
- [ ] No time-related WASM errors
- [ ] Gas estimation accurate (within 10%)

---

## Timeline Estimate

- **Phase 1** (Rust Core): 3-5 days
- **Phase 2** (FFI/WASM): 2-3 days
- **Phase 3** (SDK-4337 TS): 3-4 days
- **Phase 4** (Integration Tests): 2-3 days
- **Phase 5** (Documentation): 1-2 days

**Total**: ~11-17 days (2-3 weeks)

---

## Dependencies

### External

- `viem` v2.30.0+
- `viem/account-abstraction`
- `zksync-sso-web-sdk` (WASM bindings)

### Internal

- Deployed contracts: `SessionKeyValidator`, `MSAFactory`, `EntryPoint`
- Running bundler for integration tests

---

## Risk Mitigation

### Risk: WASM Time Functions

**Mitigation**: Create time-agnostic signature functions early, test in browser
immediately

### Risk: Signature Encoding Mismatch

**Mitigation**: Generate test vectors in Rust, verify in TypeScript

### Risk: Nonce Conflicts

**Mitigation**: Extensive testing of keyed nonce calculation across languages

### Risk: Gas Estimation Failures

**Mitigation**: Stub signature must match real signature format exactly

---

## Open Questions

1. **Should deployment support multiple initial sessions?**  
   → Yes, to minimize deployment transactions

2. **Should we support batch transactions with sessions?**  
   → No, start with single execute() only (like EOA implementation)

3. **Do we need session revocation in this phase?**  
   → Not critical for MVP, can add later

4. **Should sessions be installable post-deployment without EOA signer?**  
   → Yes, via another valid signer (EOA or passkey)

---

## Next Steps

1. **Review this plan** with team
2. **Prioritize** any missing requirements
3. **Begin Phase 1** (Rust Core implementation)
4. **Set up CI** to run tests automatically
5. **Track progress** against success metrics
