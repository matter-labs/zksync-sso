import type { Address } from "abitype";
import {
  type CustomSource,
  type Hash,
  hashTypedData,
  type Hex,
  type LocalAccount,
} from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type ToOidcAccountParameters = {
  address: Address;
  signTransaction: (parameters: {
    hash: Hash;
  }) => Promise<Hex>;
};

export type OidcKey = {
  issHash: Hex;
  kid: Hex;
  n: bigint[];
  e: Hex;
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
    async signMessage() {
      throw new Error("Oidc account cannot sign messages");
    },
    async signTypedData(typedData) {
      const digest = hashTypedData(typedData);
      return signTransaction({
        hash: digest,
      });
    },
  });

  return {
    ...account,
    source: "ssoOidcAccount",
    type: "local",
  } as OidcAccount;
};
