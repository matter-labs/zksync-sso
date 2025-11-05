import type { Address } from "abitype";
import { type CustomSource, type Hash, hashTypedData, type Hex, type LocalAccount } from "viem";
import { toAccount } from "viem/accounts";

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
      const digest = hashTypedData({
        domain: {
          name: "Session",
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

      const signature = await signTransaction({
        hash: digest,
        to: transaction.to!,
        callData: transaction.data,
      });

      // Return transaction with custom signature
      return signature;
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
