import type { AuthServerRpcSchema, ExtractReturnType, SessionSpec } from "zksync-sso-4337/client";

// Type alias for compatibility
type SessionConfig = SessionSpec;

export const constructReturn = (address: `0x${string}`, chainId: number, session?: { sessionKey: `0x${string}`; sessionConfig: SessionConfig }): ExtractReturnType<"eth_requestAccounts", AuthServerRpcSchema> => {
  return {
    account: {
      address,
      activeChainId: chainId,
      session: !session ? undefined : session,
    },
    chainsInfo: supportedChains.map((chain) => ({
      id: chain.id,
      capabilities: {
        paymasterService: {
          supported: true,
        },
        atomicBatch: {
          supported: true,
        },
        auxiliaryFunds: {
          supported: true,
        },
      },
      contracts: contractsByChain[chain.id],
    })),
  };
};
