import type { Address } from "viem";
import { fetchAccount } from "zksync-sso/client";
import { WebAuthnValidatorAbi } from "zksync-sso-4337/abi";

export const useConfigurableAccount = () => {
  const { getPublicClient, getConfigurableClient, defaultChain } = useClientStore();

  const { inProgress: getConfigurableAccountInProgress, error: getConfigurableAccountError, execute: getConfigurableAccount } = useAsync(async ({ address }: { address: Address }) => {
    const publicClient = getPublicClient({ chainId: defaultChain.id });
    const factoryAddress = contractsByChain[defaultChain.id].accountFactory;

    // FIXME: events should be scoped to the origin domain
    // As well, this doesn't seem to be a reliable way of retrieving a `credentialId`
    // but works for now.
    const [events, removedEvents] = await Promise.all([
      publicClient.getContractEvents({
        address: factoryAddress,
        abi: WebAuthnValidatorAbi,
        eventName: "PasskeyCreated",
        args: {
          keyOwner: address,
        },
        fromBlock: "earliest",
        strict: true,
      }),
      publicClient.getContractEvents({
        address: factoryAddress,
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

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const { username, passkeyPublicKey } = await fetchAccount(publicClient as any, {
      contracts: contractsByChain[defaultChain.id],
      uniqueAccountId: latestEvent.args.credentialId,
    });

    return getConfigurableClient({
      chainId: defaultChain.id,
      address,
      credentialPublicKey: passkeyPublicKey,
      username,
    });
  });

  return {
    getConfigurableAccountInProgress,
    getConfigurableAccountError,
    getConfigurableAccount,
  };
};
