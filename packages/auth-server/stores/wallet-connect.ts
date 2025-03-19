import WalletKit, { type WalletKitTypes } from "@reown/walletkit";
import Core from "@walletconnect/core";
import type { SessionTypes } from "@walletconnect/types";
import { buildApprovedNamespaces } from "@walletconnect/utils";
import { fromHex } from "viem";

export const useWalletConnectStore = defineStore("wallet-connect", () => {
  const { defaultChain, getClient } = useClientStore();
  const { address: accountAddress } = useAccountStore();

  const walletKit = ref<WalletKit | null>(null);
  const sessionProposal = ref<WalletKitTypes.SessionProposal | null>(null);
  const sessionRequest = ref<WalletKitTypes.SessionRequest | null>(null);
  const openSessions = ref<SessionTypes.Struct[]>([]);

  const initialize = async () => {
    walletKit.value = await WalletKit.init({
      core,
      metadata: appKitMetadata,
    });
    updateOpenSessions();

    walletKit.value.on("session_proposal", async (proposal: WalletKitTypes.SessionProposal) => {
      console.log("[WC] Session proposal received", proposal);
      useWalletConnectStore().sessionProposal = proposal;
    });

    walletKit.value.on("session_request", async (req: WalletKitTypes.SessionRequest) => {
      const client = getClient({ chainId: defaultChain.id });
      switch (req.params.request.method) {
        case "eth_sendRawTransaction":
        {
          const tx = await client.sendRawTransaction({
            serializedTransaction: req.params.request.params[0],
          });
          walletKit.value!.respondSessionRequest({
            topic: req.topic,
            response: { id: req.id, result: tx, jsonrpc: "2.0" },
          });
          break;
        }
        case "eth_signTypedData_v4":
        case "eth_sendTransaction":
        case "personal_sign":
          console.log("[WC] Session request received", req);
          useWalletConnectStore().sessionRequest = req;
          break;
        default:
          console.warn("[WC] Unsupported session request received", req);
      }
    });

    walletKit.value.on("session_delete", (req: WalletKitTypes.SessionDelete) => {
      console.log("[WC] Session deleted received", req);
      useWalletConnectStore().updateOpenSessions();
    });
  };

  const updateOpenSessions = () => {
    openSessions.value = walletKit.value ? Object.values(walletKit.value.getActiveSessions()) : [];
  };

  const approveSessionProposal = async () => {
    if (!sessionProposal.value) {
      return;
    }

    const approvedNamespaces = buildApprovedNamespaces({
      proposal: sessionProposal.value.params,
      supportedNamespaces: getSupportedNamespaces(accountAddress!),
    });
    await walletKit.value!.approveSession({
      id: sessionProposal.value.id,
      namespaces: approvedNamespaces,
    });
    sessionProposal.value = null;
    updateOpenSessions();
  };

  const rejectSessionProposal = async () => {
    if (walletKit.value === null || sessionProposal.value === null) {
      return;
    }

    await walletKit.value!.rejectSession({
      id: sessionProposal.value.id,
      reason: { code: 4100, message: "Session rejected by user" },
    });
    sessionProposal.value = null;
  };

  const pairAccount = async (uri: string) => {
    await walletKit.value!.pair({ uri });
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
    const { types, primaryType, message, domain } = JSON.parse(txData.params.request.params[1]);
    const signature = await client.signTypedData({
      domain: domain ?? {
        name: "zkSync",
        version: "2",
        chainId: defaultChain.id,
      },
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

  const signPersonal = async (txData: WalletKitTypes.SessionRequest) => {
    const client = getClient({ chainId: defaultChain.id });
    const message = fromHex(txData.params.request.params[0], "string");
    console.log(message);
    const signature = await client.signMessage({
      message,
    }) as `0x${string}`;

    console.log(signature);
    const isValid = await client.verifyMessage({
      address: txData.params.request.params[1],
      message,
      signature,
    });
    console.log(isValid);

    walletKit.value!.respondSessionRequest({
      topic: txData.topic,
      response: { id: txData.id, result: signature, jsonrpc: "2.0" },
    });
    return { signature };
  };

  return {
    walletKit,
    sessionProposal,
    sessionRequest,
    openSessions,
    initialize,
    updateOpenSessions,
    pairAccount,
    closeSession,
    sendTransaction,
    signTypedData,
    signPersonal,
    approveSessionProposal,
    rejectSessionProposal,
  };
});

function getSupportedNamespaces(accountAddress: string) {
  return {
    eip155: {
      chains: ["eip155:260", "eip155:300"],
      methods: ["eth_sendTransaction", "eth_sendRawTransaction", "personal_sign", "eth_signTypedData_v4"],
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
