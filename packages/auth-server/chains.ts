import { zksyncSepoliaTestnet } from "viem/chains";

export const chain1 = {
  ...zksyncSepoliaTestnet,
  id: 505,
  name: "ZKsync Interop",
  network: "zksync-interop-chain",
  rpcUrls: {
    default: {
      http: ["https://www.zk-interop.xyz/chain-1"],
    },
  },
};
