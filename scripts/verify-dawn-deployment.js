/**
 * Verification script for Dawn Mainnet deployment
 * Checks that all contracts are deployed correctly and accessible
 */

import { ethers } from "ethers";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { Provider, Wallet } from "zksync-ethers";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DAWN_MAINNET_RPC = "https://zksync-os-mainnet-dawn.zksync.io";
const DEPLOYMENT_FILE = path.join(__dirname, "../packages/auth-server/stores/dawn-mainnet.json");

async function main() {
  console.log("🔍 Verifying Dawn Mainnet Deployment...\n");

  // Check if deployment file exists
  if (!fs.existsSync(DEPLOYMENT_FILE)) {
    throw new Error(`Deployment file not found: ${DEPLOYMENT_FILE}`);
  }

  const contracts = JSON.parse(fs.readFileSync(DEPLOYMENT_FILE, "utf8"));
  const provider = new Provider(DAWN_MAINNET_RPC);

  console.log("📋 Checking deployed contracts:\n");

  const contractNames = {
    factory: "Factory",
    webauthnValidator: "WebAuthn Validator",
    sessionValidator: "Session Validator",
    eoaValidator: "EOA Validator",
    guardianExecutor: "Guardian Executor",
    beacon: "Beacon",
    accountImplementation: "Account Implementation",
    entryPoint: "Entry Point",
  };

  let allValid = true;

  for (const [key, name] of Object.entries(contractNames)) {
    const address = contracts[key];
    if (!address) {
      console.log(`❌ ${name}: NOT DEPLOYED`);
      allValid = false;
      continue;
    }

    try {
      const code = await provider.getCode(address);
      if (code === "0x" || code === "0x0") {
        console.log(`❌ ${name}: No code at ${address}`);
        allValid = false;
      } else {
        console.log(`✅ ${name}: ${address}`);
      }
    } catch (error) {
      console.log(`❌ ${name}: Error checking ${address} - ${error.message}`);
      allValid = false;
    }
  }

  console.log("\n📊 Network Information:");
  const blockNumber = await provider.getBlockNumber();
  console.log(`  Current block: ${blockNumber}`);

  if (process.env.WALLET_PRIVATE_KEY) {
    const wallet = new Wallet(process.env.WALLET_PRIVATE_KEY, provider);
    const balance = await wallet.getBalance();
    console.log(`  Deployer balance: ${ethers.formatEther(balance)} ETH`);
  }

  // Check paymaster balance if deployed
  if (contracts.accountPaymaster) {
    try {
      const paymasterBalance = await provider.getBalance(contracts.accountPaymaster);
      console.log(`  Paymaster balance: ${ethers.formatEther(paymasterBalance)} ETH`);

      if (paymasterBalance === 0n) {
        console.log(`\n⚠️  Warning: Paymaster has zero balance. Fund it to enable sponsored transactions.`);
      }
    } catch (error) {
      console.log(`  ⚠️  Could not check paymaster balance: ${error.message}`);
    }
  }

  if (!allValid) {
    console.log("\n❌ Deployment verification FAILED");
    process.exit(1);
  }

  console.log("\n✅ Deployment verification PASSED");
  console.log("\n📝 Next steps:");
  console.log("1. Add these contract addresses to packages/auth-server/stores/client.ts");
  console.log("2. If paymaster balance is low, fund it with: pnpm hardhat fund-paymaster --network dawnMainnet");
  console.log("3. Test account creation and transactions");
  console.log("4. Deploy bundler service with these contract addresses");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("\n❌ Error:", error.message);
    process.exit(1);
  });
