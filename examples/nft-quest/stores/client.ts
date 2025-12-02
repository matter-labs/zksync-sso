import type { Address, Hex } from "viem";
import { createPublicClient, http } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import type { Chain } from "viem/chains";
import { getGeneralPaymasterInput } from "viem/zksync";
import { createPasskeyClient } from "zksync-sso-4337/client";

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
    const runtimeConfig = useRuntimeConfig();
    const paymasterAddress = runtimeConfig.public.contracts.paymaster as Address;
    const bundlerUrl = runtimeConfig.public.bundlerUrl as string;

    return createBundlerClient({
      client: publicClient,
      chain,
      transport: http(bundlerUrl),
      paymaster: {
        async getPaymasterData() {
          return {
            paymasterAndData: `${paymasterAddress}${getGeneralPaymasterInput({ innerInput: "0x" }).substring(2)}` as Hex,
          };
        },
        async getPaymasterStubData() {
          return {
            paymasterAndData: `${paymasterAddress}${getGeneralPaymasterInput({ innerInput: "0x" }).substring(2)}` as Hex,
          };
        },
      },
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

    const runtimeConfig = useRuntimeConfig();
    const bundlerClient = getBundlerClient();

    const client = createPasskeyClient({
      account: {
        address: address.value,
        validatorAddress: runtimeConfig.public.contracts.webauthnValidator as Address,
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
    const runtimeConfig = useRuntimeConfig();
    const bundlerClient = getBundlerClient();

    return createPasskeyClient({
      account: {
        address: addr,
        validatorAddress: runtimeConfig.public.contracts.webauthnValidator as Address,
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
