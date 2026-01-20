import { waitForTransactionReceipt, writeContract } from "@wagmi/core";
import type { Address } from "viem";

export const useMintNft = async (_address: MaybeRef<Address>) => {
  const address = toRef(_address);

  return await useAsyncData("mintZeek", async () => {
    const runtimeConfig = useRuntimeConfig();
    const { wagmiConfig } = storeToRefs(useConnectorStore());

    const mintingForAddress = address.value;
    // Paymaster is configured at connector level, no need to specify here
    const transactionHash = await writeContract(wagmiConfig.value, {
      address: runtimeConfig.public.contracts.nft as Address,
      abi: nftAbi,
      functionName: "mint",
      args: [mintingForAddress],
    });

    const transactionReceipt = await waitForTransactionReceipt(wagmiConfig.value, { hash: transactionHash });
    if (transactionReceipt.status === "reverted") {
      throw new Error("Transaction reverted");
    }

    return transactionReceipt;
  }, {
    server: false,
    immediate: false,
  });
};
