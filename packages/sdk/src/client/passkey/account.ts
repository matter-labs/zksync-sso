import type { Address } from "abitype";
import { type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type ToPasskeyAccountParameters = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  sign: (parameters: { hash: Hash }) => Promise<Hex>;
};

export type PasskeyAccount = LocalAccount<"ssoPasskeyAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toPasskeyAccount(
  parameters: ToPasskeyAccountParameters,
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
    async signTransaction(transaction) {
      const signableTransaction = {
        ...transaction,
        from: this.address!,
        type: "eip712",
      } as ZksyncTransactionSerializableEIP712;

      const eip712DomainAndMessage = getEip712Domain(signableTransaction);
      const digest = hashTypedData(eip712DomainAndMessage);

      return serializeTransaction({
        ...signableTransaction,
        customSignature: await sign({
          hash: digest,
        }),
      });
    },
    async signTypedData(typedData) {
      return sign({
        hash: hashTypedData(typedData),
      });
    },
  });

  return {
    ...account,
    source: "ssoPasskeyAccount",
  } as PasskeyAccount;
}
