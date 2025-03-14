import {
  type Account,
  type Address,
  type Chain,
  type Client,
  encodeFunctionData, hashTypedData, type Hex,
  type Prettify,
  type Transport,
} from "viem";
import { prepareTransactionRequest } from "viem/actions";
import type { ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { WebAuthValidatorAbi } from "../../../abi/index.js";
import { getAction } from "../../recovery/actions/sendEip712Transaction.js";
import { getEip712Domain } from "../../utils/getEip712Domain.js";

export type CalculateTxHashArgs = {
  credentialId: Hex;
  passkeyPubKey: [Hex, Hex];
  passkeyDomain: string;
  contracts: {
    recoveryOidc: Address; // oidc recovery module
    passkey: Address;
  };
};

export async function calculateTxHash<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<CalculateTxHashArgs>): Promise<Hex> {
  const callData = encodeFunctionData({
    abi: WebAuthValidatorAbi,
    functionName: "addValidationKey",
    args: [
      args.credentialId,
      args.passkeyPubKey,
      args.passkeyDomain,
    ],
  });

  const sendTransactionArgs = {
    account: client.account,
    to: args.contracts.passkey,
    data: callData,
    gas: 50_000_000n, // TODO: Remove when gas estimation is fixed
    type: "eip712",
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const request = await getAction(
    client,
    prepareTransactionRequest,
    "prepareTransactionRequest",
  )({
    account: client.account,
    chain: client.chain,
    nonceManager: client.account.nonceManager,
    ...sendTransactionArgs,
  });

  const signableTransaction = {
    ...request,
    from: client.account.address,
    type: "eip712",
  } as ZksyncTransactionSerializableEIP712;

  const eip712DomainAndMessage = getEip712Domain(signableTransaction);
  return hashTypedData(eip712DomainAndMessage);
}
