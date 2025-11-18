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
