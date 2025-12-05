import type { Address, Hex } from "viem";

export const useAccountStore = defineStore("account", () => {
  const address = ref<Address | undefined>(undefined);
  const credentialId = ref<Hex | undefined>(undefined);

  const isConnected = computed(() => !!address.value && !!credentialId.value);

  const setAccount = (addr: Address, credId: Hex) => {
    address.value = addr;
    credentialId.value = credId;
  };

  const clearAccount = () => {
    address.value = undefined;
    credentialId.value = undefined;
  };

  return {
    address,
    credentialId,
    isConnected,
    setAccount,
    clearAccount,
  };
});
