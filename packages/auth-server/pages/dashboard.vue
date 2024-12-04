<template>
  <div>
    <header class="max-w-[1920px] mx-auto mb-8">
      <AppNav />
    </header>
    <main class="max-w-[900px] m-auto">
      <NuxtPage />
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  middleware: ["logged-in"],
  layout: "dashboard",
});
</script>
