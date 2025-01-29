import type { Address, Hex } from "viem";

import type { CustomPaymasterHandler, CustomPaymasterHandlerResponse, CustomPaymasterParameters } from "../index.js";

const ZYFI_ENDPOINT = "https://api.zyfi.org/api/";
const ERC20_PAYMASTER = `${ZYFI_ENDPOINT}erc20_paymaster/v1`;
const SPONSORED_PAYMASTER = `${ZYFI_ENDPOINT}erc20_sponsored_paymaster/v1`;

export interface ZyfiPaymasterParams {
  /** Your API key. Get it from Dashboard on https://www.zyfi.org/ */
  apiKey: string;
  /** The address of the token to be used for fee payment */
  feeTokenAddress?: Address;
  /** Whether to check for NFT ownership for fee payment */
  checkNFT?: boolean;
  /** Whether the transaction is on a testnet */
  isTestnet?: boolean;
  /** The ratio of fees to be sponsored by the paymaster (0-100) */
  sponsorshipRatio?: number;
  /** determines the user nonces interval for which the response will be valid (current nonce + replayLimit). */
  replayLimit?: number;
}

export function createZyfiPaymaster(params: ZyfiPaymasterParams): CustomPaymasterHandler {
  if (!params.apiKey) throw new Error("ZyFi: Provide API KEY");
  if (
    params.sponsorshipRatio !== undefined
    && (params.sponsorshipRatio > 100
      || params.sponsorshipRatio < 0)
  ) throw new Error("ZyFi: Sponsorship ratio must be between 0-100");

  return async function zyfiPaymaster(args: CustomPaymasterParameters): CustomPaymasterHandlerResponse {
    const url = params.sponsorshipRatio ? SPONSORED_PAYMASTER : ERC20_PAYMASTER;
    const payload = {
      replayLimit: params.replayLimit,
      sponsorshipRatio: params.sponsorshipRatio,
      chainId: args.chainId,
      checkNFT: params.checkNFT,
      feeTokenAddress: params.feeTokenAddress,
      isTestnet: params.isTestnet,
      gasLimit: args.gas || undefined,
      txData: {
        from: args.from,
        to: args.to,
        data: args.data,
        value: args.value,
      },
    };
    const response: ApiResponse = await fetch(
      url,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-API-Key": params.apiKey,
        },
        body: JSON.stringify(payload),
      },
    ).then((response) => {
      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
      return response.json();
    });

    const { paymasterParams, gasPerPubdata } = response.txData.customData;
    return {
      paymaster: paymasterParams.paymaster,
      paymasterInput: paymasterParams.paymasterInput,
      maxFeePerGas: BigInt(response.txData.maxFeePerGas),
      gas: BigInt(response.txData.gasLimit),
      maxPriorityFeePerGas: BigInt(response.txData.maxPriorityFeePerGas ?? 0),
      gasPerPubdata: gasPerPubdata ? BigInt(gasPerPubdata) : undefined,
    };
  };
}

type ApiResponse = {
  txData: {
    customData: {
      paymasterParams: {
        paymaster: Address;
        paymasterInput: Hex;
      };
      gasPerPubdata?: number;
    };
    maxFeePerGas: string;
    maxPriorityFeePerGas?: string;
    gasLimit: number;
  };
  gasLimit: string;
  gasPrice: string;
  tokenAddress: Address;
  tokenPrice: string;
  feeTokenAmount: string;
  feeTokendecimals: string;
  feeUSD: string;
  expirationTime: string;
  expiresIn: string;
  paymasterAddress: Address;
  messageHash: string;
  maxNonce?: string;
  protocolAddress?: Address;
  sponsorshipRatio?: string;
  warnings?: string[];
};
