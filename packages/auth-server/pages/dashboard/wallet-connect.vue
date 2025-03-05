<template>
  <div class="flex flex-col flex-1">
    <Card class="border-yellow-400 bg-yellow-50 dark:bg-yellow-950 dark:border-yellow-600">
      <h3 class="font-semibold text-yellow-800 mb-2 dark:text-yellow-200">
        You don't have any account recovery methods configured.
      </h3>
      <p class="text-yellow-700 mb-4 dark:text-yellow-300">
        Configure your account recovery methods to ensure your account is secure.
      </p>
      <ZkInput
        v-model="pairingUrl"
        placeholder="0x..."
        class="w-full text-left"
      />
      <Button
        class="bg-yellow-500 hover:bg-yellow-600 text-white dark:bg-yellow-600 dark:hover:bg-yellow-700 focus:bg-yellow-600 active:bg-yellow-700 disabled:bg-yellow-500 disabled:text-yellow-300 disabled:dark:bg-yellow-600 disabled:dark:hover:bg-yellow-700 dark:focus:bg-yellow-700 dark:active:bg-yellow-800 focus:ring-yellow-400 dark:focus:ring-yellow-800"
        @click="pairAccount"
      >
        Pair
      </Button>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { WalletKit, type WalletKitTypes } from "@reown/walletkit";
import { Core } from "@walletconnect/core";
import { buildApprovedNamespaces } from "@walletconnect/utils";
import type { Address } from "viem";

import Button from "~/components/zk/button.vue";
import Card from "~/components/zk/panel/card.vue";

const pairingUrl = defineModel<Address>();
const { defaultChain, getClient } = useClientStore();
const { address: accountAddress } = useAccountStore();
const client = getClient({ chainId: defaultChain.id });

const core = new Core({
  projectId: "4460d3c08eabdbc5f822eefaa2216f0a",
});

const metadataAppKit = {
  name: "zksync-sso",
  description: "AppKit Example",
  url: "http://localhost:3002", // origin must match your domain & subdomain
  icons: ["https://assets.reown.com/reown-profile-pic.png"],
};

const walletKit = await WalletKit.init({
  core, // <- pass the shared 'core' instance
  metadata: metadataAppKit,
});
const supportedNamespaces = {
  // You can add multiple namespaces like cosmos, near, solana, etc
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
core.pairing.events.on("pairing", (pairing) => {
  console.log("pairing", pairing);
});
core.pairing.events.on("pairing_expire", (pairing_expire) => {
  console.log("pairing_expire", pairing_expire);
});

walletKit.on("session_proposal", async ({ id, params }: WalletKitTypes.SessionProposal) => {
  try {
    const approvedNamespaces = buildApprovedNamespaces({
      proposal: params,
      supportedNamespaces,
    });

    await walletKit.approveSession({
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
walletKit.on("session_request", async (req: WalletKitTypes.SessionRequest) => {
  switch (req.params.request.method) {
    case "eth_signTypedData_v4":
      client.signTypedData(JSON.parse(req.params.request.params[1]));
      // Implement your logic to handle the session request here
      break;
    case "eth_sendTransaction":
      client.sendTransaction(JSON.parse(req.params.request.params[1]));
      // Implement your logic to handle the session request here
      break;
    case "personal_sign":
      // Implement your logic to handle the session request here
      break;
  }
  console.log("Req", req);
});

/**
 * Event listener for the "auth_request" event from walletKit.
 *
 * @param {WalletKitTypes.AuthRequest} authRequest - The auth request object containing details of the auth request.
 * @returns {Promise<void>} - A promise that resolves when the auth request handling is complete.
 */
walletKit.on("auth_request", async (authRequest) => {
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

const pairAccount = async () => {
  if (!pairingUrl.value) {
    return;
  }
  try {
    await walletKit.pair({ uri: pairingUrl.value });
  } catch (error) {
    console.log(error);
  }
};
</script>
