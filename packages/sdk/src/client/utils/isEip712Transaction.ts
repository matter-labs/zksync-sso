import type { ExactPartial } from "viem";
import type { ZksyncTransactionSerializable } from "viem/zksync";

export function isEIP712Transaction(
  transaction: ExactPartial<ZksyncTransactionSerializable>,
) {
  if (transaction.type === "eip712") return true;
  if (
    ("customSignature" in transaction && transaction.customSignature)
    || ("paymaster" in transaction && transaction.paymaster)
    || ("paymasterInput" in transaction && transaction.paymasterInput)
    || ("gasPerPubdata" in transaction
      && typeof transaction.gasPerPubdata === "bigint")
      || ("factoryDeps" in transaction && transaction.factoryDeps)
  )
    return true;
  return false;
}
