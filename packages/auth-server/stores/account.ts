import { StorageSerializers, useStorage } from "@vueuse/core";
import type { Address, Hex } from "viem";

type SmartAccount = {
  address: Address;
  credentialId: Hex;
};

export const useAccountStore = defineStore("account", () => {
  const accountData = useStorage<SmartAccount | null>("account", null, undefined, {
    serializer: StorageSerializers.object,
  });
  const address = computed(() => accountData.value?.address || null);
  const credentialId = computed(() => accountData.value?.credentialId || null);
  const isLoggedIn = computed(() => !!address.value);
  const login = (data: SmartAccount) => {
    accountData.value = data;
  };
  const logout = () => {
    accountData.value = null;
  };

  // Verify cached account still exists on-chain (handles chain restarts / redeployments)
  const verifyAccount = async () => {
    if (!address.value) return;
    try {
      const { getClient, contracts } = useClientStore();
      const client = getClient();
      const entryPoint = contracts.entryPoint;
      if (!entryPoint) return;
      await client.readContract({
        address: entryPoint,
        abi: [{
          type: "function",
          name: "getNonce",
          inputs: [
            { type: "address", name: "sender" },
            { type: "uint192", name: "key" },
          ],
          outputs: [{ type: "uint256" }],
          stateMutability: "view",
        }],
        functionName: "getNonce",
        args: [address.value, 0n],
        account: address.value,
      });
    } catch {
      console.warn("Cached account no longer exists on-chain, logging out");
      logout();
    }
  };

  if (import.meta.client) {
    verifyAccount();
  }

  const { subscribe: subscribeOnAccountChange, notify: notifyOnAccountChange } = useObservable<Address | null>();
  watch(address, (newAddress) => {
    notifyOnAccountChange(newAddress);
  });

  return {
    address,
    credentialId,
    isLoggedIn,
    subscribeOnAccountChange,
    login,
    logout,
  };
});
