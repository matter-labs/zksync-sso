import {
  type Account,
  type Address,
  type Chain,
  type Client,
  type Hex,
  type Prettify, type TransactionReceipt,
  type Transport, zeroAddress,
} from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { sendTransaction } from "viem/zksync";

export type AddNewPasskeyViaOidcArgs = {
  expectedTxHash: Hex;
  passkeyPubKey: [Buffer, Buffer];
  passkeyDomain: string;
  contracts: {
    recoveryOidc: Address; // oidc recovery module
    passkey: Address;
  };
};

export async function addNewPasskeyViaOidc<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
// eslint-disable-next-line @typescript-eslint/no-unused-vars
>(client: Client<transport, chain, account>, args: Prettify<AddNewPasskeyViaOidcArgs>): Promise<TransactionReceipt> {
  // const encoded = encodePasskeyModuleParameters({
  //   passkeyPublicKey: args.passkeyPubKey,
  //   expectedOrigin: args.passkeyDomain,
  // });
  //
  // const callData = encodeFunctionData({
  //   abi: WebAuthModuleAbi,
  //   functionName: "addValidationKey",
  //   args: [encoded],
  // });
  // console.log("callData", callData);

  const sendTransactionArgs = {
    account: client.account,
    to: zeroAddress,
    data: "0x",
    gas: 20_000_000n, // TODO: Remove when gas estimation is fixed
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  const transactionHash = await sendTransaction(client, sendTransactionArgs);

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  console.log("transactionReceipt", transactionReceipt);
  if (transactionReceipt.status !== "success") {
    throw new Error("Add passkey via oidc reverted");
  }

  return transactionReceipt;
}
