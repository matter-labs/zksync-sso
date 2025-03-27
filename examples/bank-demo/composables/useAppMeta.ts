import { useStorage } from "@vueuse/core";
import type { Address } from "viem";

export interface AppMetadata {
  name: string;
  icon: string | null;
  privateKey: string | null;
  credentialPublicKey: string | null;
  credentialId: string | null;
  cryptoAccountAddress: `0x${string}` | null;
  hasCompletedInitialTransfer: boolean;
  hasCompletedAaveStake: boolean;
}

export const useAppMeta = () => {
  const appMetaStorage = useStorage<AppMetadata>("app-meta", {
    name: "",
    icon: null,
    // k1 owner fallback if no passkey support
    privateKey: null,
    // Uint8Array from your Passkey
    credentialPublicKey: null,
    // base64 from your Passkey
    credentialId: null,
    // Account address that got created
    cryptoAccountAddress: null,
    // Have you purchased any ETH?
    hasCompletedInitialTransfer: false,
    // Have you staked any ETH?
    hasCompletedAaveStake: false,
  });

  const config = useRuntimeConfig();
  return {
    appMeta: appMetaStorage,
    userDisplay: "Jane Doe",
    userId: "jdoe",
    contracts: {
      accountFactory: config.public.accountFactory as Address,
      passkey: config.public.passkey as Address,
      session: config.public.session as Address,
    },
    deployerKey: config.public.bankDemoDeployerKey,
    aaveAddress: config.public.aaveAddress as Address,
    explorerUrl: config.public.explorerUrl,
  };
};
