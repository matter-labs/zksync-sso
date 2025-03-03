import type { Address } from "abitype";
import { type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type ToOidcAccountParameters = {
  address: Address;
  signTransaction: (parameters: { hash: Hash }) => Promise<Hex>;
};

export type OidcAccount = LocalAccount<"ssoOidcAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toOidcAccount(
  parameters: ToOidcAccountParameters,
): OidcAccount {
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
        }),
      });
    },
    async sign({ hash }) {
      return signTransaction({ hash });
    },
    async signMessage({ message }) {
      return signTransaction({
        hash: hashMessage(message),
      });
    },
    async signTypedData(typedData) {
      return signTransaction({
        hash: hashTypedData(typedData),
      });
    },
  });

  return {
    ...account,
    source: "ssoOidcAccount",
    type: "local",
  } as OidcAccount;
};
