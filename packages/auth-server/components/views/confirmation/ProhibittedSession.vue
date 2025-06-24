<template>
  <SessionTemplate>
    <template
      v-if="isLoggedIn"
      #header
    >
      <SessionAccountHeader message="Connecting with" />
    </template>

    <SessionMetadata
      :app-meta="appMeta"
      :domain="domain"
      size="sm"
    />

    <CommonAlert
      class="mt-4"
      variant="error"
    >
      <template #icon>
        <ShieldExclamationIcon aria-hidden="true" />
      </template>
      <template #default>
        <p>
          One of the requested onchain actions target address is not allowed.
        </p>
      </template>
    </CommonAlert>

    <template #footer>
      <ZkButton
        class="w-full"
        type="secondary"
        @click="deny()"
      >
        Close
      </ZkButton>
    </template>
  </SessionTemplate>
</template>

<script lang="ts" setup>
import { ShieldExclamationIcon } from "@heroicons/vue/20/solid";

const { appMeta, domain } = useAppMeta();
const { isLoggedIn } = storeToRefs(useAccountStore());
const { deny } = useRequestsStore();
</script>
