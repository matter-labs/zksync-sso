import type { Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import { allowlist, app } from "./app.js";
import { env, prividiumConfig, SUPPORTED_CHAINS } from "./config.js";
import { initAdminAuthService } from "./services/prividium/admin-auth.js";

async function start() {
  // Initialize Prividium admin auth at startup if enabled
  if (prividiumConfig.enabled) {
    const chain = SUPPORTED_CHAINS[0];
    if (!chain) {
      console.error("Prividium mode requires at least one configured chain");
      process.exit(1);
    }
    await initAdminAuthService(prividiumConfig, chain);
  }

  // Start server
  const port = parseInt(env.PORT, 10);
  app.listen(port, () => {
    console.log(`Auth Server API listening on port ${port}`);
    console.log(`CORS origins: ${allowlist.join(", ")}`);
    console.log(`Deployer address: ${privateKeyToAccount(env.DEPLOYER_PRIVATE_KEY as Hex).address}`);
    console.log(`RPC URL: ${env.RPC_URL}`);
  });
}

start().catch((error) => {
  console.error("Failed to start server:", error);
  process.exit(1);
});
