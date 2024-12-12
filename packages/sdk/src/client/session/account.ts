import type { Address } from "abitype";
import { type CustomSource, type Hash, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type ToSessionAccountParameters = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  signTransaction: (parameters: { hash: Hash; to: Address; callData?: Hash }) => Promise<Hex>;
};

export type SessionAccount = LocalAccount<"ssoSessionAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toSessionAccount(
  parameters: ToSessionAccountParameters,
): SessionAccount {
  const { address, signTransaction } = parameters;

  const account = toAccount({
    address,
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
        customSignature: await signTransaction({
          hash: digest,
          to: signableTransaction.to!,
          callData: signableTransaction.data,
        }),
      });
    },
    sign: async () => {
      throw new Error(`account.sign not supported for SSO Session Client`);
    },
    signMessage: async () => {
      throw new Error(`account.signMessage not supported for SSO Session Client`);
    },
    signTypedData: async () => {
      throw new Error(`account.signTypedData not supported for SSO Session Client`);
    },
  });

  return {
    ...account,
    source: "ssoSessionAccount",
  } as SessionAccount;
}
