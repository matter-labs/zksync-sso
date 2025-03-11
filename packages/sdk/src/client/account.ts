// Copyright 2025 cbe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { randomBytes } from "crypto";
import { AAFactoryAbi } from "src/abi/AAFactory.js";
import { encodeModuleData, encodeSession } from "src/utils/encoding.js";
import { noThrow } from "src/utils/helpers.js";
import type { SessionConfig } from "src/utils/session.js";
import { type Account, type Address, type Chain, type Client, getAddress, type Hash, type Hex, keccak256, parseEventLogs, type Prettify, toHex, type TransactionReceipt, type Transport } from "viem";
import { waitForTransactionReceipt, writeContract } from "viem/actions";
import { getGeneralPaymasterInput } from "viem/zksync";

import type { FetchEcdsaAccountArgs } from "./ecdsa/index.js";
import { type DeployPasskeyAccountArgs, type FetchPasskeyAccountArgs, getPasskeyData } from "./passkey/actions/account.js";

export type DeployAccountArgs = {
  paymasterAddress?: Address; // Paymaster used to pay the fees of creating accounts
  paymasterInput?: Hex; // Input for paymaster (if provided)
  moduleArgs: (SessionConfig | DeployPasskeyAccountArgs)[];
  contracts: {
    accountFactory: Address;
    passkey: Address;
    session: Address;
  };
  onTransactionSent?: (hash: Hash) => void;
};
export type DeployAccountReturnType = {
  address: Address;
  transactionReceipt: TransactionReceipt;
};
export type FetchAccountArgs = {
  location: FetchPasskeyAccountArgs | FetchEcdsaAccountArgs;
  contracts: {
    accountFactory: Address;
    passkey: Address;
    session: Address;
  };
};
export type FetchAccountReturnType = {
  address: Address;
  passkeyPublicKey: Uint8Array;
};
export const deployAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: Client<transport, chain, account>, // Account deployer (any viem client)
  args: Prettify<DeployAccountArgs>,
): Promise<DeployAccountReturnType> => {
  let origin: string | undefined = args.expectedOrigin;
  if (!origin) {
    try {
      origin = window.location.origin;
    } catch {
      throw new Error("Can't identify expectedOrigin, please provide it manually");
    }
  }

  const owners: Address[] = args.ownerPublicKeys ?? [];
  const moduleData: Hex[] = [];
  const accountId = args.uniqueAccountId ?? randomBytes(32).toString("hex");
  if (args.credentialPublicKey.length == 0 && args.ownerPublicKeys?.length == 0) {
    throw new Error("No public keys provided");
  }
  if (args.credentialPublicKey.length != 0) {
    moduleData.push(getPasskeyData(args.credentialPublicKey, args.credentialId, args.contracts));
  }

  if (args.contracts.session) {
    moduleData.push(encodeModuleData({
      address: args.contracts.session,
      parameters: args.initialSession ? encodeSession(args.initialSession) : "0x",
    }));
  }

  let deployProxyArgs = {
    account: client.account!,
    chain: client.chain!,
    address: args.contracts.accountFactory,
    abi: AAFactoryAbi,
    functionName: "deployProxySsoAccount",
    args: [
      keccak256(toHex(accountId)),
      moduleData,
      owners,
    ],
  } as any;

  if (args.paymasterAddress) {
    deployProxyArgs = {
      ...deployProxyArgs,
      paymaster: args.paymasterAddress,
      paymasterInput: args.paymasterInput ?? getGeneralPaymasterInput({ innerInput: "0x" }),
    };
  }

  const transactionHash = await writeContract(client, deployProxyArgs);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("Account deployment transaction reverted");
  const getAccountId = () => {
    if (transactionReceipt.contractAddress) {
      return transactionReceipt.contractAddress;
    }
    const accountCreatedEvent = parseEventLogs({ abi: AAFactoryAbi, logs: transactionReceipt.logs })
      .find((log) => log && log.eventName === "AccountCreated");

    if (!accountCreatedEvent) {
      throw new Error("No contract address in transaction receipt");
    }

    const { accountAddress } = accountCreatedEvent.args;
    return accountAddress;
  };

  const accountAddress = getAccountId();

  return {
    address: getAddress(accountAddress),
    transactionReceipt: transactionReceipt,
  };
};

export const fetchAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: Client<transport, chain, account>, // Account deployer (any viem client)
  args: Prettify<FetchAccountArgs>,
): Promise<FetchAccountReturnType> => {
  if (!args.contracts.accountFactory) throw new Error("Account factory address is not set");

  return {
    username,
    address: accountAddress,
    passkeyPublicKey,
  };
};
