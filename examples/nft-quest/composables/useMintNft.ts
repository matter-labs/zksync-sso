import { waitForTransactionReceipt, writeContract } from "@wagmi/core";
import type { Address } from "viem";
import { getGeneralPaymasterInput } from "viem/zksync";

import { Nft, Paymaster } from "~/abi";
import type { SupportedChainId } from "~/stores/connector";

export const useMintNft = async (_address: MaybeRef<Address>) => {
  const address = toRef(_address);

  return await useAsyncData("mintZeek", async () => {
    const runtimeConfig = useRuntimeConfig();
    const { wagmiConfig } = storeToRefs(useConnectorStore());
    const chainId = runtimeConfig.public.defaultChainId as SupportedChainId;

    const mintingForAddress = address.value;

    const transactionHash = await writeContract(wagmiConfig.value, {
      address: Nft.addressByChain[chainId] as Address,
      abi: Nft.Abi,
      functionName: "mint",
      args: [mintingForAddress],
      paymaster: Paymaster.addressByChain[chainId] as Address,
      paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
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
