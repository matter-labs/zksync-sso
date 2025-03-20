import type { Address } from "abitype";
import { type Chain, createPublicClient, type CustomSource, type Hash, hashMessage, hashTypedData, type Hex, type LocalAccount, type Transport } from "viem";
import { toAccount } from "viem/accounts";
import { serializeTransaction, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { SsoAccountAbi } from "../../abi/index.js";
import { getEip712Domain } from "../utils/getEip712Domain.js";
import type { PasskeyRequiredContracts } from "./client.js";

export type ToPasskeyAccountParameters<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
> = {
  /** Address of the deployed Account's Contract implementation. */
  address: Address;
  sign: (parameters: { hash: Hash }) => Promise<Hex>;
  chain: NonNullable<chain>;
  transport: transport;
  contracts: PasskeyRequiredContracts;
};

export type PasskeyAccount = LocalAccount<"ssoPasskeyAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
};

export function toPasskeyAccount<
  transport extends Transport = Transport,
  chain extends Chain = Chain,
>(
  parameters: ToPasskeyAccountParameters<transport, chain>,
): PasskeyAccount {
  const { address, sign } = parameters;

  const account = toAccount({
    address,
    sign,
    async signMessage({ message }) {
      const publicClient = createPublicClient({ chain: parameters.chain, transport: parameters.transport });
      const hash = await publicClient.readContract({
        address: address,
        abi: SsoAccountAbi,
        functionName: "getEip712Hash",
        args: [{ signedHash: hashMessage(message) }],
      });
      return sign({
        hash,
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
      if (typedData.primaryType === "Transaction") { // Need way better check
        return sign({
          hash: hashTypedData(typedData),
        });
      } else {
        const publicClient = createPublicClient({ chain: parameters.chain, transport: parameters.transport });
        const hash = await publicClient.readContract({
          address: address,
          abi: SsoAccountAbi,
          functionName: "getEip712Hash",
          args: [{ signedHash: hashTypedData(typedData) }],
        });
        return sign({
          hash,
        });
      }
    },
  });

  return {
    ...account,
    source: "ssoPasskeyAccount",
  } as PasskeyAccount;
}
