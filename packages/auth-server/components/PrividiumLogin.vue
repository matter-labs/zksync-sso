<template>
  <div class="h-full flex flex-col justify-center px-4">
    <div class="max-w-md mx-auto w-full">
      <AppAccountLogo class="dark:text-neutral-100 h-16 md:h-20 mb-14" />
      <!-- Error display -->
      <CommonHeightTransition :opened="!!error">
        <p class="pb-3 text-sm text-error-300 text-center">
          {{ error }}
        </p>
      </CommonHeightTransition>

      <!-- Not authenticated: Show Prividium auth button -->
      <div
        v-if="!isAuthenticated"
        class="flex flex-col gap-5"
      >
        <ZkHighlightWrapper>
          <ZkButton
            class="w-full"
            :loading="loading"
            data-testid="prividium-login"
            @click="handlePrividiumLogin"
          >
            <template #icon>
              <svg
                class="w-5 h-5"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z" />
              </svg>
            </template>
            Authorize with Prividium
          </ZkButton>
        </ZkHighlightWrapper>
      </div>

      <!-- Authenticated: Show account creation flow or login options -->
      <div
        v-else
        class="flex flex-col"
      >
        <!-- Profile display -->
        <div class="bg-slate-50 dark:bg-slate-800 rounded-lg p-3 border border-slate-200 dark:border-slate-700 mb-6">
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-green-500 rounded-full" />
              <span class="text-sm text-slate-600 dark:text-slate-400">
                {{ profile?.displayName || profile?.userId || "Authenticated User" }}
              </span>
            </div>
            <button
              class="text-xs text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 underline"
              @click="logout"
            >
              Sign Out
            </button>
          </div>
        </div>

        <!-- Account creation or login flow -->
        <div
          v-if="!accountDeployed"
          class="flex flex-col gap-5"
        >
          <ZkHighlightWrapper>
            <ZkButton
              class="w-full"
              :loading="deployInProgress"
              data-testid="prividium-create-account"
              @click="deployAccount"
            >
              Create New Account
            </ZkButton>
          </ZkHighlightWrapper>

          <ZkButton
            type="secondary"
            class="!text-slate-400"
            :loading="loginInProgress"
            data-testid="prividium-login-existing"
            @click="logIn"
          >
            Log In to Existing Account
          </ZkButton>
        </div>

        <!-- Address association step (after account deployment) -->
        <div
          v-else
          class="space-y-6"
        >
          <!-- Progress indicator -->
          <div class="mb-6 px-8">
            <div class="flex items-center justify-between">
              <div class="flex flex-col items-center">
                <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium bg-primary-300 text-white">
                  ✓
                </div>
                <span class="text-xs text-slate-600 dark:text-slate-400 mt-2 text-center">Account Created</span>
              </div>
              <div
                :class="[
                  'flex-1 h-1 mx-2 self-start mt-3.5',
                  addressAssociated ? 'bg-primary-300' : 'bg-slate-200 dark:bg-slate-700',
                ]"
              />
              <div class="flex flex-col items-center">
                <div
                  :class="[
                    'w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ring-2 ring-offset-1',
                    addressAssociated
                      ? 'bg-primary-300 text-white ring-primary-300 ring-offset-white dark:ring-offset-slate-900'
                      : 'bg-white dark:bg-slate-900 text-primary-300 ring-primary-300 ring-offset-white dark:ring-offset-slate-900',
                  ]"
                >
                  {{ addressAssociated ? '✓' : '2' }}
                </div>
                <span class="text-xs text-slate-600 dark:text-slate-400 mt-2 text-center">Complete Setup</span>
              </div>
            </div>
          </div>

          <!-- Complete Setup -->
          <div class="bg-slate-50 dark:bg-slate-800 rounded-lg p-4">
            <p class="text-center text-sm text-slate-600 dark:text-slate-400 mb-4">
              Confirm your passkey to complete the setup.
            </p>
            <ZkButton
              class="w-full"
              :loading="associateInProgress"
              :disabled="addressAssociated"
              @click="executeAssociation"
            >
              {{ addressAssociated ? 'Setup Complete ✓' : 'Confirm Passkey' }}
            </ZkButton>
          </div>

          <!-- Go Back Button -->
          <div class="text-center">
            <button
              class="text-sm text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200 underline"
              @click="resetToInitialState"
            >
              Go Back
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { toHex, zeroAddress } from "viem";
import { createZksyncPasskeyClient } from "zksync-sso/client/passkey";

