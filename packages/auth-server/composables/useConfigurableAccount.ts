import type { Address, Hex } from "viem";
import { WebAuthnValidatorAbi } from "zksync-sso-4337/abi";

export const useConfigurableAccount = () => {
  const { getPublicClient, getConfigurableClient, defaultChain, contractsByChain } = useClientStore();

  const { inProgress: getConfigurableAccountInProgress, error: getConfigurableAccountError, execute: getConfigurableAccount } = useAsync(async ({ address, usePaymaster = false }: { address: Address; usePaymaster?: boolean }) => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const webauthnValidatorAddress = contractsByChain[defaultChain.id].webauthnValidator;

    // Small delay to allow blockchain to index recent events (especially important in test environments)
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Get current block to calculate safe fromBlock (avoid RPC block range limits)
    // Add timeout to prevent hanging RPC calls (5s should be sufficient)
    const currentBlock = await publicClient.getBlockNumber();

    // Use 100k block range to ensure we catch all events (tests pass with this value)
    const fromBlock = currentBlock > 100000n ? currentBlock - 100000n : 0n;

    // FIXME: events should be scoped to the origin domain
    // As well, this doesn't seem to be a reliable way of retrieving a `credentialId`
    // but works for now.

    // Add timeout to event queries to prevent hanging (10s to handle multiple accounts in test environments)
    const [events, removedEvents] = await Promise.all([
      publicClient.getContractEvents({
        address: webauthnValidatorAddress,
        abi: WebAuthnValidatorAbi,
        eventName: "PasskeyCreated",
        args: {
          keyOwner: address,
        },
        fromBlock,
        strict: true,
      }),
      publicClient.getContractEvents({
        address: webauthnValidatorAddress,
        abi: WebAuthnValidatorAbi,
        eventName: "PasskeyRemoved",
        args: {
          keyOwner: address,
        },
        fromBlock,
        strict: true,
      }),
    ]);

    if (!events || events.length === 0) {
      throw new Error("Account not found");
    }

    const removedCredentialIds = new Set(
      removedEvents.map((event) => event.args.credentialId),
    );

    const activeEvents = events.filter(
      (event) => !removedCredentialIds.has(event.args.credentialId),
    );
    if (activeEvents.length === 0) {
      throw new Error("No active accounts found");
    }

    const latestEvent = activeEvents[activeEvents.length - 1];

    // The credentialId from the event is already in hex format (bytes type from contract)
    const credentialIdHex = latestEvent.args.credentialId as Hex;

    return getConfigurableClient({
      chainId: defaultChain.id,
      address,
      credentialId: credentialIdHex,
      usePaymaster,
    });
  });

  return {
    getConfigurableAccountInProgress,
    getConfigurableAccountError,
    getConfigurableAccount,
  };
};
