import { StorageSerializers, useStorage } from "@vueuse/core";
import { type Address, type Hash, toBytes } from "viem";

type SmartAccount = {
  factory: Address; // where was this account made (helps with version)
  username: string; // unique id in the factory (prevent duplicate sign-up)
  address: Address; // public address (of the SSO Beacon proxy deployed)
  passkey: Hash; // Raw Public Key (for signing)
  ownerPublicKey: Address; // owner id of the account
  ownerPrivateKey: Hash; // owner signing key of the account for signing
};

export const useAccountStore = defineStore("account", () => {
  const accountData = useStorage<SmartAccount | null>("account", null, undefined, {
    serializer: StorageSerializers.object,
  });
  const address = computed(() => accountData.value?.address || null);
  const passkey = computed(() => accountData.value?.passkey ? toBytes(accountData.value?.passkey) : null);
  const username = computed(() => accountData.value?.username || null);
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
    passkey,
    username,
    isLoggedIn,
    subscribeOnAccountChange,
    login,
    logout,
  };
});
