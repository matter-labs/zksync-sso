<template>
  <main class="h-full flex flex-col justify-center px-4">
    <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-12" />

    <div class="space-y-2">
      <h2 class="text-3xl font-bold text-center mb-2 text-gray-900 dark:text-white">
        Account in Recovery
      </h2>
      <p class="text-center text-gray-600 dark:text-gray-400 text-lg">
        Your account is not ready yet
      </p>
    </div>

    <div class="flex flex-col items-center mt-8">
      <div class="p-6 rounded-lg bg-gray-50 dark:bg-gray-800 max-w-md w-full text-center">
        <p class="text-gray-600 dark:text-gray-300 mb-4">
          Your account is currently in the recovery process. It will be ready in <span class="font-semibold text-gray-900 dark:text-white">XXhs</span>.
        </p>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Please check back later
        </p>
      </div>
    </div>

    <div class="mt-8 text-center">
      <ZkLink
        type="ghost"
        href="/"
        class="inline-flex items-center gap-2 justify-center"
      >
        <ZkIcon
          icon="arrow_back"
        />
        Back to Home
      </ZkLink>
    </div>
  </main>
</template>
