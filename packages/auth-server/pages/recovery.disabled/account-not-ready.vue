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
      <p
        v-if="recoveryState"
        class="text-center text-gray-600 dark:text-gray-400 text-lg"
      >
        {{ recoveryState.type === 'notYet' ? "Your account is not ready yet" : "Your recovery request already expired"
        }}
      </p>
    </div>

    <div
      v-if="recoveryState?.type === 'notYet'"
      class="flex flex-col items-center mt-8"
    >
      <div
        v-if="recoveryState"
        class="p-6 rounded-lg bg-gray-50 dark:bg-gray-800 max-w-md w-full text-center"
      >
        <p class="text-gray-600 dark:text-gray-300 mb-4">
          Your account is currently in the recovery process. It will be ready in <span
            class="font-semibold text-gray-900 dark:text-white"
          >{{ formatDuration(intervalToDuration({
            start: 0,
            end: Number(recoveryState.remainingTime * 1000n),
          })) }}</span>.
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
import { z } from "zod";

import { AddressSchema } from "@/utils/schemas";

const route = useRoute();
const { checkRecoveryRequest } = useRecoveryGuardian();

const recoveryState = ref<{ type: "notYet"; remainingTime: bigint } | { type: "expired" } | null>(null);
const countdownInterval = ref<NodeJS.Timeout | null>(null);

// Function to start the countdown timer
const startCountdown = () => {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value);
  }

  countdownInterval.value = setInterval(() => {
    if (recoveryState.value?.type === "notYet" && recoveryState.value.remainingTime > 0n) {
      recoveryState.value.remainingTime -= 1n;

      // If countdown reaches zero, update the state
      if (recoveryState.value.remainingTime <= 0n) {
        clearInterval(countdownInterval.value!);
        checkRecoveryStatus();
      }
    }
  }, 1000);
};

// Function to check recovery status
const checkRecoveryStatus = async () => {
  const params = z.object({
    address: AddressSchema,
  }).safeParse(route.query);

  if (!params.success) {
    throw createError({
      statusCode: 404,
      statusMessage: "Page not found",
      fatal: true,
    });
  }

  const recoveryData = await checkRecoveryRequest({ address: params.data.address });
  if (!recoveryData) {
    throw createError({
      statusCode: 404,
      statusMessage: "Page not found",
      fatal: true,
    });
  }

  if (!recoveryData.pendingRecovery) {
    navigateTo("/");
    return;
  }

  if (recoveryData.remainingTime > 0n) {
    recoveryState.value = {
      type: "notYet",
      remainingTime: recoveryData.remainingTime,
    };
    startCountdown();
    return;
  } else if (recoveryData.ready) {
    navigateTo("/");
    return;
  }

  recoveryState.value = {
    type: "expired",
  };
};

// Clean up interval when component is unmounted
onUnmounted(() => {
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value);
  }
});

// Initial check
watchEffect(checkRecoveryStatus);
</script>
