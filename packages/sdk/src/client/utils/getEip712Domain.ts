import { toHex } from "viem";
import type { EIP712DomainFn, ZksyncEIP712TransactionSignable, ZksyncTransactionSerializable, ZksyncTransactionSerializableEIP712 } from "viem/zksync";
import { hashBytecode } from "viem/zksync";

import { assertEip712Transaction } from "./assertEip712Transaction.js";

const ZeroBytes = "0x0";

const gasPerPubdataDefault = 50000n;

type EIP712FieldType = "uint256" | "bytes" | "bytes32[]";
type EIP712Field = { name: string; type: EIP712FieldType };

/* TODO: This is already available at viem already but not exported (viem also has a mistake in domain name letter casing) */
export const getEip712Domain: EIP712DomainFn<
  ZksyncTransactionSerializable,
  ZksyncEIP712TransactionSignable
> = (transaction) => {
  assertEip712Transaction(transaction);

  const message = transactionToMessage(
    transaction as ZksyncTransactionSerializableEIP712,
  );

  const transactionTypes: Record<string, EIP712Field[]> = {
    Transaction: [
      { name: "txType", type: "uint256" },
      { name: "from", type: "uint256" },
      { name: "to", type: "uint256" },
      { name: "gasLimit", type: "uint256" },
      { name: "gasPerPubdataByteLimit", type: "uint256" },
      { name: "maxFeePerGas", type: "uint256" },
      { name: "maxPriorityFeePerGas", type: "uint256" },
      { name: "paymaster", type: "uint256" },
      { name: "nonce", type: "uint256" },
      { name: "value", type: "uint256" },
      { name: "data", type: "bytes" },
      { name: "factoryDeps", type: "bytes32[]" },
      { name: "paymasterInput", type: "bytes" },
    ],
  };

  const signable = {
    domain: {
      // cspell:ignore zkSync
      name: `zkSync`,
      version: "2",
      chainId: transaction.chainId,
    },
    types: transactionTypes,
    primaryType: "Transaction",
    message: message,
  };
  console.log("getEip712Domain: signable", signable);

  return signable;
};

//////////////////////////////////////////////////////////////////////////////
// Utilities

function transactionToMessage(
  transaction: ZksyncTransactionSerializableEIP712,
): ZksyncEIP712TransactionSignable {
  const {
    gas,
    nonce,
    to,
    from,
    value,
    maxFeePerGas,
    maxPriorityFeePerGas,
    factoryDeps,
    paymaster,
    paymasterInput,
    gasPerPubdata,
    data,
  } = transaction;

  const signable = {
    txType: 113n,
    from: BigInt(from),
    to: to ? BigInt(to) : 0n,
    gasLimit: gas ?? 0n,
    gasPerPubdataByteLimit: gasPerPubdata ?? gasPerPubdataDefault,
    maxFeePerGas: maxFeePerGas ?? 0n,
    maxPriorityFeePerGas: maxPriorityFeePerGas ?? 0n,
    paymaster: paymaster ? BigInt(paymaster) : 0n,
    nonce: nonce ? BigInt(nonce) : 0n,
    value: value ?? 0n,
    data: data ? data : ZeroBytes,
    factoryDeps: factoryDeps?.map((dep) => toHex(hashBytecode(dep))) ?? [],
    paymasterInput: paymasterInput ? paymasterInput : "0x",
  };
  console.log("transactionToMessage: signable.data", signable.data);
  return signable;
}
