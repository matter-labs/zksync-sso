import { BaseError, type ExactPartial, InvalidAddressError, InvalidChainIdError, isAddress } from "viem";
import type { ZksyncTransactionSerializable, ZksyncTransactionSerializableEIP712 } from "viem/zksync";

import { isEIP712Transaction } from "./isEip712Transaction.js";

export function assertEip712Transaction(
  transaction: ExactPartial<ZksyncTransactionSerializable>,
) {
  const { chainId, to, from, paymaster, paymasterInput }
    = transaction as ZksyncTransactionSerializableEIP712;

  if (!isEIP712Transaction(transaction))
    throw new InvalidEip712TransactionError();
  if (!chainId || chainId <= 0) throw new InvalidChainIdError({ chainId });
  if (to && !isAddress(to)) throw new InvalidAddressError({ address: to });
  if (from && !isAddress(from)) throw new InvalidAddressError({ address: from });
  if (paymaster && !isAddress(paymaster))
    throw new InvalidAddressError({ address: paymaster });
  if (paymaster && !paymasterInput) {
    throw new BaseError(
      "`paymasterInput` must be provided when `paymaster` is defined",
    );
  }
  if (!paymaster && paymasterInput) {
    throw new BaseError(
      "`paymaster` must be provided when `paymasterInput` is defined",
    );
  }
}

export type InvalidEip712TransactionErrorType =
  InvalidEip712TransactionError & {
    name: "InvalidEip712TransactionError";
  };
export class InvalidEip712TransactionError extends BaseError {
  override name = "InvalidEip712TransactionError";

  constructor() {
    super(
      [
        "Transaction is not an EIP712 transaction.",
        "",
        "Transaction must:",
        "  - include `type: \"eip712\"`",
        "  - include one of the following: `customSignature`, `paymaster`, `paymasterInput`, `gasPerPubdata`, `factoryDeps`",
      ].join("\n"),
    );
  }
}
