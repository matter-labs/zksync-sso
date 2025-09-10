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
    const response = await $fetch<{ message?: string; nonce?: string; error?: string }>(`${proxyBaseUrl}/permissions/user-wallets/initiate`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: {
        walletAddress: address,
        domain: window.location.host,
      },
    });

    if (response.error) {
      throw new Error(response.error);
    }

    if (!response.message || !response.nonce) {
      throw new Error("Invalid response: missing message or nonce");
    }

    return response as { message: string; nonce: string };
  };

  const associateAddress = async (address: string, message: string, signature: string) => {
    const associateResponse = await $fetch<{ success?: boolean; error?: string }>(`${proxyBaseUrl}/permissions/user-wallets/associate`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: {
        walletAddress: address,
        message: message,
        signature,
      },
    });

    if (associateResponse.error) {
      throw new Error(associateResponse.error);
    }

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
