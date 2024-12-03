<template>
  <CommonButtonLineWithImg
    as="div"
    class="token-balance-loader"
  >
    <template #image>
      <CommonContentLoader
        :length="0"
        class="token-balance-image-container"
      />
    </template>
    <template #default>
      <CommonButtonLineBodyInfo class="text-left">
        <template #label>
          <CommonContentLoader :length="24" />
        </template>
        <template #underline>
          <CommonContentLoader :length="16" />
        </template>
      </CommonButtonLineBodyInfo>
    </template>
  </CommonButtonLineWithImg>
</template>

<script lang="ts" setup>
defineProps({
  sendRouteName: {
    type: Boolean,
    default: false,
  },
});
</script>

<style lang="scss" scoped>
.token-balance-loader {
  @apply pointer-events-none;

  .token-balance-image-container {
    @apply block aspect-square rounded-full;
  }
}
</style>
