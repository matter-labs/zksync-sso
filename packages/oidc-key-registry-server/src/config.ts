import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

const validNetworks = ["mainnet", "sepolia", "localhost"] as const;

export const config = createEnv({
  server: {
    FETCH_INTERVAL: z.preprocess(
      (val) => (val === undefined ? 60 * 1000 : Number(val)),
      z.number(),
    ),
    ZKSYNC_PRIVATE_KEY: z.string(),
    CONTRACT_ADDRESS: z.string(),
    NETWORK: z.enum(validNetworks).optional(),
    RPC_URL: z.string().url().optional(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});
