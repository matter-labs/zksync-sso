<template>
  <div>
    <layout-header>
      <span class="font-thin">Welcome to ZKsync SSO</span>
      <template #aside>
        <span class="text-lg text-neutral-500 mr-2">
          {{ shortenAddress(address!) }}
        </span>
        <zk-copy :content="address!" />
      </template>
    </layout-header>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <dashboard-assets />

      <dashboard-nfts />
    </div>
  </div>
</template>

<script setup lang="ts">
const { address } = useAccountStore();
</script>
