import WalletKit, { type WalletKitTypes } from "@reown/walletkit";
import Core from "@walletconnect/core";
import type { JsonRpcTypes, SessionTypes } from "@walletconnect/types";
import { buildApprovedNamespaces } from "@walletconnect/utils";
import type { RpcRequestError } from "viem";
import { fromHex } from "viem";

import { supportedChains } from "./client";

export const useWalletConnectStore = defineStore("wallet-connect", () => {
  const { defaultChain, getClient } = useClientStore();
  const { address: accountAddress } = useAccountStore();

  const walletKit = ref<WalletKit | null>(null);
  const sessionProposal = ref<WalletKitTypes.SessionProposal | null>(null);
  const sessionRequest = ref<WalletKitTypes.SessionRequest | null>(null);
  const openSessions = ref<Record<string, SessionTypes.Struct>>({});

  const initialize = async () => {
    walletKit.value = await WalletKit.init({
      core,
      metadata: appKitMetadata,
    });
    updateOpenSessions();

    walletKit.value.on("session_proposal", async (proposal: WalletKitTypes.SessionProposal) => {
      // eslint-disable-next-line no-console
      console.log("[WC] Session proposal received", proposal);
      useWalletConnectStore().sessionProposal = proposal;
    });

    walletKit.value.on("session_request", async (req: WalletKitTypes.SessionRequest) => {
      // eslint-disable-next-line no-console
      console.log("[WC] Session request received", req);
      switch (req.params.request.method) {
        case "eth_sendRawTransaction":
        {
          const client = getClient({ chainId: defaultChain.id });
          try {
            const tx = await client.sendRawTransaction({
              serializedTransaction: req.params.request.params[0],
            });
            await walletKit.value!.respondSessionRequest({
              topic: req.topic,
              response: { id: req.id, result: tx, jsonrpc: "2.0" },
            });
          } catch (error) {
            // eslint-disable-next-line no-console
            console.error("[WC] Error sending raw transaction", error);
            await walletKit.value!.respondSessionRequest({
              topic: req.topic,
              response: { id: req.id, error: (error as RpcRequestError).cause as JsonRpcTypes.Error, jsonrpc: "2.0" },
            });
          }
          break;
        }
        case "eth_signTypedData_v4":
        case "eth_sendTransaction":
        case "personal_sign":
          useWalletConnectStore().sessionRequest = req;
          break;
        default:
          // eslint-disable-next-line no-console
          console.warn("[WC] Unsupported session request received", req);
      }
    });

    walletKit.value.on("session_delete", (req: WalletKitTypes.SessionDelete) => {
      // eslint-disable-next-line no-console
      console.log("[WC] Session deleted received", req);
      useWalletConnectStore().updateOpenSessions();
    });
  };

  const updateOpenSessions = () => {
    openSessions.value = walletKit.value ? walletKit.value.getActiveSessions() : {};
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
      data: data ?? "0x",
      value: value ?? "0",
    });
    await walletKit.value!.respondSessionRequest({
      topic: txData.topic,
      response: { id: txData.id, result: tx, jsonrpc: "2.0" },
    });
    return { hash: tx };
  };

  const sendRawTransaction = async (txData: WalletKitTypes.SessionRequest) => {
    const client = getClient({ chainId: defaultChain.id });
    const tx = await client.sendRawTransaction({
      serializedTransaction: txData.params.request.params[0],
    });
    await walletKit.value!.respondSessionRequest({
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
        name: "ZKsync",
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
    const signature = await client.signMessage({
      message,
    }) as `0x${string}`;

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
    sendRawTransaction,
    signTypedData,
    signPersonal,
    approveSessionProposal,
    rejectSessionProposal,
  };
});

function getSupportedNamespaces(accountAddress: string) {
  return {
    eip155: {
      chains: supportedChains.map((chain) => `eip155:${chain.id}`),
      methods: ["eth_sendTransaction", "eth_sendRawTransaction", "personal_sign", "eth_signTypedData_v4"],
      events: ["accountsChanged", "chainChanged"],
      accounts: supportedChains.map((chain) => `eip155:${chain.id}:${accountAddress}`),

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
