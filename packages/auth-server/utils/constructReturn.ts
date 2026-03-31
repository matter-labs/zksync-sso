import type { SessionSpec } from "zksync-sso-4337/client";
import type { AuthServerRpcSchema, ExtractReturnType } from "zksync-sso-4337/client-auth-server";

type ConstructReturnOptions = {
  address: `0x${string}`;
  chainId: number;
  session?: { sessionKey: `0x${string}`; sessionConfig: SessionSpec };
  prividiumMode: boolean;
  prividiumProxyUrl: string;
};

export const constructReturn = ({
  address,
  chainId,
  session,
  prividiumMode,
  prividiumProxyUrl,
}: ConstructReturnOptions): ExtractReturnType<"eth_requestAccounts", AuthServerRpcSchema> => {
  const { defaultChain, contracts } = useClientStore();
  return {
    account: {
      address,
      activeChainId: chainId,
      session: !session ? undefined : session,
    },
    chainsInfo: [{
      id: defaultChain.id,
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
      contracts,
      bundlerUrl: prividiumMode
        ? prividiumProxyUrl
        : contracts.bundlerUrl || "",
      prividiumMode,
    }],
  };
};
