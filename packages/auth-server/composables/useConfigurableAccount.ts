import type { Address } from "viem";
import { fetchAccount } from "zksync-sso-4337";
import { WebAuthnValidatorAbi } from "zksync-sso-4337/abi";

export const useConfigurableAccount = () => {
  const { getPublicClient, getConfigurableClient, defaultChain, contractsByChain } = useClientStore();

  const { inProgress: getConfigurableAccountInProgress, error: getConfigurableAccountError, execute: getConfigurableAccount } = useAsync(async ({ address }: { address: Address }) => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const webauthnValidatorAddress = contractsByChain[defaultChain.id].webauthnValidator;

    // FIXME: events should be scoped to the origin domain
    // As well, this doesn't seem to be a reliable way of retrieving a `credentialId`
    // but works for now.
    const [events, removedEvents] = await Promise.all([
      publicClient.getContractEvents({
        address: webauthnValidatorAddress,
        abi: WebAuthnValidatorAbi,
        eventName: "PasskeyCreated",
        args: {
          keyOwner: address,
        },
        fromBlock: "earliest",
        strict: true,
      }),
      publicClient.getContractEvents({
        address: webauthnValidatorAddress,
        abi: WebAuthnValidatorAbi,
        eventName: "PasskeyRemoved",
        args: {
          keyOwner: address,
        },
        fromBlock: "earliest",
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

    const { credentialId } = await fetchAccount({
      client: publicClient,
      contracts: {
        webauthnValidator: webauthnValidatorAddress,
      },
      originDomain: window.location.origin,
      credentialId: latestEvent.args.credentialId,
    });

    return getConfigurableClient({
      chainId: defaultChain.id,
      address,
      credentialId,
    });
  });

  return {
    getConfigurableAccountInProgress,
    getConfigurableAccountError,
    getConfigurableAccount,
  };
};
