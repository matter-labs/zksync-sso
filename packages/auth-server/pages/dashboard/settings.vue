<template>
  <div>
    <layout-header> Settings </layout-header>
    <div class="flex gap-4 lg:flex-row flex-col">
      <aside class="lg:w-1/4 mb-3 lg:mb-0">
        <nav class="flex gap-x-2 lg:flex-col lg:gap-x-0 lg:gap-y-1">
          <NuxtLink
            v-for="link in NavLinks"
            :key="link.path"
            :to="link.path"
            :class="['aside-link', { active: $route.path === link.path }]"
          >
            {{ link.title }}
          </NuxtLink>
        </nav>
      </aside>
      <div class="flex flex-1 lg:p-1 lg:ml-4 flex-col gap-6">
        <div>
          <h2 class="text-lg font-medium">
            {{ currentLink?.title }}
          </h2>
          <p class="text-sm text-neutral-500">
            {{ currentLink?.description }}
          </p>
        </div>
        <div
          role="none"
          class="shrink-0 bg-neutral-100 dark:bg-neutral-900 h-[1px] w-full"
        />
        <NuxtPage />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const NavLinks = [
  {
    title: "Account Recovery",
    description: "Configure your account recovery settings.",
    path: "/dashboard/settings/account-recovery",
  },
  {
    title: "Owner Keys",
    description: "Configure ECDSA Private Keys.",
    path: "/dashboard/settings/owner-keys",
  },
  {
    title: "Passkeys",
    description: "Configure WebAuthn Passkeys.",
    path: "/dashboard/settings/passkeys",
  },
  {
    title: "Sessions",
    description: "Manage Application Sessions.",
    path: "/dashboard/settings/sessions",
  },
];

const route = useRoute();
const currentLink = computed(() => NavLinks.find((link) => link.path === route.path));
</script>

<style lang="scss" scoped>
.aside-link {
  @apply hover:underline text-sm font-medium h-9 px-4 py-2 rounded-3xl dark:text-neutral-100;

  &.active {
    @apply bg-neutral-100 dark:bg-neutral-900;
  }
}
</style>
