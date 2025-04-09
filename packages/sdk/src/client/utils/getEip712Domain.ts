import { toHex } from "viem";
import { type EIP712DomainFn, hashBytecode, type ZksyncEIP712TransactionSignable, type ZksyncTransactionSerializable, type ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { assertEip712Transaction } from "./assertEip712Transaction.js";

const gasPerPubdataDefault = 50000n;

/* TODO: This is already available at viem already but not exported (viem also has a mistake in domain name letter casing) */
export const getEip712Domain: EIP712DomainFn<
  ZksyncTransactionSerializable,
  ZksyncEIP712TransactionSignable
> = (transaction) => {
  assertEip712Transaction(transaction);

  const message = transactionToMessage(
    transaction as ZksyncTransactionSerializableEIP712,
  );

  return {
    domain: {
      // cspell:ignore zkSync
      name: `zkSync`,
      version: "2",
      chainId: transaction.chainId,
    },
    types: {
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
    },
    primaryType: "Transaction",
    message: message,
  };
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

  return {
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
    data: data ? data : "0x",
    factoryDeps: factoryDeps?.map((dep) => toHex(hashBytecode(dep))) ?? [],
    paymasterInput: paymasterInput ? paymasterInput : "0x",
  };
}
