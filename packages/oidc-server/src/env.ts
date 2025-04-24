import { createEnv } from "@t3-oss/env-core";
import { config } from "dotenv";
import { z } from "zod";

import { hexType, validNetworks } from "./types.js";

config();

export const env = createEnv({
  server: {
    FETCH_INTERVAL: z.preprocess(
      (val) => (val === undefined ? 60 * 1000 : Number(val)),
      z.number(),
    ),
    ADMIN_PRIVATE_KEY: hexType,
    CONTRACT_ADDRESS: hexType,
    NETWORK: validNetworks,
    RPC_URL: z.string().url(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});
