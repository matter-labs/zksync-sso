import { waitForTransactionReceipt, writeContract } from "@wagmi/core";
import type { Address } from "viem";

export const useMintNft = async (_address: MaybeRef<Address>) => {
  const address = toRef(_address);

  return await useAsyncData("mintZeek", async () => {
    const runtimeConfig = useRuntimeConfig();
    const { wagmiConfig } = storeToRefs(useConnectorStore());

    const mintingForAddress = address.value;
    console.log("[useMintNft] Starting mint for address:", mintingForAddress);
    console.log("[useMintNft] NFT contract:", runtimeConfig.public.contracts.nft);

    try {
      // Paymaster is configured at connector level, no need to specify here
      console.log("[useMintNft] Calling writeContract...");
      const transactionHash = await writeContract(wagmiConfig.value, {
        address: runtimeConfig.public.contracts.nft as Address,
        abi: nftAbi,
        functionName: "mint",
        args: [mintingForAddress],
      });
      console.log("[useMintNft] Transaction hash:", transactionHash);

      console.log("[useMintNft] Waiting for transaction receipt...");
      const transactionReceipt = await waitForTransactionReceipt(wagmiConfig.value, { hash: transactionHash });
      console.log("[useMintNft] Transaction status:", transactionReceipt.status);

      if (transactionReceipt.status === "reverted") {
        console.error("[useMintNft] Transaction reverted!");
        throw new Error("Transaction reverted");
      }

      console.log("[useMintNft] Mint successful!");
      return transactionReceipt;
    } catch (error) {
      console.error("[useMintNft] Error during mint:", error);
      throw error;
    }
  }, {
    server: false,
    immediate: false,
  });
};
