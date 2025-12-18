import type { Address, Hex } from "viem";
import { createPublicClient, http } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import type { Chain } from "viem/chains";
import { createPasskeyClient } from "zksync-sso-4337/client";

import contractsConfig from "../contracts-anvil.json";

// Anvil chain configuration (chain ID 1337 to match erc4337-contracts setup)
const anvilChain: Chain = {
  id: 1337,
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
    const bundlerUrl = contractsConfig.bundlerUrl;

    return createBundlerClient({
      client: publicClient,
      chain,
      transport: http(bundlerUrl),
      // EntryPoint 0.8 - no paymaster for now to simplify testing
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
    getPublicClient,
    getBundlerClient,
    getClient,
    getConfigurableClient,
  };
});
