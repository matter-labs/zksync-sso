#!/usr/bin/env node
import { createPublicClient, http } from "viem";
import { localhost } from "viem/chains";

const publicClient = createPublicClient({
  chain: localhost,
  transport: http("http://127.0.0.1:8545"),
});

const GUARDIAN_EXECUTOR = "0x7fB73973d71dA68b5B7b9D29AEc7d278A16aA661";
const DEPLOYED_ACCOUNT = "0xc0765cdb762d96e7e6562399c9db229565ed5747";

async function test() {
  console.log("\n=== Testing GuardianExecutor Installation ===\n");

  // 1. Check if GuardianExecutor contract exists
  const guardianCode = await publicClient.getCode({ address: GUARDIAN_EXECUTOR });
  console.log(`1. GuardianExecutor contract deployed: ${!!guardianCode && guardianCode !== "0x"}`);

  if (!guardianCode || guardianCode === "0x") {
    console.log("❌ GuardianExecutor not deployed!");
    return;
  }

  // 2. Check if GuardianExecutor reports correct module type
  const isExecutor = await publicClient.readContract({
    address: GUARDIAN_EXECUTOR,
    abi: [{
      type: "function",
      name: "isModuleType",
      inputs: [{ name: "moduleType", type: "uint256" }],
      outputs: [{ name: "", type: "bool" }],
      stateMutability: "pure",
    }],
    functionName: "isModuleType",
    args: [2n], // MODULE_TYPE_EXECUTOR = 2
  });
  console.log(`2. GuardianExecutor.isModuleType(EXECUTOR): ${isExecutor}`);

  // 3. Check if account exists
  const accountCode = await publicClient.getCode({ address: DEPLOYED_ACCOUNT });
  console.log(`3. Account ${DEPLOYED_ACCOUNT} exists: ${!!accountCode && accountCode !== "0x"}`);

  if (!accountCode || accountCode === "0x") {
    console.log("❌ Account not deployed!");
    return;
  }

  // 4. Check if module is installed on account
  const isInstalled = await publicClient.readContract({
    address: DEPLOYED_ACCOUNT,
    abi: [{
      type: "function",
      name: "isModuleInstalled",
      inputs: [
        { name: "moduleType", type: "uint256" },
        { name: "module", type: "address" },
        { name: "additionalContext", type: "bytes" },
      ],
      outputs: [{ name: "", type: "bool" }],
      stateMutability: "view",
    }],
    functionName: "isModuleInstalled",
    args: [2n, GUARDIAN_EXECUTOR, "0x"],
  });
  console.log(`4. Account.isModuleInstalled(EXECUTOR, ${GUARDIAN_EXECUTOR}): ${isInstalled}`);

  if (!isInstalled) {
    console.log("\n❌ GuardianExecutor NOT installed despite being in deployment calldata!");
    console.log("This suggests initializeAccount is not installing it.");
  } else {
    console.log("\n✅ GuardianExecutor successfully installed!");
  }
}

test().catch(console.error);
