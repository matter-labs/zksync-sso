import { useAppKitProvider } from "@reown/appkit/vue";
import { type Address, createPublicClient, createWalletClient, custom, type Hex, http, publicActions, walletActions } from "viem";
import { createBundlerClient } from "viem/account-abstraction";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createPasskeyClient } from "zksync-sso/client";

export type ChainContracts = {
  eoaValidator: Address;
  webauthnValidator: Address;
  sessionValidator: Address;
  guardianExecutor: Address;
  factory: Address;
  entryPoint?: Address;
  bundlerUrl?: string;
  beacon?: Address;
  testPaymaster?: Address;
  recovery?: Address;
  accountPaymaster?: Address;
  recoveryOidc?: Address;
  oidcKeyRegistry?: Address;
  oidcVerifier?: Address;
  passkey?: Address;
};

export const useClientStore = defineStore("client", () => {
  const runtimeConfig = useRuntimeConfig();
  const { address, credentialId } = storeToRefs(useAccountStore());
  const prividiumAuthStore = usePrividiumAuthStore();

  const defaultChain = useChain();

  // Build contracts from runtime config
  const contracts: ChainContracts = {
    factory: runtimeConfig.public.factoryAddress as Address,
    eoaValidator: runtimeConfig.public.eoaValidatorAddress as Address,
    webauthnValidator: runtimeConfig.public.webauthnValidatorAddress as Address,
    sessionValidator: runtimeConfig.public.sessionValidatorAddress as Address,
    guardianExecutor: runtimeConfig.public.guardianExecutorAddress as Address,
    entryPoint: (runtimeConfig.public.entryPointAddress as Address) || undefined,
    bundlerUrl: runtimeConfig.public.bundlerUrl || undefined,
    beacon: (runtimeConfig.public.beaconAddress as Address) || undefined,
    testPaymaster: (runtimeConfig.public.testPaymasterAddress as Address) || undefined,
    accountPaymaster: (runtimeConfig.public.accountPaymasterAddress as Address) || undefined,
    recoveryOidc: (runtimeConfig.public.recoveryOidcAddress as Address) || undefined,
    oidcKeyRegistry: (runtimeConfig.public.oidcKeyRegistryAddress as Address) || undefined,
    oidcVerifier: (runtimeConfig.public.oidcVerifierAddress as Address) || undefined,
    passkey: (runtimeConfig.public.passkeyAddress as Address) || undefined,
  };

  const blockExplorerUrl = runtimeConfig.public.blockExplorerUrl || "";
  const blockExplorerApiUrl = runtimeConfig.public.blockExplorerApiUrl || "";

  // Create transport with or without authentication based on Prividium mode
  const createTransport = () => {
    if (runtimeConfig.public.prividiumMode) {
      const prividiumTransport = prividiumAuthStore.getTransport();
      if (!prividiumTransport) {
        throw new Error("Prividium transport not available. User may need to authenticate.");
      }
      return prividiumTransport;
    }

    return http();
  };

  const getPublicClient = () => {
    return createPublicClient({
      chain: defaultChain,
      transport: createTransport(),
    });
  };

  const getBundlerClient = () => {
    const publicClient = getPublicClient();

    const bundlerTransport = runtimeConfig.public.prividiumMode
      ? (() => {
          const prividiumTransport = prividiumAuthStore.getTransport();
          if (!prividiumTransport) {
            throw new Error("Prividium transport not available. User may need to authenticate.");
          }
          return prividiumTransport;
        })()
      : http(contracts.bundlerUrl || "http://localhost:4337");

    return createBundlerClient({
      client: publicClient,
      chain: defaultChain,
      transport: bundlerTransport,
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

  const getClient = ({ usePaymaster = false, paymasterAddress }: { usePaymaster?: boolean; paymasterAddress?: string } = {}) => {
    if (!address.value) throw new Error("Address is not set");
    if (!credentialId.value) throw new Error("Credential ID is not set");
    const bundlerClient = getBundlerClient();

    const finalPaymasterAddress = paymasterAddress as Address | undefined ?? (usePaymaster ? contracts.testPaymaster : undefined);

    return createPasskeyClient({
      account: {
        address: address.value,
        validatorAddress: contracts.webauthnValidator,
        credentialId: credentialId.value,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain: defaultChain,
      transport: createTransport(),
      paymaster: finalPaymasterAddress,
    });
  };

  const getConfigurableClient = ({
    address,
    credentialId,
    usePaymaster = false,
  }: {
    address: Address;
    credentialId: Hex;
    usePaymaster?: boolean;
  }) => {
    const bundlerClient = getBundlerClient();

    return createPasskeyClient({
      account: {
        address,
        validatorAddress: contracts.webauthnValidator,
        credentialId,
        rpId: window.location.hostname,
        origin: window.location.origin,
      },
      bundlerClient,
      chain: defaultChain,
      transport: createTransport(),
      paymaster: usePaymaster ? contracts.testPaymaster : undefined,
    });
  };

  const getThrowAwayClient = () => {
    return createWalletClient({
      account: privateKeyToAccount(generatePrivateKey()),
      chain: defaultChain,
      transport: createTransport(),
    })
      .extend(publicActions)
      .extend(walletActions);
  };

  const getWalletClient = async () => {
    const accountProvider = useAppKitProvider("eip155");

    if (!accountProvider.walletProvider) throw new Error("No ethereum provider found");

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = accountProvider.walletProvider as any;
    const accounts = await (provider as { request: (args: { method: string }) => Promise<Address[]> }).request({
      method: "eth_requestAccounts",
    });

    return createWalletClient({
      chain: defaultChain,
      account: accounts[0],
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      transport: custom(provider as any),
    })
      .extend(publicActions)
      .extend(walletActions);
  };

  return {
    defaultChain,
    contracts,
    blockExplorerUrl,
    blockExplorerApiUrl,
    createTransport,
    getPublicClient,
    getBundlerClient,
    getClient,
    getThrowAwayClient,
    getWalletClient,
    getConfigurableClient,
  };
});
