import { zksyncSepoliaTestnet } from "viem/chains";

export const chain1 = {
  ...zksyncSepoliaTestnet,
  id: 505,
  name: "Interop Chain 1",
  network: "zksync-interop-chain",
  rpcUrls: {
    default: {
      http: ["https://www.zk-interop.xyz/chain-1"],
    },
  },
};
