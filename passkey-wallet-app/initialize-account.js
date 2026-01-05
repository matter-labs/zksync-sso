import {
  createPublicClient,
  createWalletClient,
  defineChain,
  encodeAbiParameters,
  http,
  parseAbiParameters,
  toHex,
} from "viem";
import { privateKeyToAccount } from "viem/accounts";

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
const DEPLOYER_PRIVATE_KEY = "0xef506537558847aa991149381c4fedee8fe1252cf868986ac1692336530ec85c";

const CONTRACTS = {
  passkey: "0xa5C2c5C723239C0cD11a5691954CdAC4369C874b",
};

// Account to initialize
const ACCOUNT_ADDRESS = "0x69AEC5eF03DbC61a5a183513bf0CDa2Ec8558676";

// Passkey credentials (from localStorage)
const CREDENTIAL_ID = "D03Ni5A74E1HWYcV0aPct78TOoNhHocQaWKPpSfVpNk"; // Replace with your actual credential ID
const PUBLIC_KEY_X = "0x9af587016b121b50026632305de699a2728c864a56d96658c757296fb4225820"; // Replace with your actual public key x
const PUBLIC_KEY_Y = "0xb5bb81b83db382e6056ce78ab1ce7afbf10b0f8dd8c695e6b474e3e056ae406b"; // Replace with your actual public key y
const ORIGIN = "http://localhost:3000";

async function initializeAccount() {
  console.log("🔧 Initializing account...");
  console.log(`Account: ${ACCOUNT_ADDRESS}`);

  const publicClient = createPublicClient({
    chain: zksyncOsTestnet,
    transport: http(RPC_URL),
  });

  const deployerAccount = privateKeyToAccount(DEPLOYER_PRIVATE_KEY);
  const deployerClient = createWalletClient({
    account: deployerAccount,
    chain: zksyncOsTestnet,
    transport: http(RPC_URL),
  });

  console.log(`Deployer: ${deployerAccount.address}`);

  // Check if already initialized
  console.log("\nChecking initialization status...");
  try {
    const validators = await publicClient.readContract({
      address: ACCOUNT_ADDRESS,
      abi: [{
        type: "function",
        name: "listModuleValidators",
        inputs: [],
        outputs: [{ name: "validatorList", type: "address[]" }],
        stateMutability: "view",
      }],
      functionName: "listModuleValidators",
    });
    console.log("✅ Account is already initialized!");
    console.log("Validators:", validators);
    return;
  } catch (error) {
    if (!error.message.includes("NotInitialized") && !error.message.includes("0x48c9ceda")) {
      console.error("❌ Unexpected error:", error.message);
      return;
    }
    console.log("⚠️  Account not initialized, proceeding...");
  }

  // Encode init data for WebAuthn validator
  const credentialIdHex = toHex(new TextEncoder().encode(CREDENTIAL_ID));
  const webauthnInitData = encodeAbiParameters(
    parseAbiParameters("bytes, bytes32, bytes32, string"),
    [credentialIdHex, PUBLIC_KEY_X, PUBLIC_KEY_Y, ORIGIN],
  );

  console.log("\nInitialization parameters:");
  console.log("Credential ID:", CREDENTIAL_ID);
  console.log("Public Key X:", PUBLIC_KEY_X);
  console.log("Public Key Y:", PUBLIC_KEY_Y);
  console.log("Origin:", ORIGIN);

  // Call initializeAccount
  console.log("\n📤 Sending initialization transaction...");
  const hash = await deployerClient.writeContract({
    address: ACCOUNT_ADDRESS,
    abi: [{
      type: "function",
      name: "initializeAccount",
      inputs: [
        { name: "modules", type: "address[]" },
        { name: "data", type: "bytes[]" },
      ],
      stateMutability: "nonpayable",
    }],
    functionName: "initializeAccount",
    args: [[CONTRACTS.passkey], [webauthnInitData]],
  });

  console.log(`Transaction hash: ${hash}`);
  console.log("⏳ Waiting for confirmation...");

  const receipt = await publicClient.waitForTransactionReceipt({ hash });

  if (receipt.status === "success") {
    console.log("✅ Account initialized successfully!");
    console.log(`Block: ${receipt.blockNumber}`);
    console.log(`Gas used: ${receipt.gasUsed}`);

    // Verify initialization
    const validators = await publicClient.readContract({
      address: ACCOUNT_ADDRESS,
      abi: [{
        type: "function",
        name: "listModuleValidators",
        inputs: [],
        outputs: [{ name: "validatorList", type: "address[]" }],
        stateMutability: "view",
      }],
      functionName: "listModuleValidators",
    });
    console.log("\n✅ Verified! Installed validators:", validators);
  } else {
    console.error("❌ Initialization failed!");
  }
}

initializeAccount().catch(console.error);
