#!/usr/bin/env node
import * as dotenv from "dotenv";
import { createPublicClient, createWalletClient, defineChain, encodeAbiParameters, encodeFunctionData, http, keccak256, parseAbiParameters } from "viem";
import { privateKeyToAccount } from "viem/accounts";

// Load .env file
dotenv.config();

const zksyncOsTestnet = defineChain({
  id: 8022833,
  name: "ZKsync OS Developer Preview",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev/"],
      webSocket: ["wss://zksync-os-testnet-alpha.zksync.dev/ws"],
    },
    public: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev/"],
      webSocket: ["wss://zksync-os-testnet-alpha.zksync.dev/ws"],
    },
  },
  blockExplorers: {
    default: {
      name: "ZKsync OS Explorer",
      url: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
    },
  },
});

// Configuration
const RPC_URL = "https://zksync-os-testnet-alpha.zksync.dev/";
const FACTORY_ADDRESS = "0x757b5c9854d327A6B76840c996dfAac0F6b3Dc1f";

// Get private key from environment
const PRIVATE_KEY = process.env.DEPLOYER_PRIVATE_KEY;

if (!PRIVATE_KEY) {
  console.error("❌ Error: DEPLOYER_PRIVATE_KEY not found in environment");
  console.error("Please create a .env file with:");
  console.error("DEPLOYER_PRIVATE_KEY=0x...");
  process.exit(1);
}

// Get passkey data from command line arguments
const args = process.argv.slice(2);
if (args.length !== 3) {
  console.error("Usage: node deploy-account.js <credentialIdHex> <publicKeyX> <publicKeyY>");
  console.error("Example: node deploy-account.js 0x123... 0xabc... 0xdef...");
  process.exit(1);
}

const [credentialIdHex, publicKeyX, publicKeyY] = args;

// Factory ABI
const FACTORY_ABI = [
  {
    type: "function",
    name: "deployAccount",
    inputs: [
      { name: "accountId", type: "bytes32" },
      { name: "initData", type: "bytes" },
    ],
    outputs: [{ name: "account", type: "address" }],
    stateMutability: "nonpayable",
  },
  {
    type: "event",
    name: "AccountCreated",
    inputs: [
      { name: "accountId", type: "bytes32", indexed: true },
      { name: "account", type: "address", indexed: true },
    ],
  },
];

async function deployAccount() {
  console.log("🚀 Deploying Smart Account...\n");

  // Create clients
  const account = privateKeyToAccount(PRIVATE_KEY);
  const publicClient = createPublicClient({
    chain: zksyncOsTestnet,
    transport: http(RPC_URL),
  });

  const walletClient = createWalletClient({
    account,
    chain: zksyncOsTestnet,
    transport: http(RPC_URL),
  });

  console.log("Deployer Address:", account.address);

  // Check balance
  const balance = await publicClient.getBalance({ address: account.address });
  console.log("Deployer Balance:", balance.toString(), "wei");

  if (balance === 0n) {
    console.error("❌ Deployer has no ETH. Please fund:", account.address);
    process.exit(1);
  }

  // Generate account ID from credential
  const accountId = keccak256(credentialIdHex);
  console.log("\nAccount ID:", accountId);
  console.log("Status: Ready to deploy\n");

  // Encode init data for WebAuthn validator module
  const WEBAUTHN_VALIDATOR = "0xa5C2c5C723239C0cD11a5691954CdAC4369C874b";
  const origin = "http://localhost:3000"; // Change if using different origin

  // Init data for the WebAuthn validator
  const webauthnInitData = encodeAbiParameters(
    parseAbiParameters("bytes, bytes32, bytes32, string"),
    [credentialIdHex, publicKeyX, publicKeyY, origin],
  );

  // Encode initializeAccount call: initializeAccount(address[] modules, bytes[] data)
  const initData = encodeFunctionData({
    abi: [{
      type: "function",
      name: "initializeAccount",
      inputs: [
        { name: "modules", type: "address[]" },
        { name: "data", type: "bytes[]" },
      ],
    }],
    functionName: "initializeAccount",
    args: [[WEBAUTHN_VALIDATOR], [webauthnInitData]],
  });

  // Try to determine the deterministic account address up-front via simulation
  let expectedAccountAddress = null;
  try {
    const simulation = await publicClient.simulateContract({
      address: FACTORY_ADDRESS,
      abi: FACTORY_ABI,
      functionName: "deployAccount",
      args: [accountId, initData],
      account: account.address,
    });
    expectedAccountAddress = simulation.result;
    console.log("Expected Account Address:", expectedAccountAddress);

    const code = await publicClient.getCode({ address: expectedAccountAddress });
    if (code && code !== "0x") {
      console.log("✅ Account already deployed at this address.");
      console.log("No deployment needed. You can start using the account immediately.");
      return;
    }
  } catch (error) {
    console.warn("⚠️ Could not simulate deployAccount to predict address:", error?.shortMessage || error.message);
    console.warn("Proceeding with deployment without pre-check.\n");
  }

  console.log("Deploying account...");
  console.log("Factory:", FACTORY_ADDRESS);
  console.log("Account ID:", accountId);
  console.log("Init Data:", initData);
  console.log("");

  // Deploy the account
  try {
    const hash = await walletClient.writeContract({
      address: FACTORY_ADDRESS,
      abi: FACTORY_ABI,
      functionName: "deployAccount",
      args: [accountId, initData],
    });

    console.log("Transaction Hash:", hash);
    console.log("Waiting for confirmation...");

    const receipt = await publicClient.waitForTransactionReceipt({ hash });

    console.log("\n✅ Account deployed successfully!");
    console.log("Block Number:", receipt.blockNumber);
    console.log("Gas Used:", receipt.gasUsed.toString());

    // Parse logs to find AccountCreated event
    const accountCreatedLog = receipt.logs.find((log) => {
      try {
        const decoded = publicClient.decodeEventLog({
          abi: FACTORY_ABI,
          data: log.data,
          topics: log.topics,
        });
        return decoded.eventName === "AccountCreated";
      } catch {
        return false;
      }
    });

    if (accountCreatedLog) {
      const decoded = publicClient.decodeEventLog({
        abi: FACTORY_ABI,
        data: accountCreatedLog.data,
        topics: accountCreatedLog.topics,
      });
      console.log("Account Address:", decoded.args.account);
    } else if (expectedAccountAddress) {
      console.log("Account Address:", expectedAccountAddress);
    }

    console.log("\n🎉 Done! Your account is now deployed and ready to use.");
    console.log("You can now send transactions from the app.");
  } catch (error) {
    console.error("\n❌ Deployment failed:", error.message);
    if (error.cause) {
      console.error("Cause:", error.cause);
    }
    process.exit(1);
  }
}

deployAccount().catch(console.error);
