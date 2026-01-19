<template>
  <div>
    <div
      v-if="isExpired"
      class="text-neutral-500"
    >
      Expired
    </div>
    <div
      v-else-if="isRevoked"
      class="text-neutral-500"
    >
      Revoked
    </div>
    <div v-else-if="status === SessionStatus.NotInitialized">
      Not initialized
    </div>
    <div v-else>
      <div :title="`${sessionExpiry.formattedDate} at ${sessionExpiry.formattedTime}`">
        <span v-if="sessionExpiry.isToday">Expires {{ expiresIn }}</span>
        <span v-else-if="sessionExpiry.isTomorrow">Expires tomorrow at {{ sessionExpiry.formattedTime }}</span>
        <span v-else>Expires on {{ sessionExpiry.formattedDate }} at {{ sessionExpiry.formattedTime }}</span>
      </div>
      <div class="session-row-line">
        <div
          class="session-row-line-inner"
          :style="{ width: `${timeLeftPercentage}%` }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SessionStatus } from "zksync-sso-4337/client";

const props = defineProps<{
  status: SessionStatus;
  isExpired: boolean;
  now: number;
  expiresAt: number;
  maxExpiresAt: number;
}>();

const expiresIn = useTimeAgo(props.expiresAt, { showSecond: true, updateInterval: 1000 });

const sessionExpiry = computed(() => {
  const expiresDate = new Date(props.expiresAt);
  const nowDate = new Date(props.now);

  return formatExpiryDate({
    expiresAt: expiresDate,
    now: nowDate,
  });
});
const timeLeft = computed<number>(() => Math.max(0, props.expiresAt - props.now));
const maxTimeLeft = computed<number>(() => Math.max(0, props.maxExpiresAt - props.now));
const timeLeftPercentage = computed<number>(() => {
  if (maxTimeLeft.value === 0) return 0;
  return Math.min(100, (timeLeft.value / maxTimeLeft.value) * 100);
});
const isRevoked = computed(() => props.status === SessionStatus.Closed);
</script>
