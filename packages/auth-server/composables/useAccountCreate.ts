import type { Hex } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import { waitForTransactionReceipt } from "viem/actions";
import type { SessionSpec } from "zksync-sso-4337/client";
import { getAccountAddressFromLogs, prepareDeploySmartAccount } from "zksync-sso-4337/client";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>, prividiumMode = false) => {
  const chainId = toRef(_chainId);
  const { getThrowAwayClient } = useClientStore();
  const { registerPasskey } = usePasskeyRegister();
  const { fetchAddressAssociationMessage, associateAddress, deleteAddressAssociation } = usePrividiumAddressAssociation();

  const { inProgress: registerInProgress, error: createAccountError, execute: createAccount } = useAsync(async (session?: Omit<SessionSpec, "signer">) => {
    const result = await registerPasskey();
    if (!result) {
      throw new Error("Failed to register passkey");
    }
    const { credentialPublicKey, credentialId } = result;

    // TODO: Session support during deployment - to be implemented
    // For now, sessions can be added after deployment
    let sessionData: SessionSpec | undefined;
    const sessionKey = generatePrivateKey();
    const signer = privateKeyToAddress(sessionKey);
    if (session) {
      sessionData = {
        ...session,
        signer: signer,
      };
    }

    // EOA owner for initial deployment signing
    const ownerKey = generatePrivateKey();
    const ownerAddress = privateKeyToAddress(ownerKey);

    const deployerClient = getThrowAwayClient({ chainId: chainId.value });

    // For Prividium mode, associate temporary address before deployment
    if (prividiumMode) {
      const { message } = await fetchAddressAssociationMessage(deployerClient.account.address);
      const signature = await deployerClient.signMessage({ message });
      await associateAddress(deployerClient.account.address, message, signature);
    }

    const chainContracts = contractsByChain[chainId.value];

    // Prepare deployment transaction using sdk-4337
    const { transaction } = prepareDeploySmartAccount({
      contracts: {
        factory: chainContracts.factory,
        webauthnValidator: chainContracts.webauthnValidator,
      },
      passkeySigners: [{
        credentialId: credentialId as Hex,
        publicKey: credentialPublicKey,
        originDomain: typeof window !== "undefined" ? window.location.origin : "http://localhost",
      }],
      eoaSigners: [ownerAddress],
      userId: credentialId, // Use credential ID as unique user ID
    });

    // Send deployment transaction
    const hash = await deployerClient.sendTransaction(transaction);
    const receipt = await waitForTransactionReceipt(deployerClient, { hash });

    // Extract deployed account address from logs
    const address = getAccountAddressFromLogs(receipt.logs);
    if (!address) {
      throw new Error("Failed to extract account address from deployment logs");
    }

    // Clean up temporary association for Prividium mode
    if (prividiumMode) {
      await deleteAddressAssociation(deployerClient.account.address).catch((err) => {
        // Ignore errors on cleanup
        // eslint-disable-next-line no-console
        console.warn("Failed to delete temporary address association:", err);
      });
    }

    return {
      address,
      chainId: chainId.value,
      sessionKey: session ? sessionKey : undefined,
      signer,
      sessionConfig: sessionData,
      credentialId,
      credentialPublicKey,
    };
  });

  return {
    registerInProgress,
    createAccount,
    createAccountError,
  };
};
