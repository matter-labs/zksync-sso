import type { Address } from "abitype";
import { type Chain, type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount, type Transport } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";
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
      const signableTransaction = {
        ...transaction,
        from: this.address!,
        type: "eip712",
      } as ZksyncTransactionSerializableEIP712;

      const eip712DomainAndMessage = getEip712Domain(signableTransaction);
      console.log("signTransaction: eip712DomainAndMessage", eip712DomainAndMessage);
      const digest = hashTypedData(eip712DomainAndMessage);
      console.log("signTransaction: digest", digest, "digest.length", digest.length);

      return serializeTransaction({
        ...signableTransaction,
        customSignature: await sign({
          hash: digest,
        }),
      });
    },
  });

  return {
    ...account,
    source: "ssoPasskeyAccount",
  } as PasskeyAccount;
}
