import type { Hex, Chain } from "viem";
import { createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

export const getDeployerClient = async (deployerKey: Hex) => {
  const config = useRuntimeConfig();
  return createWalletClient({
    account: privateKeyToAccount(deployerKey),
    chain: config.public.network as Chain,
    transport: http()
  });
};