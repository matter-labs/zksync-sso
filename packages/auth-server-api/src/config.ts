import dotenv from "dotenv";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { type Chain, defineChain } from "viem";
import { localhost } from "viem/chains";
import { z } from "zod";

// Load environment variables
dotenv.config();

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load contracts from JSON file as fallback
let contractsFromFile: {
  factory?: string;
  eoaValidator?: string;
  webauthnValidator?: string;
  sessionValidator?: string;
} = {};

try {
  const contractsPath = join(__dirname, "contracts.json");
  const contractsJson = readFileSync(contractsPath, "utf-8");
  contractsFromFile = JSON.parse(contractsJson);
} catch {
  // contracts.json is optional if all addresses are in env
  console.log("Note: contracts.json not found, will use environment variables only");
}

// Environment schema with optional contract addresses (can fall back to contracts.json)
const envSchema = z.object({
  PORT: z.string().default("3004"),
  CORS_ORIGINS: z.string().default("http://localhost:3002,http://localhost:3003,http://localhost:3004,http://localhost:3005,http://localhost:3000"),
  DEPLOYER_PRIVATE_KEY: z.string().default("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
  RPC_URL: z.string().default("http://127.0.0.1:8545"),
  FACTORY_ADDRESS: z.string().optional(),
  EOA_VALIDATOR_ADDRESS: z.string().optional(),
  WEBAUTHN_VALIDATOR_ADDRESS: z.string().optional(),
  SESSION_VALIDATOR_ADDRESS: z.string().optional(),
  // Prividium Mode Configuration
  PRIVIDIUM_MODE: z.string().transform((v) => v === "true").default("false"),
  PRIVIDIUM_PERMISSIONS_BASE_URL: z.string().optional(),
  PRIVIDIUM_RPC_PROXY_BASE_URL: z.string().optional(),
  PRIVIDIUM_ADMIN_PRIVATE_KEY: z.string().optional(),
  PRIVIDIUM_TEMPLATE_KEY: z.string().optional(),
  SSO_AUTH_SERVER_BASE_URL: z.string().optional(),
});

// Parse and validate environment variables
let env: z.infer<typeof envSchema>;
try {
  env = envSchema.parse(process.env);
} catch (error) {
  console.error("Environment validation failed:", error);
  process.exit(1);
}

// Validate Prividium configuration when enabled
if (env.PRIVIDIUM_MODE) {
  const missingPrividiumVars: string[] = [];
  if (!env.PRIVIDIUM_PERMISSIONS_BASE_URL) missingPrividiumVars.push("PRIVIDIUM_PERMISSIONS_BASE_URL");
  if (!env.PRIVIDIUM_RPC_PROXY_BASE_URL) missingPrividiumVars.push("PRIVIDIUM_RPC_PROXY_BASE_URL");
  if (!env.PRIVIDIUM_ADMIN_PRIVATE_KEY) missingPrividiumVars.push("PRIVIDIUM_ADMIN_PRIVATE_KEY");
  if (!env.PRIVIDIUM_TEMPLATE_KEY) missingPrividiumVars.push("PRIVIDIUM_TEMPLATE_KEY");
  if (!env.SSO_AUTH_SERVER_BASE_URL) missingPrividiumVars.push("SSO_AUTH_SERVER_BASE_URL");

  if (missingPrividiumVars.length > 0) {
    console.error("PRIVIDIUM_MODE is enabled but missing required configuration:", missingPrividiumVars.join(", "));
    process.exit(1);
  }
}

// Use env vars if provided, otherwise fall back to contracts.json
const FACTORY_ADDRESS = env.FACTORY_ADDRESS || contractsFromFile.factory;
const EOA_VALIDATOR_ADDRESS = env.EOA_VALIDATOR_ADDRESS || contractsFromFile.eoaValidator;
const WEBAUTHN_VALIDATOR_ADDRESS = env.WEBAUTHN_VALIDATOR_ADDRESS || contractsFromFile.webauthnValidator;
const SESSION_VALIDATOR_ADDRESS = env.SESSION_VALIDATOR_ADDRESS || contractsFromFile.sessionValidator;

// Validate that we have all required contract addresses
if (!FACTORY_ADDRESS || !EOA_VALIDATOR_ADDRESS || !WEBAUTHN_VALIDATOR_ADDRESS || !SESSION_VALIDATOR_ADDRESS) {
  console.error("Missing required contract addresses. Please provide them via environment variables or contracts.json");
  console.error("Missing:", {
    FACTORY_ADDRESS: !FACTORY_ADDRESS,
    EOA_VALIDATOR_ADDRESS: !EOA_VALIDATOR_ADDRESS,
    WEBAUTHN_VALIDATOR_ADDRESS: !WEBAUTHN_VALIDATOR_ADDRESS,
    SESSION_VALIDATOR_ADDRESS: !SESSION_VALIDATOR_ADDRESS,
  });
  process.exit(1);
}

// Supported chains configuration
const zksyncOsTestnet = defineChain({
  id: 8022833,
  name: "ZKsyncOS Testnet",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["https://zksync-os-testnet-alpha.zksync.dev"],
    },
  },
  blockExplorers: {
    default: {
      name: "ZKsyncOS Testnet Explorer",
      url: "https://zksync-os-testnet-alpha.staging-scan-v2.zksync.dev",
    },
  },
});
const zksyncOsLocal = defineChain({
  id: 6565,
  name: "ZKsyncOS Local",
  nativeCurrency: {
    name: "Ether",
    symbol: "ETH",
    decimals: 18,
  },
  rpcUrls: {
    default: {
      http: ["http://localhost:5050"],
    },
  },
});
const SUPPORTED_CHAINS: Chain[] = [localhost, zksyncOsTestnet, zksyncOsLocal];

function getChain(chainId: number): Chain {
  const chain = SUPPORTED_CHAINS.find((c) => c.id === chainId);
  if (!chain) {
    throw new Error(`Unsupported chainId: ${chainId}. Supported: ${SUPPORTED_CHAINS.map((c) => c.id).join(", ")}`);
  }
  return chain;
}

// Prividium configuration object for services
export interface PrividiumConfig {
  enabled: boolean;
  permissionsApiUrl: string;
  proxyUrl: string;
  adminPrivateKey: string;
  templateKey: string;
  ssoAuthServerBaseUrl: string;
}

const prividiumConfig: PrividiumConfig = {
  enabled: env.PRIVIDIUM_MODE,
  permissionsApiUrl: env.PRIVIDIUM_PERMISSIONS_BASE_URL || "",
  proxyUrl: env.PRIVIDIUM_RPC_PROXY_BASE_URL ? `${env.PRIVIDIUM_RPC_PROXY_BASE_URL}/rpc` : "",
  adminPrivateKey: env.PRIVIDIUM_ADMIN_PRIVATE_KEY || "",
  templateKey: env.PRIVIDIUM_TEMPLATE_KEY || "",
  ssoAuthServerBaseUrl: env.SSO_AUTH_SERVER_BASE_URL || "",
};

export { env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, getChain, prividiumConfig, SESSION_VALIDATOR_ADDRESS, SUPPORTED_CHAINS, WEBAUTHN_VALIDATOR_ADDRESS };
