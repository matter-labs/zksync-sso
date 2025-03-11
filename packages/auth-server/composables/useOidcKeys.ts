import { StandardMerkleTree } from "@openzeppelin/merkle-tree";
import { OidcKeyRegistryAbi } from "zksync-sso/abi";
import type { OidcKey } from "zksync-sso/client/oidc";

export const useOidcKeys = () => {
  const { getPublicClient, defaultChain } = useClientStore();

  const getOidcKeysInProgress = ref(false);
  const getOidcKeysError = ref<Error | null>(null);
  const getOidcKeysData = ref<readonly OidcKey[] | null>(null);

  async function getOidcKeys() {
    getOidcKeysInProgress.value = true;
    getOidcKeysError.value = null;

    try {
      const client = getPublicClient({ chainId: defaultChain.id });
      const data = await client.readContract({
        address: contractsByChain[defaultChain.id].oidcKeyRegistry,
        abi: OidcKeyRegistryAbi,
        functionName: "getKeys",
        args: [],
      });
      getOidcKeysData.value = data as OidcKey[];
      return;
    } catch (err) {
      console.log(err);
      getOidcKeysError.value = err as Error;
      return;
    } finally {
      getOidcKeysInProgress.value = false;
    }
  }

  function getMerkleProof(key: OidcKey) {
    if (getOidcKeysData.value === null) {
      throw new Error("Oidc keys not found");
    }

    const keys = getOidcKeysData.value.map((key) => [key.issHash, key.kid, key.n, key.e]);
    const tree = StandardMerkleTree.of(keys, ["bytes32", "bytes32", "uint256[17]", "bytes"]);
    const proof = tree.getProof([key.issHash, key.kid, key.n, key.e]);

    return proof;
  }

  return {
    getOidcKeys,
    getOidcKeysInProgress,
    getOidcKeysError,
    getOidcKeysData,
    getMerkleProof,
  };
};
