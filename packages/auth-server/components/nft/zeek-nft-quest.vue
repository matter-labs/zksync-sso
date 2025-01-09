<template>
  <div>
    <ZkTableRow
      v-if="hasNft"
      ui="px-4"
    >
      <ZkTableCellData>
        <video
          autoplay
          loop
          muted
          class="aspect-square max-w-[68px]"
        >
          <source
            :src="nftMetadata?.animation_url"
            type="video/mp4"
          >
        </video>
      </ZkTableCellData>
      <ZkTableCellData class="flex flex-col justify-center">
        <ZkTableCellData class="flex-auto text-sm">
          {{ nftMetadata?.description }}
          <template #sub>
            NFT Quest
          </template>
        </ZkTableCellData>
      </ZkTableCellData>
    </ZkTableRow>
    <ZkTableRow
      v-else
      ui="px-4"
    >
      <ZkTableCellData class="w-full text-center text-neutral-500 py-4 flex items-center justify-center">
        We can't find any NFTs related to this account.
        <br>Try collecting your first NFT at <a
          href=""
          target="_blank"
          class="text-blue-500"
        >NFT Quest</a>.
      </ZkTableCellData>
    </ZkTableRow>
  </div>
</template>

<script setup lang="ts">
import type { Address } from "viem";

import { ZeekNftQuestAbi } from "~/abi/ZeekNFTQuest";

const runtimeConfig = useRuntimeConfig();
const { address } = useAccountStore();
const chainId = runtimeConfig.public.chainId as SupportedChainId;
const nftAddress = runtimeConfig.public[chainId].nftQuestAddress as Address;
const nftMetadata = ref<null | { animation_url: string; background_color: string; description: string; image: string }>(null);
const hasNft = ref(false);

const getNFTTransactions = async function () {
  const { getPublicClient, defaultChain } = useClientStore();

  const client = getPublicClient({ chainId: chainId ?? defaultChain.id });
  const res = await client.readContract({
    address: nftAddress,
    abi: ZeekNftQuestAbi,
    functionName: "balanceOf",
    args: [address as Address],
  });

  if (res) {
    hasNft.value = true;
  } else {
    hasNft.value = false;
    return;
  }

  const fetchNftMetadata = await useNftMetadata({ address: nftAddress, abi: ZeekNftQuestAbi });
  nftMetadata.value = fetchNftMetadata.data.value as { animation_url: string; background_color: string; description: string; image: string };
};

await getNFTTransactions();
</script>
