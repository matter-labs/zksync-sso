import { toHex } from "viem";
import { generatePrivateKey, privateKeyToAddress } from "viem/accounts";
import { deployModularAccount } from "zksync-sso/client";
import type { SessionConfig } from "zksync-sso/utils";

export const useAccountCreate = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { login } = useAccountStore();
  const { getThrowAwayClient } = useClientStore();
  const { registerPasskey } = usePasskeyRegister();

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

    login({
      username: credentialId,
      address: deployedAccount.address,
      passkey: toHex(credentialPublicKey),
    });

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
