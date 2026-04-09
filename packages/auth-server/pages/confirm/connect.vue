<template>
  <div class="h-[100dvh]">
    <ViewsLoading
      v-if="loading"
      key="loading"
    />
    <ViewsConnect
      v-else-if="requestMethod === 'eth_requestAccounts'"
      key="connect"
    />
    <ViewsConfirmationMethodNotSupported
      v-else
      key="unsupported"
      :method="requestMethod!"
    />
  </div>
</template>

<script lang="ts" setup>
const { requestMethod } = storeToRefs(useRequestsStore());
const { isLoggedIn } = storeToRefs(useAccountStore());

const loading = computed(() => !isLoggedIn.value);
</script>
