import { createEnv } from "@t3-oss/env-core";
import { config } from "dotenv";
import { z } from "zod";

config();

const validNetworks = ["mainnet", "sepolia", "localhost"] as const;

export const env = createEnv({
  server: {
    FETCH_INTERVAL: z.preprocess(
      (val) => (val === undefined ? 60 * 1000 : Number(val)),
      z.number(),
    ),
    ADMIN_PRIVATE_KEY: z.string(),
    CONTRACT_ADDRESS: z.string(),
    NETWORK: z.enum(validNetworks).optional(),
    RPC_URL: z.string().url().optional(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});