const runtimeConfig = useRuntimeConfig();
const chainId = runtimeConfig.public.chainId as SupportedChainId;

const prividiumAuthStore = usePrividiumAuthStore();
const { loading, isAuthenticated, profile } = storeToRefs(prividiumAuthStore);
const { createTransport } = useClientStore();
const { login } = useAccountStore();
const { fetchAddressAssociationMessage, associateAddress } = usePrividiumAddressAssociation();
const { loginInProgress, loginToAccount } = useAccountLogin(chainId);
const { registerInProgress: deployInProgress, createAccount, createAccountError } = useAccountCreate(chainId, true);

const accountDeploymentResult = ref<Awaited<ReturnType<typeof createAccount>> | null>(null);

// Use separate getClient since the one from client store requires login data to already be present
// but we need it before logging user in
const getClient = () => {
  if (!accountDeploymentResult.value) throw new Error("No deployed account available");
  const chain = supportedChains.find((chain) => chain.id === chainId);
  if (!chain) throw new Error(`Chain with id ${chainId} is not supported`);
  const contracts = contractsByChain[chainId];

  const client = createZksyncPasskeyClient({
    address: accountDeploymentResult.value.address,
    credentialPublicKey: accountDeploymentResult.value.credentialPublicKey,
    userName: accountDeploymentResult.value.credentialId,
    userDisplayName: accountDeploymentResult.value.credentialId,
    contracts,
    chain,
    transport: createTransport(),
  });

  return client;
};

const { inProgress: associateInProgress, execute: executeAssociation, error: associationError } = useAsync(async () => {
  if (!accountDeploymentResult.value) {
    throw new Error("No deployed account to associate address with.");
  }

  // Get passkey client with the deployed account
  const passkeyClient = getClient();

  // Fetch association message
  const { message } = await fetchAddressAssociationMessage(passkeyClient.account.address);

  // Sign with passkey
  const domain = {
    name: "AddressAssociationVerifier",
    version: "1.0.0",
    chainId: chainId,
    verifyingContract: zeroAddress,
  } as const;

  const signature = await passkeyClient.signTypedData({
    domain: {
      ...domain,
      salt: undefined, // Otherwise the signature verification fails
    },
    types: {
      AddressAssociation: [
        { name: "message", type: "string" },
      ],
    },
    primaryType: "AddressAssociation",
    message: {
      message,
    },
  });

  // Associate the address
  await associateAddress(passkeyClient.account.address, message, signature);

  login({
    username: accountDeploymentResult.value.credentialId,
    address: accountDeploymentResult.value.address,
    passkey: toHex(accountDeploymentResult.value.credentialPublicKey),
  });
  addressAssociated.value = true;

  // Navigate to dashboard after successful association
  setTimeout(() => {
    navigateTo("/dashboard");
  }, 1000);
});

const addressAssociated = ref(false);
const accountDeployed = computed(() => !!accountDeploymentResult.value);
const error = computed(() => createAccountError.value?.message || associationError.value?.message || "");

const resetToInitialState = () => {
  accountDeploymentResult.value = null;
  addressAssociated.value = false;
};

const handlePrividiumLogin = async () => {
  await prividiumAuthStore.signInWithPopup();
};

const logout = () => {
  prividiumAuthStore.signOut();
  resetToInitialState();
};

const deployAccount = async () => {
  accountDeploymentResult.value = await createAccount();
};

const logIn = async () => {
  const result = await loginToAccount();
  if (result?.success) {
    navigateTo("/dashboard");
    return;
  }
  if (result?.recoveryRequest?.isReady === false) {
    navigateTo(`/recovery/account-not-ready?address=${result!.recoveryRequest.accountAddress}`);
    return;
  }
};
</script>
