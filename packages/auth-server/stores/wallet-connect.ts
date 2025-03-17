import WalletKit, { type WalletKitTypes } from "@reown/walletkit";
import Core from "@walletconnect/core";
import type { SessionTypes } from "@walletconnect/types";
import { buildApprovedNamespaces } from "@walletconnect/utils";

export const useWalletConnectStore = defineStore("wallet-connect", () => {
  const { defaultChain, getClient } = useClientStore();
  const { address: accountAddress } = useAccountStore();

  const walletKit = ref<WalletKit | null>(null);
  const sessionRequest = ref<WalletKitTypes.SessionRequest | null>(null);
  const openSessions = ref<SessionTypes.Struct[]>([]);

  const initialize = async () => {
    walletKit.value = await WalletKit.init({
      core,
      metadata: appKitMetadata,
    });

    updateOpenSessions();

    walletKit.value.on("session_proposal", async ({ id, params }: WalletKitTypes.SessionProposal) => {
      const approvedNamespaces = buildApprovedNamespaces({
        proposal: params,
        supportedNamespaces: getSupportedNamespaces(accountAddress!),
      });
      await walletKit.value!.approveSession({
        id,
        namespaces: approvedNamespaces,
      });
    });

    walletKit.value.on("session_request", async (req: WalletKitTypes.SessionRequest) => {
      switch (req.params.request.method) {
        case "eth_signTypedData_v4":
          sessionRequest.value = req;
          // client.signTypedData(JSON.parse(req.params.request.params[1]));
          // Implement your logic to handle the session request here
          break;
        case "eth_sendTransaction":
          sessionRequest.value = req;
          // client.sendTransaction(JSON.parse(req.params.request.params[1]));
          // Implement your logic to handle the session request here
          break;
        case "personal_sign":
          sessionRequest.value = req;
          // Implement your logic to handle the session request here
          break;
      }
      console.log("Req", req);
    });
  };

  const updateOpenSessions = () => {
    if (!walletKit.value) return;
    openSessions.value = Object.values(walletKit.value.getActiveSessions());
  };

  const pairAccount = async (uri: string) => {
    await walletKit.value!.pair({ uri });
    updateOpenSessions();
  };

  const closeSession = async (topic: string) => {
    await walletKit.value!.disconnectSession({
      topic: topic,
      reason: { code: 4100, message: "Session closed by user" },
    });
    updateOpenSessions();
  };

  const sendTransaction = async (txData: WalletKitTypes.SessionRequest) => {
    const client = getClient({ chainId: defaultChain.id });
    const { to, data, value } = txData.params.request.params[0];
    const tx = await client.sendTransaction({
      to,
      data,
      value,
    });

    walletKit.value!.respondSessionRequest({
      topic: txData.topic,
      response: { id: txData.id, result: tx, jsonrpc: "2.0" },
    });
    return { hash: tx };
  };

  const signTypedData = async (txData: WalletKitTypes.SessionRequest) => {
    const client = getClient({ chainId: defaultChain.id });
    const { types, primaryType, message } = JSON.parse(txData.params.request.params[1]);
    const signature = await client.signTypedData({
      types,
      primaryType,
      message,
    });

    walletKit.value!.respondSessionRequest({
      topic: txData.topic,
      response: { id: txData.id, result: signature, jsonrpc: "2.0" },
    });
    return { signature };
  };

  return {
    walletKit,
    sessionRequest,
    openSessions,
    initialize,
    updateOpenSessions,
    pairAccount,
    closeSession,
    sendTransaction,
    signTypedData,
  };
});

function getSupportedNamespaces(accountAddress: string) {
  return {
    eip155: {
      chains: ["eip155:260", "eip155:300"],
      methods: ["eth_sendTransaction", "personal_sign", "eth_signTypedData_v4"],
      events: ["accountsChanged", "chainChanged"],
      // Replace wallet address with your address
      accounts: [
        `eip155:260:${accountAddress}`,
        `eip155:300:${accountAddress}`,
      ],
    },
  };
}

const core = new Core({
  projectId: "4460d3c08eabdbc5f822eefaa2216f0a",
});

const appKitMetadata = {
  name: "zksync-sso",
  description: "AppKit Example",
  url: "http://localhost:3002",
  icons: ["https://assets.reown.com/reown-profile-pic.png"],
};
