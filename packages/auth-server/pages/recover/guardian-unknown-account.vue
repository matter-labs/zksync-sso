<template>
  <div class="min-h-screen">
    <header class="max-w-[1920px] mx-auto mb-12">
      <app-generic-nav />
    </header>
    <main class="max-w-[900px] mx-auto flex flex-col gap-6">
      <div class="flex flex-col items-center gap-8 mt-4">
        <h1 class="text-3xl font-medium text-neutral-900 dark:text-neutral-100">
          Account Recovery Not Possible
        </h1>

        <div class="w-full max-w-md flex flex-col gap-6">
          <p class="text-neutral-700 dark:text-neutral-300 text-center">
            If you don't remember your address or your guardian address, unfortunately there's no way to recover the account.
          </p>
          <p class="text-neutral-700 dark:text-neutral-300 text-center">
            Please verify if you have this information backed up somewhere else.
          </p>

          <ZkButton
            class="w-full mt-4"
            @click="navigateTo('/recover')"
          >
            Back to Recovery Options
          </ZkButton>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: "dashboard",
});
</script>
