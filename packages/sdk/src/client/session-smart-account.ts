import type { Address } from "abitype";
import { type CustomSource, type Hash, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "./utils/getEip712Domain.js";

export type ToSmartAccountParameters = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  /** Function to sign a hash. */
  signTransaction: (parameters: { hash: Hash; to: Address; callData?: Hash }) => Promise<Hex>;
};

export type ZksyncSmartAccount = LocalAccount<"smartAccountZksync"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

type ErrorType<name extends string = "Error"> = Error & { name: name };
export type ToSmartAccountErrorType = ErrorType;

/**
 * Creates a [ZKsync Smart Account](https://docs.zksync.io/build/developer-reference/account-abstraction/building-smart-accounts)
 * from a Contract Address and a custom sign function.
 */
export function toSmartAccount(
  parameters: ToSmartAccountParameters,
): ZksyncSmartAccount {
  const { address, signTransaction } = parameters;

  const account = toAccount({
    address,
    sign: async () => {
      throw new Error(`account.sign not supported for SSO Session Client`);
    },
    signMessage: async () => {
      throw new Error(`account.signMessage not supported for SSO Session Client`);
    },
    signTypedData: async () => {
      throw new Error(`account.signTypedData not supported for SSO Session Client`);
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
        customSignature: await signTransaction({
          hash: digest,
          to: signableTransaction.to!,
          callData: signableTransaction.data,
        }),
      });
    },
  });

  return {
    ...account,
    source: "smartAccountZksync",
  } as ZksyncSmartAccount;
}
