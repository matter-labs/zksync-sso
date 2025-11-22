import type { Address } from "viem";

export const useMintNft = async (_address: MaybeRef<Address>) => {
  const address = toRef(_address);

  return await useAsyncData("mintZeek", async () => {
    const runtimeConfig = useRuntimeConfig();
    const clientStore = useClientStore();

    // WORKAROUND: Use throw-away client (rich account #0) to mint
    // TODO: Use passkey client with bundler once factory contracts are deployed
    const client = clientStore.getThrowAwayClient();

    const mintingForAddress = address.value;

    // Submit mint transaction directly (not via bundler)
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
