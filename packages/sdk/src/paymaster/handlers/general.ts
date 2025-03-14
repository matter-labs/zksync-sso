import type { Address } from "viem";
import { getGeneralPaymasterInput } from "viem/zksync";

import type { CustomPaymasterHandler, CustomPaymasterHandlerResponse, CustomPaymasterParameters } from "../index.js";

export function createGeneralPaymaster(paymaster: Address): CustomPaymasterHandler {
  /* eslint-disable @typescript-eslint/no-unused-vars */
  return async (_: CustomPaymasterParameters): CustomPaymasterHandlerResponse => {
    return {
      paymaster,
      paymasterInput: getGeneralPaymasterInput({ innerInput: "0x" }),
    };
  };
}
