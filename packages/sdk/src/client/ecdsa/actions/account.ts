import type { Account, Address, Chain, Client, Hash, Hex, Prettify, TransactionReceipt, Transport } from "viem";
import { concat, encodePacked, getAddress, keccak256, parseEventLogs, toHex } from "viem";
import { readContract, waitForTransactionReceipt, writeContract } from "viem/actions";
import { getGeneralPaymasterInput } from "viem/zksync";

import { AAFactoryAbi } from "../../../abi/AAFactory.js";
import { type CustomPaymasterHandler } from "../../../paymaster/index.js";
import { encodeModuleData, encodeSession } from "../../../utils/encoding.js";
import { noThrow } from "../../../utils/helpers.js";
import type { SessionConfig } from "../../../utils/session.js";

export type DeployAccountArgs = {
  owner: Address; // Wallet owner
  contracts: {
    accountFactory: Address;
    session: Address;
  };
  initialSession?: SessionConfig;
  salt?: Uint8Array; // Random 32 bytes
  prefix?: string; // vendor prefix
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
};

export type DeployAccountReturnType = {
  address: Address;
  transactionReceipt: TransactionReceipt;
};

export type FetchAccountArgs = {
  prefix?: string; // vendor prefix
  owner: Address; // Wallet owner
  contracts: {
    accountFactory: Address;
    session: Address;
  };
};

export type FetchAccountReturnType = {
  address: Address;
  owner: Address;
};

const NULL_ADDRESS = "0x0000000000000000000000000000000000000000";

export const deployAccount = async <
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: Client<transport, chain, account>, // Account deployer (any viem client)
  args: Prettify<DeployAccountArgs>,
): Promise<DeployAccountReturnType> => {
  if (!args.salt) {
    args.salt = crypto.getRandomValues(new Uint8Array(32));
  }

  if (args.prefix && args.prefix.length > 12) throw new Error("prefix must not be longer than 12");

  const uniqueId = concat([toHex(args.prefix || "", { size: 12 }), args.owner]);

  const encodedSessionKeyModuleData = encodeModuleData({
    address: args.contracts.session,
    parameters: args.initialSession ? encodeSession(args.initialSession) : "0x",
  });

  let deployProxyArgs = {
    account: client.account!,
    chain: client.chain!,
    address: args.contracts.accountFactory,
    abi: AAFactoryAbi,
    functionName: "deployProxySsoAccount",
    args: [
      uniqueId,
      [encodedSessionKeyModuleData],
      [args.owner],
    ],
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;

  if (args.paymaster) {
    deployProxyArgs = {
      ...deployProxyArgs,
      paymaster: args.paymaster.address,
      paymasterInput: args.paymaster.paymasterInput ?? getGeneralPaymasterInput({ innerInput: "0x" }),
    };
  }

  const transactionHash = await writeContract(client, deployProxyArgs);
  if (args.onTransactionSent) {
    noThrow(() => args.onTransactionSent?.(transactionHash));
  }

  const transactionReceipt = await waitForTransactionReceipt(client, { hash: transactionHash });
  if (transactionReceipt.status !== "success") throw new Error("Account deployment transaction reverted");

  const accountCreatedEvent = parseEventLogs({ abi: AAFactoryAbi, logs: transactionReceipt.logs })
    .find((log) => log && log.eventName === "AccountCreated");

  if (!accountCreatedEvent) {
    throw new Error("No contract address in transaction receipt");
  }

  const { accountAddress } = accountCreatedEvent.args;

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

  if (args.prefix && args.prefix.length > 12) throw new Error("prefix must not be longer than 12");

  const uniqueId = concat([toHex(args.prefix || "", { size: 12 }), args.owner]);
  const accountId = keccak256(encodePacked(["bytes32", "address"], [uniqueId, client.account.address]));

  if (!accountId) throw new Error("No account ID provided");

  const accountAddress = await readContract(client, {
    abi: AAFactoryAbi,
    address: args.contracts.accountFactory,
    functionName: "accountMappings",
    args: [accountId],
  });

  if (!accountAddress || accountAddress == NULL_ADDRESS) throw new Error(`No account found for ID: ${accountId}`);

  return {
    address: accountAddress,
    owner: args.owner,
  };
};
