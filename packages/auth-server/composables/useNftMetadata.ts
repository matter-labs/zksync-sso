import type { Abi, Address } from "viem";

export const useNftMetadata = async ({
  address,
  abi,
}: { address: Address; abi: Abi }) => {
  const { getPublicClient } = useClientStore();

  const client = getPublicClient();
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
