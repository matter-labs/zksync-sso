import type { Address } from "abitype";
import { type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";

export type ToRecoveryAccountParameters = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  signTransaction: (parameters: { hash: Hash }) => Promise<Hex>;
};

export type RecoveryAccount = LocalAccount<"ssoRecoveryAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toRecoveryAccount(
  parameters: ToRecoveryAccountParameters,
): RecoveryAccount {
  const { address, signTransaction } = parameters;

  const account = toAccount({
    address,
    async signTransaction(transaction) {
      const digest = hashTypedData({
        domain: {
          name: "Recovery",
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
    source: "ssoRecoveryAccount",
  } as RecoveryAccount;
}
