import type { Address } from "viem";

export const useMintNft = async (_address: MaybeRef<Address>) => {
  const address = toRef(_address);

  return await useAsyncData("mintZeek", async () => {
    const runtimeConfig = useRuntimeConfig();
    const clientStore = useClientStore();

    const client = clientStore.getClient();

    const mintingForAddress = address.value;

    const hash = await client.writeContract({
      address: runtimeConfig.public.contracts.nft as Address,
      abi: nftAbi,
      functionName: "mint",
      args: [mintingForAddress],
    });

    // Wait for transaction receipt
    const receipt = await client.waitForTransactionReceipt({ hash });

    if (receipt.status !== "success") {
      throw new Error("Mint transaction failed");
    }

    return receipt;
  }, {
    server: false,
    immediate: false,
  });
};
