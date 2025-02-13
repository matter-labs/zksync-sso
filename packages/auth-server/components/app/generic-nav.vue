<template>
  <div
    class="border border-neutral-200 rounded-zk flex justify-between dark:border-neutral-900 dark:bg-neutral-950 h-[52px]"
  >
    <div class="flex items-center pl-3">
      <NuxtLink to="/">
        <app-account-logo
          :height="24"
          class="dark:text-neutral-100"
        />
      </NuxtLink>
    </div>
    <div
      class="flex items-center pr-2"
    >
      <app-color-mode />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useWindowSize } from "@vueuse/core";
import { onBeforeUnmount, onMounted, watch } from "vue";

const { width: windowWidth } = useWindowSize();
const menuWrapper = useTemplateRef("menu-wrapper");
const menu = useTemplateRef("menu");
const menuWidth = ref(0);

const showMobileMenu = ref(false);

const checkWidths = () => {
  const menuWrapperWidth = menuWrapper.value?.offsetWidth || 0;
  if (menuWrapperWidth <= menuWidth.value) {
    showMobileMenu.value = true;
  } else {
    showMobileMenu.value = false;
  }
};

onMounted(() => {
  menuWidth.value = menu.value?.offsetWidth || 0;
  checkWidths();
  window.addEventListener("resize", checkWidths);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", checkWidths);
});

watch(windowWidth, checkWidths);
</script>

  <style lang="scss" scoped>
  .router-link-exact-active {
    @apply border-b-neutral-700 text-neutral-900 dark:text-neutral-100 dark:border-b-neutral-200 dark:hover:text-neutral-100
  }
  </style>
