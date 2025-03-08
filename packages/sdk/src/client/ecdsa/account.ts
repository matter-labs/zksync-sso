import type { Address } from "abitype";
import { type CustomSource, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";
import { type Signer, toOwner } from "./types.js";

export type ToEcdsaAccountParameters = {
  /** Address of the deployed SSO Wallet */
  address: Address;
  /** Owner of the EOA */
  owner: Signer;
};

export type EcdsaAccount = LocalAccount<"ssoEcdsaAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export async function toEcdsaAccount(
  parameters: ToEcdsaAccountParameters,
): Promise<EcdsaAccount> {
  const { address, owner } = parameters;

  const localOwner = await toOwner({ owner, address });

  const account = toAccount({
    address,
    async signMessage({ message }) {
      return localOwner.signMessage({ message });
    },
    async signTransaction(transaction) {
      const signableTransaction = {
        ...transaction,
        from: this.address!,
        type: "eip712",
      } as ZksyncTransactionSerializableEIP712;

      const eip712DomainAndMessage = getEip712Domain(signableTransaction);
      const signature = await localOwner.signTypedData(eip712DomainAndMessage);

      return serializeTransaction({
        ...signableTransaction,
        customSignature: signature,
      });
    },
    async signTypedData(typedData) {
      return localOwner.signTypedData(typedData);
    },
  });

  return {
    ...account,
    source: "ssoEcdsaAccount",
  } as EcdsaAccount;
}
