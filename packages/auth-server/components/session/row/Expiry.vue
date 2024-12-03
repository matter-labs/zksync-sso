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
          class="bg-white rounded-full h-full will-change-[width] transition-[width] duration-300"
          :style="{ width: `${timeLeftPercentage}%` }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SessionStatus } from "zksync-sso/utils";

const props = defineProps<{
  status: SessionStatus;
  isExpired: boolean;
  now: number;
  createdAt: number;
  expiresAt: number;
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
const timeTotal = computed<number>(() => Math.max(0, props.expiresAt - props.createdAt));
const timeLeftPercentage = computed<number>(() => Math.min(100, (timeLeft.value / timeTotal.value) * 100));
const isRevoked = computed(() => props.status === SessionStatus.Closed);
</script>
