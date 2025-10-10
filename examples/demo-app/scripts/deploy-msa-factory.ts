#!/usr/bin/env tsx
/**
 * Deploy MSA Factory and modules for demo-app
 *
 * This script deploys:
 * - EOAKeyValidator module
 * - SessionKeyValidator module
 * - WebAuthnValidator module
 * - GuardianExecutor module
 * - ModularSmartAccount implementation
 * - UpgradeableBeacon
 * - MSAFactory
 *
 * The deployed addresses are saved to contracts.json for use by the demo-app
 */

import { writeFileSync } from "fs";
import { join } from "path";
import { createWalletClient, createPublicClient, http, parseEther, type Address } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { localhost } from "viem/chains";

// Local rich wallet from zkSync in-memory node
const DEPLOYER_PRIVATE_KEY = "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";
const RPC_URL = "http://localhost:8011";
const CHAIN_ID = 260;

// Contract artifacts paths
const ERC4337_CONTRACTS_PATH = "../../packages/erc4337-contracts";

interface DeployedContracts {
  eoaValidator: Address;
  sessionValidator: Address;
  webauthnValidator: Address;
  guardianExecutor: Address;
  accountImplementation: Address;
  beacon: Address;
  factory: Address;
  chainId: number;
  rpcUrl: string;
  deployedAt: string;
}

async function main() {
  console.log("🚀 Deploying MSA Factory and modules...\n");

  // Create clients
  const account = privateKeyToAccount(DEPLOYER_PRIVATE_KEY as `0x${string}`);
  const publicClient = createPublicClient({
    chain: { ...localhost, id: CHAIN_ID },
    transport: http(RPC_URL),
  });

  const walletClient = createWalletClient({
    account,
    chain: { ...localhost, id: CHAIN_ID },
    transport: http(RPC_URL),
  });

  console.log(`📍 Deployer address: ${account.address}`);
  console.log(`🌐 RPC URL: ${RPC_URL}`);
  console.log(`⛓️  Chain ID: ${CHAIN_ID}\n`);

  // Check deployer balance
  const balance = await publicClient.getBalance({ address: account.address });
  console.log(`💰 Deployer balance: ${(Number(balance) / 1e18).toFixed(4)} ETH\n`);

  if (balance === 0n) {
    throw new Error("Deployer has no balance. Please fund the account.");
  }

  // Deploy using forge script
  console.log("📦 Deploying contracts using Forge...\n");

  const { execSync } = await import("child_process");

  try {
    // Run the forge script to deploy all contracts
    const output = execSync(
      `forge script ${ERC4337_CONTRACTS_PATH}/script/Deploy.s.sol `
      + `--rpc-url ${RPC_URL} `
      + "--broadcast "
      + `--private-key ${DEPLOYER_PRIVATE_KEY} `
      + "--sig 'deployAll()' "
      + `--chain ${CHAIN_ID} `
      + "--slow",
      { encoding: "utf-8", stdio: "pipe" },
    );

    console.log("✅ Forge deployment completed\n");
    console.log(output);

    // Parse the broadcast file to get deployed addresses
    const broadcastPath = join(
      process.cwd(),
      ERC4337_CONTRACTS_PATH,
      "broadcast/Deploy.s.sol",
      `${CHAIN_ID}`,
      "deployAll-latest.json",
    );

    console.log(`📖 Reading deployment data from: ${broadcastPath}\n`);

    const broadcastData = await import(broadcastPath);
    const transactions = broadcastData.transactions;

    // Extract contract addresses from the transactions
    // Order matches the Deploy.s.sol script:
    // 0: TransparentProxy for EOAKeyValidator
    // 1: EOAKeyValidator implementation
    // 2: TransparentProxy for SessionKeyValidator
    // 3: SessionKeyValidator implementation
    // 4: TransparentProxy for WebAuthnValidator
    // 5: WebAuthnValidator implementation
    // 6: TransparentProxy for GuardianExecutor
    // 7: GuardianExecutor implementation
    // 8: ModularSmartAccount implementation
    // 9: UpgradeableBeacon
    // 10: TransparentProxy for MSAFactory
    // 11: MSAFactory implementation
    // 12: Deployed account (if deployAll was used)

    const contracts: DeployedContracts = {
      eoaValidator: transactions[0].contractAddress as Address,
      sessionValidator: transactions[2].contractAddress as Address,
      webauthnValidator: transactions[4].contractAddress as Address,
      guardianExecutor: transactions[6].contractAddress as Address,
      accountImplementation: transactions[8].contractAddress as Address,
      beacon: transactions[9].contractAddress as Address,
      factory: transactions[10].contractAddress as Address,
      chainId: CHAIN_ID,
      rpcUrl: RPC_URL,
      deployedAt: new Date().toISOString(),
    };

    // Save to contracts.json
    const outputPath = join(process.cwd(), "contracts.json");
    writeFileSync(outputPath, JSON.stringify(contracts, null, 2));

    console.log("✅ Contract addresses saved to contracts.json\n");
    console.log("📋 Deployed Contracts:");
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    console.log(`EOAKeyValidator:      ${contracts.eoaValidator}`);
    console.log(`SessionKeyValidator:  ${contracts.sessionValidator}`);
    console.log(`WebAuthnValidator:    ${contracts.webauthnValidator}`);
    console.log(`GuardianExecutor:     ${contracts.guardianExecutor}`);
    console.log(`Account Implementation: ${contracts.accountImplementation}`);
    console.log(`UpgradeableBeacon:    ${contracts.beacon}`);
    console.log(`MSAFactory:           ${contracts.factory}`);
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    console.log("🎉 Deployment complete!\n");
    console.log("💡 The factory address can now be used in the demo-app");
    console.log(`   Factory: ${contracts.factory}\n`);
  } catch (error: any) {
    console.error("❌ Deployment failed:", error.message);
    if (error.stdout) {
      console.error("\nStdout:", error.stdout.toString());
    }
    if (error.stderr) {
      console.error("\nStderr:", error.stderr.toString());
    }
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
