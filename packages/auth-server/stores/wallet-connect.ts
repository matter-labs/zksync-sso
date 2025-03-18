import WalletKit, { type WalletKitTypes } from "@reown/walletkit";
import Core from "@walletconnect/core";
import type { SessionTypes } from "@walletconnect/types";
import { buildApprovedNamespaces } from "@walletconnect/utils";
import { fromHex } from "viem";

export const useWalletConnectStore = defineStore("wallet-connect", () => {
  const { defaultChain, getClient } = useClientStore();
  const { address: accountAddress } = useAccountStore();
  const walletKit = ref<WalletKit | null>(null);

  const sessionRequest = ref<WalletKitTypes.SessionRequest | null>(null);
  const openSessions = ref<[string, SessionTypes.Struct][]>([]);

  const core = new Core({
    projectId: "4460d3c08eabdbc5f822eefaa2216f0a",
  });

  const metadataAppKit = {
    name: "zksync-sso",
    description: "AppKit Example",
    url: "http://localhost:3002", // origin must match your domain & subdomain
    icons: ["https://assets.reown.com/reown-profile-pic.png"],
  };

  watchEffect(async () => {
    walletKit.value = await WalletKit.init({
      core, // <- pass the shared 'core' instance
      metadata: metadataAppKit,
    });

    const sessions = walletKit.value.getActiveSessions();
    openSessions.value = Object.values(sessions);

    walletKit.value.on("session_proposal", async ({ id, params }: WalletKitTypes.SessionProposal) => {
      try {
        const approvedNamespaces = buildApprovedNamespaces({
          proposal: params,
          supportedNamespaces,
        });

        await walletKit.value!.approveSession({
          id: id as number,
          namespaces: approvedNamespaces,
        });
      } catch (error) {
        console.error(error);
      }
      console.log("session_proposal", params); // Implement your logic to handle the session proposal here
    });

    /**
       * Event listener for the "session_request" event from walletKit.
       *
       * @param {WalletKitTypes.SessionRequest} req - The session request object containing details of the session request.
       * @returns {Promise<void>} - A promise that resolves when the session request handling is complete.
       */
    walletKit.value.on("session_request", async (req: WalletKitTypes.SessionRequest) => {
      const client = getClient({ chainId: defaultChain.id });
      switch (req.params.request.method) {
        case "eth_signTypedData_v4":
        case "eth_sendTransaction":
        case "personal_sign":
          sessionRequest.value = req;
          break;
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
      }
      console.log("Req", req);
    });

    /**
       * Event listener for the "auth_request" event from walletKit.
       *
       * @param {WalletKitTypes.AuthRequest} authRequest - The auth request object containing details of the auth request.
       * @returns {Promise<void>} - A promise that resolves when the auth request handling is complete.
       */
    walletKit.value!.on("auth_request", async (authRequest) => {
      const { verifyContext } = authRequest;
      const validation = verifyContext.verified.validation; // can be VALID, INVALID or UNKNOWN
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const origin = verifyContext.verified.origin; // the actual verified origin of the request
      const isScam = verifyContext.verified.isScam; // true if the domain is flagged as malicious

      // if the domain is flagged as malicious, you should warn the user as they may lose their funds - check the `Threat` case for more info
      if (isScam) {
        // show a warning screen to the user
        // and proceed only if the user accepts the risk
      }

      switch (validation) {
        case "VALID":
          // proceed with the request - check the `Domain match` case for more info
          break;
        case "INVALID":
          // show a warning dialog to the user - check the `Mismatch` case for more info
          // and proceed only if the user accepts the risk
          break;
        case "UNKNOWN":
          // show a warning dialog to the user - check the `Unverified` case for more info
          // and proceed only if the user accepts the risk
          break;
      }
    });
  });
  const supportedNamespaces = {
    // You can add multiple namespaces like cosmos, near, solana, etc
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
  core.pairing.events.on("pairing", (pairing) => {
    console.log("pairing", pairing);
  });
  core.pairing.events.on("pairing_expire", (pairing_expire) => {
    console.log("pairing_expire", pairing_expire);
  });

  const pairAccount = async (pairingUrl: Ref<string>) => {
    if (!pairingUrl.value) {
      return;
    }
    try {
      await walletKit.value!.pair({ uri: pairingUrl.value });
    } catch (error) {
      console.log(error);
    }
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
  const closeSession = async (topic: string) => {
    walletKit.value!.disconnectSession({
      topic: topic,
      reason: { code: 4100, message: "Session closed by user" },
    });
    const sessions = walletKit.value!.getActiveSessions();
    openSessions.value = Object.values(sessions);
  };

  return {
    pairAccount,
    sendTransaction,
    signTypedData,
    signPersonal,
    sessionRequest,
    openSessions,
    closeSession,
  };
});
