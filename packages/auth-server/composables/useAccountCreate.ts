import { toHex, zeroAddress } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import { deployModularAccount } from "zksync-sso/client";
import type { SessionConfig } from "zksync-sso/utils";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { $config } = useNuxtApp();
  const { login } = useAccountStore();
  const { getClient, getThrowAwayClient } = useClientStore();
  const { registerPasskey } = usePasskeyRegister();
  const { fetchAddressAssociationMessage, associateAddress, deleteAddressAssociation } = usePrividiumAddressAssociation();

  const isPrividiumMode = computed(() => $config.public.prividiumMode);

  const { inProgress: registerInProgress, error: createAccountError, execute: createAccount } = useAsync(async (session?: Omit<SessionConfig, "signer">) => {
    const result = await registerPasskey();
    if (!result) {
      throw new Error("Failed to register passkey");
    }
    const { credentialPublicKey, credentialId } = result;

    let sessionData: SessionConfig | undefined;
    const sessionKey = generatePrivateKey();
    const signer = privateKeyToAddress(sessionKey);
    if (session) {
      sessionData = {
        ...session,
        signer: signer,
      };
    }

    // Don't yet want this to be imported as part of the setup process
    const ownerKey = generatePrivateKey();
    const ownerAddress = privateKeyToAddress(ownerKey);

    const deployerClient = getThrowAwayClient({ chainId: chainId.value });

    if (isPrividiumMode.value) {
      const { message } = await fetchAddressAssociationMessage(deployerClient.account.address);
      const signature = await deployerClient.signMessage({ message });
      await associateAddress(deployerClient.account.address, message, signature);
    }

    const chainContracts = contractsByChain[chainId.value];
    const deployedAccount = await deployModularAccount(deployerClient, {
      accountFactory: chainContracts.accountFactory,
      passkeyModule: {
        location: chainContracts.passkey,
        credentialId,
        credentialPublicKey,
      },
      paymaster: {
        location: chainContracts.accountPaymaster,
      },
      uniqueAccountId: credentialId,
      sessionModule: {
        location: chainContracts.session,
        initialSession: sessionData,
      },
      owners: [ownerAddress],
      installNoDataModules: [chainContracts.recovery],
    });

    if (isPrividiumMode.value) {
      await deleteAddressAssociation(deployerClient.account.address).catch((err) => {
        // Ignore errors on cleanup
        // eslint-disable-next-line no-console
        console.warn("Failed to delete address association:", err);
      });
    }

    login({
      username: credentialId,
      address: deployedAccount.address,
      passkey: toHex(credentialPublicKey),
    });

    if (isPrividiumMode.value) {
      const passkeyClient = getClient({ chainId: chainId.value });
      const { message } = await fetchAddressAssociationMessage(passkeyClient.account.address);
      const domain = {
        name: "AddressAssociationVerifier",
        version: "1.0.0",
        chainId: chainId.value,
        verifyingContract: zeroAddress,
      } as const;
      const signature = await passkeyClient.signTypedData({
        domain: {
          ...domain,
          salt: undefined, // Otherwise the signature verification fails (todo: figure out why)
        },
        types: {
          AddressAssociation: [
            { name: "message", type: "string" },
          ],
        },
        primaryType: "AddressAssociation",
        message: {
          message,
        },
      });
      await associateAddress(passkeyClient.account.address, message, signature);
    }

    return {
      address: deployedAccount.address,
      chainId: chainId.value,
      sessionKey: session ? sessionKey : undefined,
      signer,
      sessionConfig: sessionData,
    };
  });

  return {
    registerInProgress,
    createAccount,
    createAccountError,
  };
};
