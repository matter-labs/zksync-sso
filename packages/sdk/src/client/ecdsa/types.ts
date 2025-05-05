import {
  type Account,
  type Address,
  type Chain,
  createWalletClient, custom,
  type EIP1193Provider,
  hashTypedData,
  type LocalAccount,
  type OneOf,
  type Transport,
  type WalletClient } from "viem";
import { toAccount } from "viem/accounts";
import { signTypedData } from "viem/actions";
import { getAction } from "viem/utils";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type Signer = OneOf<
  | EIP1193Provider
  | WalletClient<Transport, Chain | undefined, Account>
  | LocalAccount
>;

export type EthereumProvider = OneOf<
  { request(...args: any): Promise<any> } | EIP1193Provider
>;

export async function toOwner<provider extends EthereumProvider>({
  owner,
  address,
}: {
  owner: OneOf<
    | provider
    | WalletClient<Transport, Chain | undefined, Account>
    | LocalAccount
  >;
  address?: Address;
}): Promise<LocalAccount> {
  if ("type" in owner && owner.type === "local") {
    return owner as LocalAccount;
  }

  let walletClient:
    | WalletClient<Transport, Chain | undefined, Account>
    | undefined = undefined;

  if ("request" in owner) {
    if (!address) {
      try {
        [address] = await (owner as EthereumProvider).request({
          method: "eth_requestAccounts",
        });
      } catch {
        [address] = await (owner as EthereumProvider).request({
          method: "eth_accounts",
        });
      }
    }
    if (!address) {
      // For TS to be happy
      throw new Error("address is required");
    }
    walletClient = createWalletClient({
      account: address,
      transport: custom(owner as EthereumProvider),
    });
  }

  if (!walletClient) {
    walletClient = owner as WalletClient<
      Transport,
          Chain | undefined,
          Account
    >;
  }

  return toAccount({
    address: walletClient.account.address,
    async signMessage({ message }) {
      return walletClient.signMessage({ message });
    },
    async signTransaction(transaction) {
      const signableTransaction = {
        ...transaction,
        from: this.address!,
        type: "eip712",
      } as ZksyncTransactionSerializableEIP712;

      const eip712DomainAndMessage = getEip712Domain(signableTransaction);
      const digest = hashTypedData(eip712DomainAndMessage);

      const signedMessage = await walletClient.signMessage({ message: digest });

      return serializeTransaction({
        ...signableTransaction,
        customSignature: signedMessage,
      });
    },
    async signTypedData(typedData) {
      return getAction(
        walletClient,
        signTypedData,
        "signTypedData",
      )(typedData as any);
    },
  });
}
