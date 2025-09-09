export const usePrividiumAddressAssociation = () => {
  const { getPrividiumInstance } = usePrividiumAuthStore();
  const runtimeConfig = useRuntimeConfig();
  const proxyBaseUrl = runtimeConfig.public.prividium?.proxyBaseUrl;

  const getAuthHeaders = () => {
    const prividium = getPrividiumInstance();
    if (!prividium) {
      throw new Error("Prividium not initialized");
    }

    return prividium.getAuthHeaders()!;
  };

  const fetchAddressAssociationMessage = async (address: string) => {
    return await $fetch<{ message: string; nonce: string }>(`${proxyBaseUrl}/permissions/user-wallets/initiate`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: {
        walletAddress: address,
        domain: window.location.host,
      },
    });
  };

  const associateAddress = async (address: string, message: string, signature: string) => {
    const associateResponse = await $fetch<{ success: boolean }>(`${proxyBaseUrl}/permissions/user-wallets/associate`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: {
        walletAddress: address,
        message: message,
        signature,
      },
    });

    if (!associateResponse.success) {
      throw new Error("Failed to associate address");
    }
  };

  const deleteAddressAssociation = async (address: string) => {
    await $fetch(`${proxyBaseUrl}/permissions/user-wallets/${address}`, {
      method: "DELETE",
      headers: getAuthHeaders(),
    });
  };

  return {
    fetchAddressAssociationMessage,
    associateAddress,
    deleteAddressAssociation,
  };
};
