import type { Account, Address, Chain, Client, Hash, Hex, Prettify, Transport } from "viem";
import { concat, encodePacked, keccak256, toHex } from "viem";
import { readContract } from "viem/actions";

import { AAFactoryAbi } from "../../../abi/AAFactory.js";
import { type CustomPaymasterHandler } from "../../../paymaster/index.js";
import type { SessionConfig } from "../../../utils/session.js";

export type DeployAccountArgs = {
  owner: Address; // Wallet owner
  contracts: {
    accountFactory: Address;
    session: Address;
  };
  initialSession?: SessionConfig;
  prefix?: string; // vendor prefix
  onTransactionSent?: (hash: Hash) => void;
  paymasterHandler?: CustomPaymasterHandler;
  paymaster?: {
    address: Address;
    paymasterInput?: Hex;
  };
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
