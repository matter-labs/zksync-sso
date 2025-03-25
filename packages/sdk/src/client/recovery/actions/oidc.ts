import {
  type Account,
  type Address,
  type Chain,
  type Client,
  encodeAbiParameters,
  encodeFunctionData,
  type Hash,
  type Hex,
  type Prettify,
  type TransactionReceipt,
  type Transport,
} from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import {
  getGeneralPaymasterInput,
  sendTransaction,
} from "viem/zksync";

import { OidcRecoveryModuleAbi } from "../../../abi/index.js";
import { noThrow } from "../../../utils/helpers.js";

export type AddOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  oidcDigest: Hex;
  iss: string;
  onTransactionSent?: (hash: Hash) => void;
};
export type AddOidcAccountReturnType = {
  transactionReceipt: TransactionReceipt;
};
export const addOidcAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<AddOidcAccountArgs>): Promise<Prettify<AddOidcAccountReturnType>> => {
  const encodedOidcData = encodeAbiParameters(
    [
      {
        type: "tuple", name: "OidcData", components: [
          { type: "bytes32", name: "oidcDigest" },
          { type: "string", name: "iss" },
        ],
      },
    ],
    [{
      oidcDigest: args.oidcDigest,
      iss: args.iss,
    }],
  );

  const callData = encodeFunctionData({
    abi: OidcRecoveryModuleAbi,
    functionName: "addValidationKey",
    args: [encodedOidcData],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.recoveryOidc,
    paymaster: args.paymaster?.address,
    paymasterInput: args.paymaster?.address ? (args.paymaster?.paymasterInput || getGeneralPaymasterInput({ innerInput: "0x" })) : undefined,
    data: callData,
    gas: 10_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionHash = await sendTransaction(client, sendTransactionArgs);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("addOidcAccount transaction reverted");

  return {
    transactionReceipt,
  };
};
