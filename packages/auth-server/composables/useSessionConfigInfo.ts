import { FetchError } from "ofetch";
import { erc20Abi, formatUnits, toFunctionSelector } from "viem";
import type { Address } from "viem/accounts";
import { type Limit, LimitType, LimitUnlimited, type SessionConfig } from "zksync-sso/utils";

export const useSessionConfigInfo = (
  _chainId: MaybeRef<SupportedChainId>,
  _sessionConfig: MaybeRef<Omit<SessionConfig, "signer">>,
  now: Ref<Date>,
) => {
  const chainId = toRef(_chainId);
  const sessionConfig = toRef(_sessionConfig);

  const { getToken, getTokenInProgress, fetchTokens } = (useTokensStore());

  const onchainActionsCount = computed(() => {
    return sessionConfig.value.callPolicies.length + sessionConfig.value.transferPolicies.length;
  });

  const tokenAddresses = computed(() => {
    return [BASE_TOKEN_ADDRESS, ...sessionConfig.value.callPolicies.map((policy) => policy.target)];
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

  const calculateMaxPeriodSpend = (args: { sessionStartsAt: Date; sessionExpiresAt: bigint; limitPerPeriod: bigint; period: bigint }) => {
    // `limitPerPeriod` has cumulative effect, and every `period` seconds it resets
    const startDate = args.sessionStartsAt;
    const expiryDate = bigintDateToDate(args.sessionExpiresAt);
    const period = Number(args.period) * 1000;
    // 1n means initial limit
    const limitResetsCount = 1n + BigInt(Math.floor((expiryDate.getTime() - startDate.getTime()) / period));
    return limitResetsCount * args.limitPerPeriod;
  };

  const limitsByToken = computed(() => {
    const limits: { [tokenAddress: Address ]: bigint | "unlimited" } = {};

    const createTokenEntry = (token: Address) => {
      if (!limits[token]) {
        limits[token] = 0n;
      }
    };
    const addLifetimeLimit = (token: Address, limit: bigint | "unlimited") => {
      createTokenEntry(token);
      if (limits[token] === "unlimited") return;
      if (limit === "unlimited") {
        limits[token] = "unlimited";
      } else {
        limits[token] += limit;
      }
    };
    const addPeriodLimit = (token: Address, limit: bigint, period: bigint) => {
      createTokenEntry(token);
      const maxSpend = calculateMaxPeriodSpend({
        sessionStartsAt: now.value,
        sessionExpiresAt: sessionConfig.value.expiresAt,
        limitPerPeriod: limit,
        period,
      });
      addLifetimeLimit(token, maxSpend);
    };
    const addLimit = (token: Address, limit: Limit) => {
      if (limit.limitType === LimitType.Lifetime) {
        addLifetimeLimit(token, limit.limit);
      } else if (limit.limitType === LimitType.Allowance) {
        addPeriodLimit(token, limit.limit, limit.period);
      } else if (limit.limitType === LimitType.Unlimited) {
        addLifetimeLimit(token, "unlimited");
      }
    };

    addLimit(BASE_TOKEN_ADDRESS, sessionConfig.value.feeLimit);

    for (const policy of sessionConfig.value.transferPolicies) {
      addLimit(BASE_TOKEN_ADDRESS, policy.valueLimit);
    }
    for (const policy of sessionConfig.value.callPolicies) {
      addLimit(BASE_TOKEN_ADDRESS, policy.valueLimit);
      const token = getToken({
        chainId: chainId.value,
        tokenAddress: policy.target,
      });
      if (!token) continue;

      const transferAbi = erc20Abi.find((e) => e.type === "function" && e.name === "transfer")!;
      const approveAbi = erc20Abi.find((e) => e.type === "function" && e.name === "approve")!;
      const isTransferSelector = policy.selector === toFunctionSelector(transferAbi);
      const isApproveSelector = policy.selector === toFunctionSelector(approveAbi);
      if (!isTransferSelector && !isApproveSelector) continue;

      const amountConstraints = policy.constraints.filter((constraint) => {
        const functionAbi = isTransferSelector ? transferAbi : approveAbi;
        if (constraint.index === BigInt(functionAbi.inputs.findIndex((e) => e.name === "amount"))) return true;
      });
      if (!amountConstraints.length) {
        addLimit(policy.target, LimitUnlimited);
        continue;
      }

      // The goal is to find max amount that could be spent within that call policy
      // To find max amount, we need to find the minimum of all constraints, since all of them should be satisfied
      const maxAmount: bigint | "unlimited" = amountConstraints.reduce((acc, constraint) => {
        switch (constraint.limit.limitType) {
          case LimitType.Unlimited: {
            if (acc === "unlimited") return "unlimited";
            return acc;
          }
          case LimitType.Allowance: {
            const periodLimit = calculateMaxPeriodSpend({
              sessionStartsAt: now.value,
              sessionExpiresAt: sessionConfig.value.expiresAt,
              limitPerPeriod: constraint.limit.limit,
              period: constraint.limit.period,
            });

            if (acc === "unlimited") return periodLimit;
            if (acc < periodLimit) return acc;
            return periodLimit;
          }
          case LimitType.Lifetime: {
            const lifetimeLimit = constraint.limit.limit;

            if (acc === "unlimited") return lifetimeLimit;
            if (acc < lifetimeLimit) return acc;
            return lifetimeLimit;
          }
        }
      }, "unlimited" as bigint | "unlimited");

      if (maxAmount === "unlimited") {
        addLimit(policy.target, LimitUnlimited);
      } else {
        addLifetimeLimit(policy.target, maxAmount);
      }
    }

    return limits;
  });

  const spendLimitTokens = computed(() => {
    return Object.entries(limitsByToken.value)
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

  const hasUnlimitedSpend = computed(() => spendLimitTokens.value.some((item) => item.amount === "unlimited"));

  const totalUsd = computed(() => (spendLimitTokens.value || []).reduce((acc, item) => {
    if (!item.token.price) return acc;
    if (item.amount === "unlimited") return acc;
    const formattedTokenAmount = formatUnits(BigInt(item.amount), item.token.decimals);
    return acc + (parseFloat(formattedTokenAmount) * item.token.price);
  }, 0));

  return {
    onchainActionsCount,
    fetchTokensError,
    tokensLoading,
    limitsByToken,
    spendLimitTokens,
    hasUnlimitedSpend,
    totalUsd,
  };
};
