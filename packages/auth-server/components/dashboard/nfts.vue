<template>
  <div>
    <ZkPanelSection
      title="NFTs"
      :ui="{footer: 'flex justify-center'}"
    >
      <ZkTableBody>
        <NftZeekNftQuest :max-display="maxCount" />
      </ZkTableBody>
      <template
        v-if="maxCount<100"
        #footer
      >
        <ZkButton
          type="secondary"
          @click="maxCount=100"
        >
          View all
        </ZkButton>
      </template>
    </ZkPanelSection>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const maxCount = ref<number>(5);
</script>
