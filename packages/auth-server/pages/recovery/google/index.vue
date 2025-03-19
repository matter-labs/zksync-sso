<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <div class="flex flex-col items-center gap-8 mt-4">
        <account-recovery-oidc-recovery-flow-root
          @back="previousStep"
          @forward="nextStep"
        />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: "dashboard",
});
</script>
