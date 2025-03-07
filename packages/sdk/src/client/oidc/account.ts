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
import { type Groth16Proof } from "zksync-sso-circuits";

import { getEip712Domain } from "../utils/getEip712Domain.js";

export type ToOidcAccountParameters = {
  address: Address;
  signTransaction: (parameters: {
    hash: Hash;
    proof: ZkProof;
  }) => Promise<Hex>;
};

export type ZkProof = {
  txHash: Hex;
  groth16Proof: Groth16Proof;
  public: bigint[];
};

export type OidcAccount = LocalAccount<"ssoOidcAccount"> & {
  sign: NonNullable<CustomSource["sign"]>;
} & {
  addProof: (proof: ZkProof) => void;
  findProof: (txHash: Hex) => ZkProof;
};

export function toOidcAccount(
  parameters: ToOidcAccountParameters,
): OidcAccount {
  const { address, signTransaction } = parameters;

  const proofs: ZkProof[] = [];

  const findProof = (txHash: Hex) => {
    const found = proofs.find((proof) => proof.txHash === txHash);
    if (!found) throw new Error("Missing proof");
    return found;
  };

  const addProof = (proof: ZkProof) => {
    proofs.push(proof);
  };
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
          proof: findProof(digest),
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
        proof: findProof(digest),
      });
    },
  });

  return {
    ...account,
    source: "ssoOidcAccount",
    type: "local",
    addProof,
    findProof,
  } as OidcAccount;
};
