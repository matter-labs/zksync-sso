<template>
  <div class="h-full flex justify-center flex-col p-4">
    <div>
      <BlurFade
        in-view
        :delay="0"
        class="block"
      >
        <span class="text-[40px] font-bold tracking-tighter dark:text-white leading-1">
          Mint your NFT.&nbsp;
        </span>
      </BlurFade>

      <BlurFade
        in-view
        :delay="350"
        class="inline"
      >
        <span class="text-[40px] font-bold tracking-tighter leading-1 text-blue-600">
          For free.
        </span>
      </BlurFade>
    </div>
    <BlurFade
      in-view
      :delay="50"
      class="inline"
    >
      <!-- Show session setup prompt if no session is active -->
      <template v-if="!hasSession">
        <p class="mt-8 text-neutral-400 max-w-prose">
          To mint NFTs without popup approvals, you need to authorize a session. This allows the app to perform transactions on your behalf within the specified limits.
        </p>
        <div class="sticky bottom-[-8px] pb-4 bg-gradient-to-t from-black to-transparent w-full flex justify-center sm:justify-start">
          <ZkButton
            type="primary"
            class="uppercase mt-8"
            data-testid="add-session"
            @click="addSession"
          >
            Add Session
          </ZkButton>
        </div>
      </template>

      <!-- Show mint UI when session is active -->
      <template v-else>
        <p class="mt-8 text-neutral-400 max-w-prose">
          Your authorized session for the NFT quest is now active! You can mint your NFT without any additional transaction approvals.
        </p>
        <p class="mt-8 text-neutral-400 max-w-prose">
          Also, it's free. ZKsync can leverage paymasters to enable app developers to choose a custom gas token or even entirely sponsor gas fees for their users.
        </p>
        <div class="sticky bottom-[-8px] pb-4 bg-gradient-to-t from-black to-transparent w-full flex justify-center sm:justify-start">
          <ZkButton
            type="primary"
            class="uppercase mt-8"
            :loading="status === 'pending'"
            @click="mintNFT"
          >
            Mint 100% free NFT
          </ZkButton>
        </div>
      </template>

      <p
        v-if="status === 'error'"
        class="text-error-400 mt-4 text-sm"
      >
        An error occurred. Please try minting again.
      </p>
    </BlurFade>
  </div>
</template>

<script setup lang="ts">
const { account, hasSession } = storeToRefs(useConnectorStore());
const { connectWithSession } = useConnectorStore();
const { error: mintNFTError, execute: mintNFT, status, data } = await useMintNft(computed(() => account.value.address!));

const addSession = async () => {
  try {
    await connectWithSession();
  } catch (error) {
    console.error("Failed to add session:", error);
  }
};

watch(status, (status) => {
  if (status === "success") {
    navigateTo({ path: "mint/share", query: { tx: data.value!.transactionHash } });
  } else if (status === "error") {
    console.error(mintNFTError.value);
  }
});
</script>
