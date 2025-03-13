<template>
  <main class="h-full flex flex-col justify-center px-4">
    <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-12" />

    <div
      class="space-y-2"
      :loading="!recoveryState"
    >
      <h2 class="text-3xl font-bold text-center mb-2 text-gray-900 dark:text-white">
        Account in Recovery
      </h2>
      <p class="text-center text-gray-600 dark:text-gray-400 text-lg">
        {{ recoveryState!.type === 'notYet' ? "Your account is not ready yet" : "Your recovery request already expired"
        }}
      </p>
    </div>

    <div
      v-if="recoveryState!.type === 'notYet'"
      class="flex flex-col items-center mt-8"
    >
      <div class="p-6 rounded-lg bg-gray-50 dark:bg-gray-800 max-w-md w-full text-center">
        <p class="text-gray-600 dark:text-gray-300 mb-4">
          Your account is currently in the recovery process. It will be ready in <span
            class="font-semibold text-gray-900 dark:text-white"
          >{{ recoveryState!.time }}</span>.
        </p>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Please check back later
        </p>
      </div>
    </div>

    <div class="mt-8 text-center">
      <ZkLink
        type="ghost"
        href="/"
        class="inline-flex items-center gap-2 justify-center"
      >
        <ZkIcon icon="arrow_back" />
        Back to Home
      </ZkLink>
    </div>
  </main>
</template>

<script setup lang="ts">
import { formatDuration, intervalToDuration } from "date-fns";
import type { Address } from "viem";
import { z } from "zod";

import { AddressSchema } from "@/utils/schemas";

const route = useRoute();
const { checkRecoveryRequest, getPendingRecoveryData } = useRecoveryGuardian();

const accountAddress = ref<Address | null>(null);

const params = z.object({
  address: AddressSchema,
}).safeParse(route.query);

const pendingRecovery = ref<Awaited<ReturnType<typeof checkRecoveryRequest>> | null>(null);
const recoveryState = ref<{ type: "notYet"; time: string } | { type: "expired" } | null>(null);

if (!params.success) {
  throw createError({
    statusCode: 404,
    statusMessage: "Page not found",
    fatal: true,
  });
} else {
  accountAddress.value = params.data.address;
  const recoveryRequestData = await getPendingRecoveryData(accountAddress.value);
  if (recoveryRequestData) {
    const recoveryData = await checkRecoveryRequest(recoveryRequestData[2]);
    pendingRecovery.value = recoveryData;
    if (recoveryData && recoveryData[2] > 0n) {
      recoveryState.value = {
        type: "notYet", time: formatDuration(intervalToDuration({
          start: 0,
          end: parseFloat((recoveryData[2]).toString()) * 1000,
        })),
      };
    } else if (recoveryData && recoveryData[2] === 0n) {
      recoveryState.value = {
        type: "expired",
      };
    }
  }
}
</script>
