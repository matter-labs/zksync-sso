import {
  type Account,
  type Address,
  type Chain,
  type Client,
  encodeAbiParameters,
  encodeFunctionData, formatTransaction,
  type Hash,
  hashTypedData,
  type Hex,
  type Prettify,
  type TransactionReceipt,
  type Transport, zeroAddress,
} from "viem";
import { waitForTransactionReceipt } from "viem/actions";
import { prepareTransactionRequest } from "viem/actions";
import {
  getGeneralPaymasterInput, sendEip712Transaction,
  sendTransaction,
} from "viem/zksync";
import { ByteVector } from "zksync-sso-circuits";

import { OidcRecoveryModuleAbi, WebAuthModuleAbi } from "../../../abi/index.js";
import { encodePasskeyModuleParameters } from "../../../utils/encoding.js";
import { noThrow } from "../../../utils/helpers.js";
import { getEip712Domain } from "../../utils/getEip712Domain.js";

export type OidcData = {
  oidcDigest: Hex;
  iss: Hex;
  aud: Hex;
};
export type ParsedOidcData = {
  oidcDigest: string;
  iss: string;
  aud: string;
};
export const parseOidcData = (oidcData: OidcData): ParsedOidcData => {
  const hexToAscii = (hex: Hex): string => ByteVector.fromHex(hex).toAsciiStr();

  return {
    oidcDigest: oidcData.oidcDigest.toString(), // Do not convert. oidcDigest is just a hash
    iss: hexToAscii(oidcData.iss),
    aud: hexToAscii(oidcData.aud),
  };
};
export type AddOidcAccountArgs = {
  contracts: {
    recoveryOidc: Address; // oidc recovery module
  };
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
  oidcData: OidcData;
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
          { type: "bytes", name: "iss" },
          { type: "bytes", name: "aud" },
        ],
      },
    ],
    [args.oidcData],
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

export type CalculateAddKeyHashArgs = {
  passkeyPubKey: [Buffer, Buffer];
  passkeyDomain: string;
  contracts: {
    recoveryOidc: Address; // oidc recovery module
    passkey: Address;
  };
};

async function oidcTxSign(address: Address, expectedTxHash: Hex, txDigest: Hex): Promise<Hex> {
  const fatSignature = encodeAbiParameters(
    [{ type: "bytes32" }, { type: "bytes32" }],
    [expectedTxHash, txDigest],
  );

  return encodeAbiParameters(
    [
      { type: "bytes" }, // fat signature
      { type: "address" }, // validator address
      { type: "bytes[]" }, // validator data
    ],
    [
      fatSignature,
      address,
      ["0x"], // FIXME: this is assuming there are no other hooks
    ],
  );
}

export const calculateAddKeyTxHash = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<CalculateAddKeyHashArgs>): Promise<Hex> => {
  const encoded = encodePasskeyModuleParameters({
    passkeyPublicKey: args.passkeyPubKey,
    expectedOrigin: args.passkeyDomain,
  });

  const callData = encodeFunctionData({
    abi: WebAuthModuleAbi,
    functionName: "addValidationKey",
    args: [encoded],
  });

  const txRequest = await prepareTransactionRequest(client, {
    account: client.account,
    to: args.contracts.recoveryOidc,
    chainId: client.chain.id,
    data: callData,
    from: client.account.address,
    gas: 20_000_000n,
    type: "eip712",
    parameters: ["gas", "nonce", "fees"],
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any);

  const eip712DomainAndMessage = getEip712Domain(txRequest);
  return hashTypedData(eip712DomainAndMessage);
};

export type AddNewPasskeyViaOidcArgs = {
  expectedTxHash: Hex;
  passkeyPubKey: [Buffer, Buffer];
  passkeyDomain: string;
  contracts: {
    recoveryOidc: Address; // oidc recovery module
    passkey: Address;
  };
};

export const addNewPasskeyViaOidc = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(client: Client<transport, chain, account>, args: Prettify<AddNewPasskeyViaOidcArgs>): Promise<void> => {
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

  const txRequest = await prepareTransactionRequest(client, {
    account: client.account,
    to: zeroAddress,
    chainId: client.chain.id,
    data: "0x",
    from: client.account.address,
    gas: 20_000_000n,
    type: "eip712",
    parameters: ["gas", "nonce", "fees"],
    customSignature: oidcTxSign(args.contracts.recoveryOidc, args.expectedTxHash, ("0x" + "01".repeat(32) as Hex)),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any);

  // const formatters = client.chain?.formatters;
  // const format = formatters?.transaction?.format || formatTransaction;
  //
  // const tx = {
  //   ...format(txRequest),
  //   type: "eip712",
  // };

  const formatters = client.chain?.formatters;
  const format = formatters?.transaction?.format || formatTransaction;

  const tx = {
    ...format(txRequest),
    type: "eip712",
  };

  await sendEip712Transaction(client, tx);
  // console.log(tx);
  // return await waitForTransactionReceipt(client, { hash: txid });
};
