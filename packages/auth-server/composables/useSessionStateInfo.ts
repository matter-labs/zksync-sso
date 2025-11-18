import { FetchError } from "ofetch";
import { erc20Abi, formatUnits, toFunctionSelector } from "viem";
import type { Address } from "viem/accounts";
import { LimitType, type SessionSpec, type SessionState } from "zksync-sso-4337/client";

// Type alias for compatibility
type SessionConfig = SessionSpec;

export const useSessionStateInfo = (
  _chainId: MaybeRef<SupportedChainId>,
  _sessionConfig: MaybeRef<SessionConfig>,
  _sessionState: MaybeRef<SessionState>,
) => {
  const chainId = toRef(_chainId);
  const sessionConfig = toRef(_sessionConfig);
  const sessionState = toRef(_sessionState);

  const { getToken, getTokenInProgress, fetchTokens } = (useTokensStore());

  const tokenAddresses = computed(() => {
    return [
      BASE_TOKEN_ADDRESS,
      ...sessionState.value.callParams.map((policy) => policy.target),
      ...sessionState.value.callValue.map((policy) => policy.target),
    ];
  });

  const { error: fetchTokensError, execute: fetchAllTokens } = useAsync(async () => {
    await fetchTokens({
      chainId: chainId.value,
      tokenAddresses: tokenAddresses.value,
      throwErrorAsserter: (e) => {
        // if (import.meta.dev) return false;
        if (e instanceof FetchError && e.statusCode === 404) return false;
        return true;
      },
    });
  });
  fetchAllTokens();

  const tokensLoading = computed(() => {
    return tokenAddresses.value.some((address) => getTokenInProgress({
      chainId: chainId.value,
      tokenAddress: address,
    }));
  });

  const spentByToken = computed(() => {
    const limits: { [tokenAddress: Address ]: bigint } = {};

    const createTokenEntry = (token: Address) => {
      if (!limits[token]) {
        limits[token] = 0n;
      }
    };
    const addSpend = (token: Address, amount: bigint) => {
      createTokenEntry(token);
      limits[token] += amount;
    };

    // TODO: handle other types
    if (sessionConfig.value.feeLimit.limitType === LimitType.Lifetime) {
      addSpend(BASE_TOKEN_ADDRESS, sessionConfig.value.feeLimit.limit - sessionState.value.feesRemaining);
    }

    // Handle transfers
    sessionConfig.value.transferPolicies.forEach((policy, index) => {
      const policyState = sessionState.value.transferValue[index];
      if (!policyState) return;
      if (policy.valueLimit.limitType !== LimitType.Lifetime) return; // TODO: handle other types
      const diff = policy.valueLimit.limit - policyState.remaining;
      addSpend(BASE_TOKEN_ADDRESS, diff);
    });

    // Handle call value
    sessionConfig.value.callPolicies.forEach((policy, index) => {
      const policyValueState = sessionState.value.callValue[index];
      if (!policyValueState) return;
      if (policy.valueLimit.limitType !== LimitType.Lifetime) return; // TODO: handle other types
      const diff = policy.valueLimit.limit - policyValueState.remaining;
      addSpend(BASE_TOKEN_ADDRESS, diff);
    });

    // Handle call arguments
    sessionConfig.value.callPolicies.forEach((policy) => {
      let constraintIndex = 0;
      policy.constraints.forEach((constraint) => {
        const constraintState = sessionState.value.callParams[constraintIndex];
        constraintIndex++;
        if (!constraintState) return;
        if (constraint.limit.limitType !== LimitType.Lifetime) return; // TODO: handle other types

        const token = getToken({
          chainId: chainId.value,
          tokenAddress: policy.target,
        });
        if (!token) return;

        const transferAbi = erc20Abi.find((e) => e.type === "function" && e.name === "transfer")!;
        const approveAbi = erc20Abi.find((e) => e.type === "function" && e.name === "approve")!;
        const isTransferSelector = policy.selector === toFunctionSelector(transferAbi);
        const isApproveSelector = policy.selector === toFunctionSelector(approveAbi);
        if (!isTransferSelector && !isApproveSelector) return;

        const diff = constraint.limit.limit - constraintState.remaining;
        addSpend(token.address, diff);
      });
    });

    return limits;
  });

  const spendLimitTokens = computed(() => {
    return Object.entries(spentByToken.value)
      .filter(([, amount]) => amount !== 0n)
      .map(([tokenAddress, amount]) => {
        const token = getToken({
          chainId: chainId.value,
          tokenAddress: tokenAddress as Address,
        });
        if (!token) return null;
        return {
          token,
          amount,
        };
      })
      .filter((e) => !!e);
  });

  const totalUsd = computed(() => (spendLimitTokens.value || []).reduce((acc, item) => {
    if (!item.token.price) return acc;
    const formattedTokenAmount = formatUnits(BigInt(item.amount), item.token.decimals);
    return acc + (parseFloat(formattedTokenAmount) * item.token.price);
  }, 0));

  return {
    fetchTokensError,
    tokensLoading,
    spendLimitTokens,
    totalUsd,
  };
};
