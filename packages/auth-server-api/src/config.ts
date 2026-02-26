import dotenv from "dotenv";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { type Chain, defineChain } from "viem";
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
  guardianExecutor?: string;
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
  GUARDIAN_EXECUTOR_ADDRESS: z.string().optional(),
  // Prividium Mode Configuration
  PRIVIDIUM_MODE: z.string().transform((v) => v === "true").default("false"),
  PRIVIDIUM_API_URL: z.string().optional(),
  PRIVIDIUM_ADMIN_PRIVATE_KEY: z.string().optional(),
  PRIVIDIUM_TEMPLATE_KEY: z.string().optional(),
  SSO_AUTH_SERVER_BASE_URL: z.string().optional(),
  // Rate Limiting Configuration
  RATE_LIMIT_DEPLOY_MAX: z.string().default("20"),
  RATE_LIMIT_DEPLOY_WINDOW_MS: z.string().default("3600000"), // 1 hour
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
  if (!env.PRIVIDIUM_API_URL) missingPrividiumVars.push("PRIVIDIUM_API_URL");
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
const GUARDIAN_EXECUTOR_ADDRESS = env.GUARDIAN_EXECUTOR_ADDRESS || contractsFromFile.guardianExecutor;

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

/**
 * Dynamically discovers and parses chain configurations from environment variables.
 * Looks for CHAIN_N_ID, CHAIN_N_RPC_URL, etc. starting from N=1.
 * Requires at least CHAIN_1_ID to be configured.
 */
function parseSupportedChains(): Chain[] {
  // Check if CHAIN_1_ID exists
  if (!process.env.CHAIN_1_ID) {
    console.error("CHAIN_1_ID is required. Please configure at least one chain using CHAIN_N_* environment variables.");
    console.error("\nExample configuration:");
    console.error("  CHAIN_1_ID=1337");
    console.error("  CHAIN_1_RPC_URL=http://localhost:8545");
    console.error("  CHAIN_1_BASE_TOKEN_DECIMALS=18  # Optional, defaults to 18");
    console.error("\nSee .env.example for more examples.");
    process.exit(1);
  }

  const chains: Chain[] = [];
  let chainIndex = 1;

  // Keep discovering chains until CHAIN_N_ID doesn't exist
  while (process.env[`CHAIN_${chainIndex}_ID`]) {
    const chainIdStr = process.env[`CHAIN_${chainIndex}_ID`];
    const rpcUrl = process.env[`CHAIN_${chainIndex}_RPC_URL`];
    const decimalsStr = process.env[`CHAIN_${chainIndex}_BASE_TOKEN_DECIMALS`];

    // Validate required fields
    if (!chainIdStr) {
      console.error(`CHAIN_${chainIndex}_ID is required but not provided`);
      process.exit(1);
    }
    if (!rpcUrl) {
      console.error(`CHAIN_${chainIndex}_RPC_URL is required but not provided`);
      process.exit(1);
    }

    // Parse and validate chain ID
    const chainId = parseInt(chainIdStr, 10);
    if (isNaN(chainId) || chainId <= 0) {
      console.error(`CHAIN_${chainIndex}_ID must be a positive integer, got: ${chainIdStr}`);
      process.exit(1);
    }

    // Parse decimals (default to 18)
    let decimals = 18;
    if (decimalsStr) {
      decimals = parseInt(decimalsStr, 10);
      if (isNaN(decimals) || decimals < 0 || decimals > 18) {
        console.error(`CHAIN_${chainIndex}_BASE_TOKEN_DECIMALS must be between 0-18, got: ${decimalsStr}`);
        process.exit(1);
      }
    }

    // Validate RPC URL format
    try {
      new URL(rpcUrl);
    } catch {
      console.error(`CHAIN_${chainIndex}_RPC_URL is not a valid URL: ${rpcUrl}`);
      process.exit(1);
    }

    // Create chain with defaults for name and currency
    const chain = defineChain({
      id: chainId,
      name: `Chain ${chainId}`,
      nativeCurrency: {
        name: "Ether",
        symbol: "ETH",
        decimals,
      },
      rpcUrls: {
        default: {
          http: [rpcUrl],
        },
      },
    });

    chains.push(chain);
    chainIndex++;
  }

  console.log(`Loaded ${chains.length} chain(s) from environment:`, chains.map((c) => `${c.name} (${c.id})`).join(", "));

  return chains;
}

// Parse supported chains from environment
const SUPPORTED_CHAINS: Chain[] = parseSupportedChains();

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
  apiUrl: string;
  adminPrivateKey: string;
  templateKey: string;
  ssoAuthServerBaseUrl: string;
  domain: string;
}

const prividiumConfig: PrividiumConfig = {
  enabled: env.PRIVIDIUM_MODE,
  apiUrl: env.PRIVIDIUM_API_URL || "",
  adminPrivateKey: env.PRIVIDIUM_ADMIN_PRIVATE_KEY || "",
  templateKey: env.PRIVIDIUM_TEMPLATE_KEY || "",
  ssoAuthServerBaseUrl: env.SSO_AUTH_SERVER_BASE_URL || "",
  domain: env.SSO_AUTH_SERVER_BASE_URL ? new URL(env.SSO_AUTH_SERVER_BASE_URL).host : "",
};

// Rate limiting configuration
const rateLimitConfig = {
  deployMax: parseInt(env.RATE_LIMIT_DEPLOY_MAX, 10),
  deployWindowMs: parseInt(env.RATE_LIMIT_DEPLOY_WINDOW_MS, 10),
};

export { env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, getChain, GUARDIAN_EXECUTOR_ADDRESS, prividiumConfig, rateLimitConfig, SESSION_VALIDATOR_ADDRESS, SUPPORTED_CHAINS, WEBAUTHN_VALIDATOR_ADDRESS };
