<template>
  <div>
    <layout-header>
      <template #default>
        Sessions
      </template>
    </layout-header>

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
      class="font-thin block text-2xl text-neutral-500 text-center"
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
import { type ConstraintCondition, LimitType, type SessionSpec } from "zksync-sso-4337/client";

const { defaultChain } = useClientStore();
const { address } = storeToRefs(useAccountStore());

// Types for WASM-returned session data (snake_case with string values)
interface WasmUsageLimit {
  limitType: string;
  limit: string;
  period: string;
}

interface WasmConstraint {
  condition: string; // Will be converted to ConstraintCondition enum
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
const convertSessionSpec = (wasmSpec: WasmSessionSpec): SessionSpec => {
  const convertLimit = (limit: WasmUsageLimit) => {
    // Map string limitType to enum value
    let limitType: LimitType;
    if (limit.limitType === "Unlimited") limitType = LimitType.Unlimited;
    else if (limit.limitType === "Lifetime") limitType = LimitType.Lifetime;
    else if (limit.limitType === "Allowance") limitType = LimitType.Allowance;
    else {
      const numericLimitType = Number(limit.limitType);
      if (Number.isNaN(numericLimitType)) {
        // eslint-disable-next-line no-console
        console.warn(
          "Unexpected limitType value received from WASM:",
          limit.limitType,
        );
        // Fallback to a safe default to avoid propagating an invalid enum value
        limitType = LimitType.Unlimited;
      } else {
        limitType = numericLimitType as LimitType;
      }
    }

    // Validate and convert BigInt values with try-catch
    let limitValue: bigint;
    let periodValue: bigint;
    try {
      limitValue = BigInt(limit.limit);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.warn("Invalid limit value from WASM, defaulting to 0:", limit.limit, e);
      limitValue = 0n;
    }
    try {
      periodValue = BigInt(limit.period);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.warn("Invalid period value from WASM, defaulting to 0:", limit.period, e);
      periodValue = 0n;
    }

    return {
      limitType,
      limit: limitValue,
      period: periodValue,
    };
  };

  // Helper to safely convert BigInt with validation
  const safeBigInt = (value: string, fieldName: string): bigint => {
    try {
      return BigInt(value);
    } catch (e) {
      // eslint-disable-next-line no-console
      console.warn(`Invalid ${fieldName} value from WASM, defaulting to 0:`, value, e);
      return 0n;
    }
  };

  return {
    signer: wasmSpec.signer,
    expiresAt: safeBigInt(wasmSpec.expiresAt, "expiresAt"),
    feeLimit: convertLimit(wasmSpec.feeLimit),
    callPolicies: (wasmSpec.callPolicies || []).map((policy: WasmCallPolicy) => ({
      target: policy.target,
      selector: policy.selector,
      maxValuePerUse: safeBigInt(policy.maxValuePerUse, "maxValuePerUse"),
      valueLimit: convertLimit(policy.valueLimit),
      constraints: (policy.constraints || []).map((constraint: WasmConstraint) => ({
        condition: constraint.condition as unknown as ConstraintCondition,
        index: safeBigInt(constraint.index, "constraint.index"),
        refValue: constraint.refValue,
        limit: convertLimit(constraint.limit),
      })),
    })),
    transferPolicies: (wasmSpec.transferPolicies || []).map((policy: WasmTransferPolicy) => ({
      target: policy.target,
      maxValuePerUse: safeBigInt(policy.maxValuePerUse, "transfer.maxValuePerUse"),
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

  if (address.value === null) {
    throw new Error("Account address is null");
  }
  // Use the new listActiveSessions function from the SDK
  const { sessions: activeSessions } = await listActiveSessions({
    account: address.value,
    rpcUrl,
    contracts: {
      sessionValidator: contracts.sessionValidator,
      entryPoint: (contracts as { entryPoint?: Address }).entryPoint || "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
      accountFactory: contracts.factory,
      webauthnValidator: contracts.webauthnValidator,
      eoaValidator: contracts.eoaValidator,
      guardianExecutor: contracts.guardianExecutor || "0x0000000000000000000000000000000000000000",
    },
  });

  // Map snake_case properties from WASM to camelCase and convert types
  type WasmSession = { session_hash: Hex; session_spec: WasmSessionSpec };
  const filtered = (activeSessions as unknown as WasmSession[])
    .filter((item) => {
      const isValid = item?.session_hash && item?.session_spec && item?.session_spec?.signer;
      if (!isValid) {
        // eslint-disable-next-line no-console
        console.warn("[sessions.vue] Filtering out invalid session:", item);
      }
      return isValid;
    })
    .map((item) => ({
      sessionHash: item.session_hash,
      sessionSpec: convertSessionSpec(item.session_spec),
    }));

  return filtered;
});

sessionsFetch();
</script>
