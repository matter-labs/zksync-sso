import type { Address, Hex, UnionRequiredBy } from "viem";
import type { TransactionRequestEIP712 } from "viem/chains";

export interface CustomPaymasterParameters {
  nonce: number;
  from: Address;
  to: Address;
  gas: bigint;
  gasPrice: bigint;
  gasPerPubdata: bigint;
  value: bigint;
  data: Hex | undefined;
  maxFeePerGas: bigint;
  maxPriorityFeePerGas: bigint;
  chainId: number;
}

export type CustomPaymasterHandlerResponse = Promise<{
  paymaster: Address;
  paymasterInput: Hex;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  gasPerPubdata?: bigint;
  gas?: bigint;
}>;

export type CustomPaymasterHandler = (
  args: CustomPaymasterParameters,
) => CustomPaymasterHandlerResponse;

export async function getTransactionWithPaymasterData(
  chainId: number,
  fromAccount: Address,
  transaction: TransactionRequestEIP712,
  customPaymasterHandler: CustomPaymasterHandler | undefined = undefined,
): Promise<
  UnionRequiredBy<TransactionRequestEIP712, "from"> & { chainId: number }
  > {
  if (
    customPaymasterHandler
    && !transaction.paymaster
    && !transaction.paymasterInput
  ) {
    const paymasterResult = await customPaymasterHandler({
      chainId,
      from: fromAccount,
      data: transaction.data,
      gas: transaction.gas ?? 0n,
      gasPrice: transaction.gasPrice ?? 0n,
      gasPerPubdata: transaction.gasPerPubdata ?? 0n,
      maxFeePerGas: transaction.maxFeePerGas ?? 0n,
      maxPriorityFeePerGas: transaction.maxPriorityFeePerGas ?? 0n,
      nonce: transaction.nonce ?? 0,
      to: transaction.to ?? "0x0",
      value: transaction.value ?? 0n,
    });
    return {
      ...transaction,
      paymaster: paymasterResult.paymaster,
      paymasterInput: paymasterResult.paymasterInput,
      gas: paymasterResult.gas ?? transaction.gas,
      maxFeePerGas: paymasterResult.maxFeePerGas ?? transaction.maxFeePerGas,
      maxPriorityFeePerGas: paymasterResult.maxPriorityFeePerGas ?? transaction.maxPriorityFeePerGas,
      gasPerPubdata: paymasterResult.gasPerPubdata ?? transaction.gasPerPubdata,
      from: fromAccount,
      chainId,
    };
  }
  return {
    ...transaction,
    from: fromAccount,
    chainId,
  };
}
