<template>
  <div>
    <layout-header>
      <template #default>
        Sessions
      </template>
      <template #aside>
        <!-- <transition v-bind="TransitionOpacity">
          <ZkButton
            v-if="sessions?.length"
            type="danger"
          >
            <template #prefix>
              <HandRaisedIcon
                class="h-5 w-5 mr-1"
                aria-hidden="true"
              />
            </template>
            <span class="leading-tight">End all sessions</span>
          </ZkButton>
        </transition> -->
      </template>
    </layout-header>

    <CommonAlert
      v-if="sessions?.length"
      class="mb-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          ZKsync SSO is still under development. The displayed spending amounts may not always be accurate.
        </p>
      </template>
    </CommonAlert>

    <CommonAlert
      v-if="sessionsFetchError"
      class="mb-4"
      type="error"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm font-semibold">
          Failed to load sessions
        </p>
        <p class="text-xs mt-1">
          {{ sessionsFetchError.message }}
        </p>
      </template>
    </CommonAlert>

    <span
      v-if="!sessions?.length && !sessionsInProgress && !sessionsFetchError"
      data-testid="empty-sessions-message"
      class="font-thin block text 2xl text-neutral-500 text-center"
    >No active sessions...</span>
    <div
      v-else
      class="border rounded-3xl divide-y bg-white border-neutral-200 divide-neutral-200 dark:bg-neutral-950 dark:border-neutral-900 dark:divide-neutral-900"
    >
      <template v-if="!sessions?.length && sessionsInProgress">
        <SessionRowLoader
          v-for="index in 3"
          :key="index"
        />
      </template>
      <template v-else>
        <SessionRow
          v-for="item in (sessions || [])"
          :key="item.sessionHash"
          :session-hash="item.sessionHash"
          :session-spec="item.sessionSpec"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { InformationCircleIcon } from "@heroicons/vue/20/solid";
import type { Address, Hex } from "viem";
import { listActiveSessions } from "zksync-sso-4337";
import { LimitType, type SessionConfig } from "zksync-sso-4337/client";

const { defaultChain } = useClientStore();
const { address } = storeToRefs(useAccountStore());

// Types for WASM-returned session data (snake_case with string values)
interface WasmUsageLimit {
  limitType: string;
  limit: string;
  period: string;
}

interface WasmConstraint {
  condition: string;
  index: string;
  refValue: Hex;
  limit: WasmUsageLimit;
}

interface WasmCallPolicy {
  target: Address;
  selector: Hex;
  maxValuePerUse: string;
  valueLimit: WasmUsageLimit;
  constraints?: WasmConstraint[];
}

interface WasmTransferPolicy {
  target: Address;
  maxValuePerUse: string;
  valueLimit: WasmUsageLimit;
}

interface WasmSessionSpec {
  signer: Address;
  expiresAt: string;
  feeLimit: WasmUsageLimit;
  callPolicies?: WasmCallPolicy[];
  transferPolicies?: WasmTransferPolicy[];
}

// Helper to convert WASM session spec format to proper TypeScript types
const convertSessionSpec = (wasmSpec: WasmSessionSpec): SessionConfig => {
  const convertLimit = (limit: WasmUsageLimit) => {
    // Map string limitType to enum value
    let limitType: LimitType;
    if (limit.limitType === "Unlimited") limitType = LimitType.Unlimited;
    else if (limit.limitType === "Lifetime") limitType = LimitType.Lifetime;
    else if (limit.limitType === "Allowance") limitType = LimitType.Allowance;
    else limitType = Number(limit.limitType) as LimitType;

    return {
      limitType,
      limit: BigInt(limit.limit),
      period: BigInt(limit.period),
    };
  };

  return {
    signer: wasmSpec.signer,
    expiresAt: BigInt(wasmSpec.expiresAt),
    feeLimit: convertLimit(wasmSpec.feeLimit),
    callPolicies: (wasmSpec.callPolicies || []).map((policy: WasmCallPolicy) => ({
      target: policy.target,
      selector: policy.selector,
      maxValuePerUse: BigInt(policy.maxValuePerUse),
      valueLimit: convertLimit(policy.valueLimit),
      constraints: (policy.constraints || []).map((constraint: WasmConstraint) => ({
        condition: constraint.condition,
        index: BigInt(constraint.index),
        refValue: constraint.refValue,
        limit: convertLimit(constraint.limit),
      })),
    })),
    transferPolicies: (wasmSpec.transferPolicies || []).map((policy: WasmTransferPolicy) => ({
      target: policy.target,
      maxValuePerUse: BigInt(policy.maxValuePerUse),
      valueLimit: convertLimit(policy.valueLimit),
    })),
  };
};

const {
  result: sessions,
  inProgress: sessionsInProgress,
  error: sessionsFetchError,
  execute: sessionsFetch,
} = useAsync(async () => {
  const contracts = contractsByChain[defaultChain.id];

  // Get RPC URL from the chain configuration
  const rpcUrl = defaultChain.rpcUrls.default.http[0];

  // Use the new listActiveSessions function from the SDK
  const { sessions: activeSessions } = await listActiveSessions({
    account: address.value,
    rpcUrl,
    contracts: {
      sessionValidator: contracts.sessionValidator,
      entryPoint: "0x0000000071727De22E5E9d8BAf0edAc6f37da032", // Standard EntryPoint v0.8
      accountFactory: contracts.factory,
      webauthnValidator: contracts.webauthnValidator,
      eoaValidator: contracts.eoaValidator,
      guardianExecutor: contracts.guardianExecutor || "0x0000000000000000000000000000000000000000",
    },
  });

  // Map snake_case properties from WASM to camelCase and convert types
  const filtered = activeSessions
    .filter((item: { session_hash?: Hex; session_spec?: WasmSessionSpec }) => {
      const isValid = item?.session_hash && item?.session_spec && item?.session_spec?.signer;
      if (!isValid) {
        // eslint-disable-next-line no-console
        console.warn("[sessions.vue] Filtering out invalid session:", item);
      }
      return isValid;
    })
    .map((item: { session_hash: Hex; session_spec: WasmSessionSpec }) => ({
      sessionHash: item.session_hash,
      sessionSpec: convertSessionSpec(item.session_spec),
    }));

  return filtered;
});

sessionsFetch();
</script>
