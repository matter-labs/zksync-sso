import type { Abi, Address } from "viem";

export const useNftMetadata = async ({
  chainId,
  address,
  abi,
}: { chainId?: SupportedChainId; address: Address; abi: Abi }) => {
  const { getPublicClient, defaultChain } = useClientStore();

  const client = getPublicClient({ chainId: chainId ?? defaultChain.id });
  const res = await client.readContract({
    address: address,
    abi,
    functionName: "tokenURI",
  });

  return await useFetch<{ animation_url: string; background_color: string; description: string; image: string }>(res, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
  });
};
