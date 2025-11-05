import type { Address } from "abitype";
import { type Chain, type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount, serializeTransaction, type Transport } from "viem";
import { toAccount } from "viem/accounts";

import type { PasskeyRequiredContracts } from "./client.js";

export type ToPasskeyAccountParameters<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  sign: (parameters: { hash: Hash }) => Promise<Hex>;
  chain: NonNullable<chain>;
  transport: transport;
  contracts: PasskeyRequiredContracts;
  credentialId: Hex;
  rpId?: string;
  origin?: string;
};

export type PasskeyAccount = LocalAccount<"ssoPasskeyAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toPasskeyAccount<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
>(
  parameters: ToPasskeyAccountParameters<transport, chain>,
): PasskeyAccount {
  const { address, sign } = parameters;

  const account = toAccount({
    address,
    sign,
    async signMessage({ message }) {
      return sign({
        hash: hashMessage(message),
      });
    },
    async signTypedData(typedData) {
      return sign({
        hash: hashTypedData(typedData),
      });
    },
    async signTransaction(transaction) {
      // Transaction signing is handled by viem's standard flow
      // The account's sign function will be called automatically by viem
      return serializeTransaction(transaction as any);
    },
  });

  return {
    ...account,
    source: "ssoPasskeyAccount",
  } as PasskeyAccount;
}
