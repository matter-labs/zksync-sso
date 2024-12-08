import { type Account, type Chain, encodeFunctionData, type Hex, type PublicActions, type Transport } from "viem";
import { estimateContractGas, estimateGas, prepareTransactionRequest } from "viem/actions";

import { type ClientWithZksyncSsoSessionData, signSessionTransaction } from "../clients/session.js";

const emptySignature = "0x" + "1b".padStart(65 * 2, "0") as Hex;

export function publicActionsRewrite<
  transport extends Transport,
  chain extends Chain,
  account extends Account,
>(
  client: ClientWithZksyncSsoSessionData<transport, chain, account>,
): Pick<PublicActions<transport, chain, account>, "estimateContractGas" | "estimateGas" | "prepareTransactionRequest"> {
  return {
    prepareTransactionRequest: async (args) => {
      if (!("customSignature" in args)) {
        (args as any).customSignature = signSessionTransaction({
          sessionKeySignedHash: emptySignature,
          sessionContract: client.contracts.session,
          sessionConfig: client.sessionConfig,
          to: args.to!,
          callData: args.data,
        });
      }
      const request = await prepareTransactionRequest(client, {
        ...args,
        type: "eip712",
      } as any) as any;
      return request;
    },
    estimateContractGas: (args) => {
      if (!("customSignature" in args)) {
        const callData = encodeFunctionData({
          abi: args.abi as any,
          functionName: args.functionName,
          args: (args.args as unknown[]) || [],
        });
        (args as any).customSignature = signSessionTransaction({
          sessionKeySignedHash: emptySignature,
          sessionContract: client.contracts.session,
          sessionConfig: client.sessionConfig,
          to: args.address,
          callData,
        });
      }
      return estimateContractGas(client, args as any);
    },
    estimateGas: async (args) => {
      if (!("customSignature" in args)) {
        (args as any).customSignature = signSessionTransaction({
          sessionKeySignedHash: emptySignature,
          sessionContract: client.contracts.session,
          sessionConfig: client.sessionConfig,
          to: args.to!,
          callData: args.data,
        });
      }
      const estimated = await estimateGas(client, args);
      return estimated;
    },
  };
}
