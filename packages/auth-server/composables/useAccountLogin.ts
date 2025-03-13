import { toHex } from "viem";
import { fetchAccount } from "zksync-sso/client";

export const useAccountLogin = (_chainId: MaybeRef<SupportedChainId>) => {
  const chainId = toRef(_chainId);
  const { login } = useAccountStore();
  const { getPublicClient } = useClientStore();

  const { inProgress: loginInProgress, error: accountLoginError, execute: loginToAccount } = useAsync(async () => {
    const client = getPublicClient({ chainId: chainId.value });

    const credential = await getPasskeyCredential();
    if (!credential) {
      throw new Error("No credential found");
    }

    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const { username, address, passkeyPublicKey } = await fetchAccount(client as any, {
        contracts: contractsByChain[chainId.value],
        uniqueAccountId: credential.id,
      });

      login({
        username,
        address,
        passkey: toHex(passkeyPublicKey),
      });
      return { success: true } as const;
    } catch {
      const { checkRecoveryRequest, executeRecovery, getRecovery } = useRecoveryGuardian();
      const recoveryRequest = await checkRecoveryRequest({ credentialId: credential.id });

      if (recoveryRequest?.ready) {
        const pendingRecoveryData = await getRecovery(recoveryRequest.accountAddress);
        if (!pendingRecoveryData) throw new Error("No pending recovery data found");

        await executeRecovery({
          accountAddress: recoveryRequest.accountAddress,
          credentialId: credential.id,
          rawPublicKey: pendingRecoveryData.rawPublicKey,
        });
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const { username, address, passkeyPublicKey } = await fetchAccount(client as any, {
          contracts: contractsByChain[chainId.value],
          uniqueAccountId: credential.id,
        });
        login({
          username,
          address,
          passkey: toHex(passkeyPublicKey),
        });
        return { success: true } as const;
      }

      if (recoveryRequest?.pendingRecovery) {
        return { success: false, recoveryRequest: {
          isReady: recoveryRequest.ready,
          remainingTime: recoveryRequest.remainingTime,
          accountAddress: recoveryRequest.accountAddress,
          guardianAddress: recoveryRequest.guardianAddress,
        },
        } as const;
      }

      throw new Error("Account not found");
    }
  });

  return {
    loginInProgress,
    accountLoginError,
    loginToAccount,
  };
};
