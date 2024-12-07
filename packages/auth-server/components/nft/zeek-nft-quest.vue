<template>
  <div>
    <ZkTableRow
      v-if="myNfts.length > 0"
      ui="px-4 hover:cursor-pointer"
      v-for="nft in myNfts"
      @click="navigateTo(nft.url, { external: true, open: { target: '_blank' } })"
    >
      <ZkTableCellData>
        <video
          v-if="nft.tokenMetadata?.animation_url"
          autoplay
          loop
          muted
          class="aspect-square max-w-[68px] rounded"
        >
          <source
            :src="nft.tokenMetadata?.animation_url"
            type="video/mp4"
          >
        </video>
        <img 
          v-if="nft.tokenMetadata?.image && !nft.tokenMetadata?.animation_url"
          class="aspect-square max-w-[68px] rounded"
          :src="nft.tokenMetadata?.image"
        />
        <UnknownNft 
          v-if="!nft.tokenMetadata?.image"
          class="aspect-square w-[68px] rounded"
        />
      </ZkTableCellData>
      <ZkTableCellData class="flex flex-col justify-center">
        <ZkTableCellData class="flex-auto text-sm" ui-sub="truncate-to-2-lines">
          <span class="font-bold text-base">
            {{ nft.tokenName }}
          </span>
          <span class="italic">
            (#{{ nft.tokenID }})
          </span>
          <template #sub>
            {{ nft.tokenMetadata?.description ?? nft.tokenName }}
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
          href="https://nft.zksync.dev"
          target="_blank"
          class="text-blue-500"
        >NFT Quest</a>.
      </ZkTableCellData>
    </ZkTableRow>
  </div>
</template>

<script setup lang="ts">
import { ContractFunctionExecutionError, type Address } from "viem";

import { ZeekNftQuestAbi } from "~/abi/ZeekNFTQuest";
import UnknownNft from "../icons/UnknownNft.vue";

const runtimeConfig = useRuntimeConfig();
const { address } = useAccountStore();
const chainId = runtimeConfig.public.chainId as SupportedChainId;
const nftAddress = runtimeConfig.public[chainId].nftQuestAddress as Address;
const nftMetadata = ref<null | NftMetadata>(null);
const hasNft = ref(false);
const myNfts = ref<AccountNftTransfer[]>([]);
const allNfts = ref<AccountNftTransfer[]>([]);
const MAX_NFTS_TO_DISPLAY = 5;

interface AccountNftTransferResponse {
  message: string;
  status: string;
  result: AccountNftTransfer[];
}

interface AccountNftTransfer {
  type: string;
  hash: string;
  to: string;
  from: string;
  transactionIndex: string;
  input: string;
  value: string;
  gas: string;
  gasPrice: string;
  gasUsed: string;
  cumulativeGasUsed: string;
  fee: string;
  nonce: string;
  confirmations: string;
  blockNumber: string;
  blockHash: string;
  l1BatchNumber: string;
  timeStamp: string;
  contractAddress: string;
  tokenDecimal: string;
  transactionType: string;
  tokenID: string;
  tokenURI: string;
  tokenSymbol: string;
  tokenName: string;
  tokenMetadata?: NftMetadata;
  invalidTokenURI?: boolean;
  url?: string;
}

interface NftMetadata {
  animation_url: string;
  background_color: string;
  description: string;
  image: string
}

function convertIpfsToGateway(ipfsUrl: string): string {
  return ipfsUrl.startsWith("ipfs://") ? ipfsUrl.replace("ipfs://", "https://ipfs.io/ipfs/") : ipfsUrl;
}

const getNFTTransactions = async function () {
  const { getPublicClient, defaultChain } = useClientStore();

  const client = getPublicClient({ chainId: chainId ?? defaultChain.id });

  try {
    const nftTransferUrl = `${blockExplorerApiByChain[chainId]}?module=account&action=tokennfttx&page=1&offset=100&sort=desc&endblock=99999999&startblock=0&address=${address}`;

    const { data: getNftTransferInfoResponse } = await useFetch(nftTransferUrl);
    const result = await getNftTransferInfoResponse.value as AccountNftTransferResponse;
    
    if (result.message == "OK") {
      const seenTokenIDs = new Set();
      let filteredNfts = [];

      for (const nft of result.result) {
        const uniqueID = `${nft.contractAddress}-${nft.tokenID}`;
        if (nft.to === address && !seenTokenIDs.has(uniqueID)) {
          filteredNfts.push(nft);
          seenTokenIDs.add(uniqueID);
        }
      }

      filteredNfts = filteredNfts.slice(0, MAX_NFTS_TO_DISPLAY);
      // const filteredNfts = result.result
      //   .filter(x => x.to == address)
      //   .slice(0, MAX_NFTS_TO_DISPLAY);
      allNfts.value = filteredNfts;
      
      try {
        await Promise.all(filteredNfts.map(async (nft) => {
          let tokenURI = await client.readContract({
            address: nft.contractAddress as Address,
            abi: ZeekNftQuestAbi,
            functionName: "tokenURI",
            args: [BigInt(+nft.tokenID)],
          });

          if (tokenURI == "") {
            // Try the backup function tokenURI()
            try {
              tokenURI = await client.readContract({
                address: nft.contractAddress as Address,
                abi: ZeekNftQuestAbi,
                functionName: "tokenURI",
                args: [],
              });
            } catch (error) {
              if (error instanceof ContractFunctionExecutionError) {
                // Contract does not have any tokenURI
                console.warn(`NFT ${nft.contractAddress} (${nft.tokenID}) does not have a valid tokenURI on-chain`);
                nft.invalidTokenURI = true;
                return;
              } else {
                throw error;
              }
            }
          }

          nft.tokenURI = tokenURI;
          nft.url = `${blockExplorerUrlByChain[chainId]}address/${nft.contractAddress}`;

          nft.tokenSymbol = await client.readContract({
            address: nft.contractAddress as Address,
            abi: ZeekNftQuestAbi,
            functionName: "symbol",
            args: [],
          });
          nft.tokenName = await client.readContract({
            address: nft.contractAddress as Address,
            abi: ZeekNftQuestAbi,
            functionName: "name",
            args: [],
          });

          if (tokenURI != "") {
            tokenURI = convertIpfsToGateway(tokenURI);

            const metadataResponse = await useFetch<NftMetadata>(tokenURI, {
              method: "GET",
              headers: {
                "Content-Type": "application/json",
              },
            });

            if (metadataResponse.status.value == "success" && metadataResponse.data.value) {
              nft.tokenMetadata = metadataResponse.data.value;

              if (nft.tokenMetadata?.image) {
                nft.tokenMetadata.image = convertIpfsToGateway(nft.tokenMetadata.image);
              }
              if (nft.tokenMetadata?.animation_url) {
                nft.tokenMetadata.animation_url = convertIpfsToGateway(nft.tokenMetadata.animation_url);
              }
            }
          }
        }));
      } catch (error) {
        // Unexpected error should not pause the loading of all other NFTs on the page
        console.log(error);
      }

      // filteredNfts.forEach(nft => {
      //   console.log("NFT Address:", nft.contractAddress);
      //   console.log("NFT ID:", nft.tokenID);
      //   console.log("NFT Symbol:", nft.tokenSymbol);
      //   console.log("NFT Name:", nft.tokenName);
      //   console.log("NFT URI:", nft.tokenURI ?? "NO URI PROVIDED");
      //   console.log("NFT Meta Description:", nft.tokenMetadata?.description)
      //   console.log("NFT Meta Background Color:", nft.tokenMetadata?.background_color);
      //   console.log("NFT Meta Image:", nft.tokenMetadata?.image);
      //   console.log("NFT Meta Animated Image:", nft.tokenMetadata?.animation_url);
      //   console.log("");
      // });

      myNfts.value = filteredNfts.filter(nft => nft.invalidTokenURI != true);
    }

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
  } catch (_error) {
    hasNft.value = false;
    return;
  }

  const fetchNftMetadata = await useNftMetadata({ address: nftAddress, abi: ZeekNftQuestAbi });
  nftMetadata.value = fetchNftMetadata.data.value as { animation_url: string; background_color: string; description: string; image: string };
};

await getNFTTransactions();
</script>
