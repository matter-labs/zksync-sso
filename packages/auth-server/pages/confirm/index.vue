<template>
  <TransitionGroup
    v-bind="TransitionOpacity"
    tag="div"
    mode="out-in"
    class="h-dvh"
  >
    <ViewsLoading
      v-if="loading && !hasRequests"
      key="loading"
    />
    <ViewsAuth
      v-else-if="requestMethod === 'eth_requestAccounts'"
      key="login"
    />
    <ViewsConfirmationProhibittedTransactionTarget
      v-else-if="hasProhibitedCallTarget"
      key="prohibited-target"
    />
    <ViewsConfirmationPersonalSign
      v-else-if="requestMethod === 'personal_sign'"
      key="personal-sign"
    />
    <ViewsConfirmationSignTypedData
      v-else-if="requestMethod === 'eth_signTypedData_v4'"
      key="sign-typed-data"
    />
    <ViewsConfirmationSend
      v-else-if="requestMethod === 'eth_sendTransaction'"
      key="confirmation"
    />
  </TransitionGroup>
</template>

<script lang="ts" setup>
import { getAddress } from "viem";
import type { ExtractParams } from "zksync-sso-4337/client";

const { isLoggedIn } = storeToRefs(useAccountStore());
const { hasRequests, requestParams, requestMethod, requestChainId } = storeToRefs(useRequestsStore());

const loading = ref(true);

const { checkTargetAddress } = useProhibitedCallsCheck(requestChainId);
const hasProhibitedCallTarget = computed(() => {
  if (requestMethod.value === "eth_sendTransaction") {
    const [transaction] = requestParams.value as ExtractParams<"eth_sendTransaction">;
    if (!transaction) return false;
    const isTargetProhibited = transaction.to && checkTargetAddress(getAddress(transaction.to.toLowerCase()));
    const isContractCall = transaction.data && transaction.data !== "0x";
    return isTargetProhibited && isContractCall;
  }
  return false;
});

watch(requestMethod, () => {
  if (isLoggedIn.value && requestMethod.value === "eth_requestAccounts") {
    navigateTo({ path: "/confirm/connect" });
  } else {
    loading.value = false;
  }
});
</script>
