<template>
  <main class="h-full flex flex-col justify-center px-4">
    <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-12" />

    <div class="space-y-2">
      <h2 class="text-3xl font-bold text-center mb-2 text-gray-900 dark:text-white">
        Welcome Back
      </h2>
      <p class="text-center text-gray-600 dark:text-gray-400 text-lg">
        Recover your account
      </p>
    </div>

    <div class="flex flex-col gap-5 mt-4 py-8">
      <ZkLink
        type="primary"
        href="/recovery/guardian"
      >
        Recover with Guardian
      </ZkLink>
      <div class="flex flex-col gap-2">
        <ZkLink
          type="primary"
          as="button"
          disabled
        >
          Recover with Email
        </ZkLink>
        <p class="text-sm text-gray-500 text-center">
          Coming soon...
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
