import type { Address } from "viem";

export type AddressByChain = {
  [k in SupportedChainId]: Address;
};
