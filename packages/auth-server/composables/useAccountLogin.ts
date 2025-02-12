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
      const { checkRecoveryRequest } = useRecoveryGuardian();
      const recoveryRequest = await checkRecoveryRequest(credential.id);
      if (recoveryRequest) {
        return {
          success: false,
          recoveryRequest: {
            account: recoveryRequest[0],
            isReady: recoveryRequest[1],
            remainingTime: recoveryRequest[2],
          },
        } as const;
      } else {
        return { success: false } as const;
      }
    }
  });

  return {
    loginInProgress,
    accountLoginError,
    loginToAccount,
  };
};
