import type { Address } from "abitype";
import {
  type CustomSource,
  type Hash,
  hashTypedData,
  type Hex,
  type LocalAccount,
} from "viem";
import { toAccount } from "viem/accounts";

export type ToOidcAccountParameters = {
  address: Address;
  signTransaction: (parameters: {
    hash: Hash;
  }) => Promise<Hex>;
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
      const digest = hashTypedData({
        domain: {
          name: "OIDC",
          version: "1",
          chainId: transaction.chainId,
          verifyingContract: transaction.to!,
        },
        types: {
          Transaction: [
            { name: "to", type: "address" },
            { name: "value", type: "uint256" },
            { name: "data", type: "bytes" },
          ],
        },
        primaryType: "Transaction",
        message: {
          to: transaction.to!,
          value: transaction.value || 0n,
          data: transaction.data || "0x",
        },
      });

      return signTransaction({ hash: digest });
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
}
