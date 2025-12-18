import type { Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import { allowlist, app } from "./app.js";
import { env } from "./config.js";

// Start server
const port = parseInt(env.PORT, 10);
app.listen(port, () => {
  console.log(`Auth Server API listening on port ${port}`);
  console.log(`CORS origins: ${allowlist.join(", ")}`);
  console.log(`Deployer address: ${privateKeyToAccount(env.DEPLOYER_PRIVATE_KEY as Hex).address}`);
  console.log(`RPC URL: ${env.RPC_URL}`);
  console.log(`Bundler URL: ${env.BUNDLER_URL}`);
});
