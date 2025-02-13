import { type Account, type Chain, type PublicActions, type Transport } from "viem";
import { estimateContractGas, estimateGas, prepareTransactionRequest } from "viem/actions";

import { type ClientWithZksyncSsoRecoveryData, signRecoveryTransaction } from "../client.js";

export function publicActionsRewrite<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: ClientWithZksyncSsoRecoveryData<transport, chain, account>,
): Pick<PublicActions<transport, chain, account>, "estimateContractGas" | "estimateGas" | "prepareTransactionRequest"> {
  return {
    prepareTransactionRequest: async (args) => {
      if (!("customSignature" in args)) {
        (args as any).customSignature = signRecoveryTransaction(client.contracts.recovery);
      }
      const request = await prepareTransactionRequest(client, {
        ...args,
        type: "eip712",
      } as any) as any;
      return request;
    },
    estimateContractGas: (args) => {
      if (!("customSignature" in args)) {
        (args as any).customSignature = signRecoveryTransaction(client.contracts.recovery);
      }
      return estimateContractGas(client, args as any);
    },
    estimateGas: async (args) => {
      if (!("customSignature" in args)) {
        (args as any).customSignature = signRecoveryTransaction(client.contracts.recovery);
      }
      const estimated = await estimateGas(client, args);
      return estimated;
    },
  };
}
