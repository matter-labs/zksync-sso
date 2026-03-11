import { $fetch, FetchError } from "ofetch";
import { type Address, erc20Abi } from "viem";

export type Token = {
  address: Address;
  name?: string;
  symbol: string;
  decimals: number;
  iconUrl?: string;
  price?: number;
  fetchedFromBlockExplorer: boolean;
};

type TokenKey = `${number}-${Address}`;
export const getTokenKey = (chainId: number, tokenAddress: Address): TokenKey => `${chainId}-${tokenAddress}`;

export const useTokensStore = defineStore("tokens", () => {
  const { getPublicClient, defaultChain, blockExplorerApiUrl } = useClientStore();

  const tokens = ref<{ [tokenKey: TokenKey]: Token }>({});
  const tokensInProgress = ref<{ [tokenKey: TokenKey]: boolean }>({});
  const addressIsNotToken = ref<{ [tokenKey: TokenKey]: boolean }>({});
  const setToken = (args: { chainId: number; token: Token }) => {
    tokens.value[getTokenKey(args.chainId, args.token.address)] = args.token;
  };
  const getToken = (args: { chainId: number; tokenAddress: Address }): Token | undefined => {
    return tokens.value[getTokenKey(args.chainId, args.tokenAddress)];
  };
  const getTokenInProgress = (args: { chainId: number; tokenAddress: Address }): boolean => {
    return tokensInProgress.value[`${args.chainId}-${args.tokenAddress}`] || false;
  };

  /* Add base token info to be available by default */
  {
    const baseToken = defaultChain.nativeCurrency;
    setToken({
      chainId: defaultChain.id,
      token: {
        address: BASE_TOKEN_ADDRESS,
        symbol: baseToken.symbol,
        name: baseToken.name,
        decimals: baseToken.decimals,
        fetchedFromBlockExplorer: false,
      },
    });
  }

  const fetchTokenWithRpc = useMemoize(async (args: { chainId: number; tokenAddress: Address }): Promise<Token | undefined> => {
    const client = getPublicClient();
    // eslint-disable-next-line prefer-const
    let [symbol, name, decimals] = await Promise.all([
      client.readContract({
        abi: erc20Abi,
        functionName: "symbol",
        address: args.tokenAddress,
      }).catch(() => undefined),
      client.readContract({
        abi: erc20Abi,
        functionName: "name",
        address: args.tokenAddress,
      }).catch(() => undefined),
      client.readContract({
        abi: erc20Abi,
        functionName: "decimals",
        address: args.tokenAddress,
      }).catch(() => undefined),
    ]);
    if (!symbol || !name || typeof decimals !== "bigint") {
      return undefined;
    }

    nextTick(() => fetchTokenWithRpc.clear()); // We don't need cache, just reuse the promise

    let iconUrl: string | undefined = undefined;
    if (args.tokenAddress === BASE_TOKEN_ADDRESS && symbol === "ETH") {
      name = "Ether";
      iconUrl = "/img/eth.svg";
    }

    return {
      address: args.tokenAddress,
      name: name,
      symbol: symbol,
      decimals: parseInt((decimals as bigint).toString()),
      price: undefined,
      iconUrl: iconUrl,
      fetchedFromBlockExplorer: false,
    };
  });

  type FetchTokenArgs = { chainId: number; tokenAddress: Address; refresh?: true };
  const _fetchToken = useMemoize(async (args: FetchTokenArgs) => {
    const tokenKey = getTokenKey(args.chainId, args.tokenAddress);
    const cachedToken = tokens.value[tokenKey];
    if (cachedToken?.fetchedFromBlockExplorer && !args.refresh) {
      return cachedToken;
    }

    const notTokenError = new FetchError("Address is not a token");
    notTokenError.statusCode = 404;
    if (addressIsNotToken.value[tokenKey]) throw notTokenError;

    let fetchError: unknown;
    const { result } = blockExplorerApiUrl
      ? await $fetch<{
        result: {
          tokenName: string;
          symbol: string;
          tokenDecimal: string;
          tokenPriceUSD: string;
          iconURL: string;
        }[];
      }>(`${blockExplorerApiUrl}?module=token&action=tokeninfo&contractaddress=${args.tokenAddress}`).catch((err) => {
        fetchError = err;
        // eslint-disable-next-line no-console
        console.error("Failed to fetch token info from block explorer", err);
        return { result: [] as never[] };
      })
      : { result: [] as never[] };

    const tokenInfo = result[0];
    if (!tokenInfo) {
      // If token not found, try to fetch it with RPC (Block Explorer might have outdated info / some issues).
      // If found in RPC return it.
      // If not found, provided address might not be token contract.
      // Only mark it as not token contract in case there were no fetch errors originally.
      const tokenInfoFromRpc = await fetchTokenWithRpc(args);
      if (!tokenInfoFromRpc) {
        if (!fetchError) {
          addressIsNotToken.value[tokenKey] = true;
          throw notTokenError;
        }
        throw fetchError;
      }
      setToken({ chainId: args.chainId, token: tokenInfoFromRpc });
      return tokenInfoFromRpc;
    }

    const token: Token = {
      address: args.tokenAddress,
      name: tokenInfo.tokenName,
      symbol: tokenInfo.symbol,
      decimals: parseInt(tokenInfo.tokenDecimal) || 0,
      price: parseFloat(tokenInfo.tokenPriceUSD) || undefined,
      iconUrl: tokenInfo.iconURL,
      fetchedFromBlockExplorer: true,
    };

    if (token.address === BASE_TOKEN_ADDRESS && token.symbol === "ETH") {
      token.iconUrl = "/img/eth.svg";
    }

    setToken({ chainId: args.chainId, token });
    return token;
  });
  const fetchToken = async (args: FetchTokenArgs) => {
    try {
      tokensInProgress.value[`${args.chainId}-${args.tokenAddress}`] = true;
      return await _fetchToken(args);
    } finally {
      tokensInProgress.value[`${args.chainId}-${args.tokenAddress}`] = false;
      _fetchToken.clear(); // We don't need cache, just reuse the promise
    }
  };
  const fetchTokens = async (args: { chainId: number; tokenAddresses: Address[]; throwErrorAsserter?: (err: unknown) => boolean }) => {
    return await Promise.all(
      args.tokenAddresses.map((tokenAddress) => fetchToken({ chainId: args.chainId, tokenAddress }).catch((err) => {
        if (args.throwErrorAsserter) {
          const toThrowError = args.throwErrorAsserter(err);
          if (toThrowError) throw err;
        } else {
          throw err;
        }
      })),
    );
  };

  return {
    getToken,
    getTokenInProgress,
    fetchToken,
    fetchTokens,
  };
});
