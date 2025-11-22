import type { Address, Hex } from "viem";
import { createPublicClient, createWalletClient, http, publicActions } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { privateKeyToAccount } from "viem/accounts";
import type { Chain } from "viem/chains";
import { createPasskeyClient } from "zksync-sso-4337/client";

import contractsConfig from "../contracts-anvil.json";

// Anvil chain configuration (chain ID 31337)
const anvilChain: Chain = {
  id: 31337,
  name: "Anvil",
  nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
  rpcUrls: {
    default: { http: ["http://127.0.0.1:8545"] },
  },
};

export const useClientStore = defineStore("client", () => {
  const { address, credentialId } = storeToRefs(useAccountStore());

  const chain = anvilChain;

  const getPublicClient = () => {
    return createPublicClient({
      chain,
      transport: http(),
    });
  };

  const getBundlerClient = () => {
    const publicClient = getPublicClient();

    return createBundlerClient({
      client: publicClient,
      chain,
      transport: http(contractsConfig.bundlerUrl),
      userOperation: {
        async estimateFeesPerGas() {
          const feesPerGas = await publicClient.estimateFeesPerGas();
          return {
            callGasLimit: 2_000_000n,
            verificationGasLimit: 2_000_000n,
            preVerificationGas: 1_000_000n,
            ...feesPerGas,
          } as const;
        },
      },
    });
  };

  const getClient = () => {
    if (!address.value) throw new Error("Address is not set");
    if (!credentialId.value) throw new Error("Credential ID is not set");

    const bundlerClient = getBundlerClient();

    const client = createPasskeyClient({
      account: {
        address: address.value,
        validatorAddress: contractsConfig.webauthnValidator as Address,
        credentialId: credentialId.value,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain,
      transport: http(),
    });

    return client;
  };

  const getThrowAwayClient = () => {
    const throwAwayClient = createWalletClient({
      account: privateKeyToAccount(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // Anvil rich account
      ),
      chain,
      transport: http(),
    })
      .extend(publicActions);
    return throwAwayClient;
  };

  const getConfigurableClient = ({
    address: addr,
    credentialId: credId,
  }: {
    address: Address;
    credentialId: Hex;
  }) => {
    const bundlerClient = getBundlerClient();

    return createPasskeyClient({
      account: {
        address: addr,
        validatorAddress: contractsConfig.webauthnValidator as Address,
        credentialId: credId,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain,
      transport: http(),
    });
  };

  return {
    chain,
    contractsConfig,
    getPublicClient,
    getBundlerClient,
    getClient,
    getThrowAwayClient,
    getConfigurableClient,
  };
});
