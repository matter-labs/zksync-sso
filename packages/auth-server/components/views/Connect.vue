<template>
  <ViewsConfirmationRequestAccounts v-if="!sessionPreferences" />
  <template v-else>
    <ViewsConfirmationProhibittedSession v-if="hasProhibitedCallTarget" />
    <ViewsConfirmationRequestSession
      v-else
      :session-preferences="sessionPreferences"
    />
  </template>
</template>

<script lang="ts" setup>
import { getAddress } from "viem";
import type { SessionPreferences } from "zksync-sso";

const { requestParams, requestMethod, requestChainId } = storeToRefs(useRequestsStore());

// TODO: if user is logged in and has an active session,
// display the request account view
// if user is logged in but does not have an active session,
// display the request session view

const sessionPreferences = computed<SessionPreferences | undefined>(() => {
  if (requestMethod.value !== "eth_requestAccounts") return undefined;
  if ("sessionPreferences" in requestParams.value!) {
    return requestParams.value!.sessionPreferences;
  }
  return undefined;
});

const { checkTargetAddress } = useProhibitedCallsCheck(requestChainId);
const hasProhibitedCallTarget = computed(() => {
  if (!sessionPreferences.value) return false;
  return (sessionPreferences.value.contractCalls || []).some(
    (policy) => checkTargetAddress(getAddress(policy.address.toLowerCase())),
  );
});
</script>
