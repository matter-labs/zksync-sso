import dotenv from "dotenv";
import { readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
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
  CORS_ORIGINS: z.string().default("http://localhost:3003,http://localhost:3002,http://localhost:3000"),
  DEPLOYER_PRIVATE_KEY: z.string().default("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
  CHAIN_ID: z.string().default("1337"),
  RPC_URL: z.string().default("http://127.0.0.1:8545"),
  BUNDLER_URL: z.string().default("http://127.0.0.1:4337"),
  FACTORY_ADDRESS: z.string().optional(),
  EOA_VALIDATOR_ADDRESS: z.string().optional(),
  WEBAUTHN_VALIDATOR_ADDRESS: z.string().optional(),
  SESSION_VALIDATOR_ADDRESS: z.string().optional(),
});

// Parse and validate environment variables
let env: z.infer<typeof envSchema>;
try {
  env = envSchema.parse(process.env);
} catch (error) {
  console.error("Environment validation failed:", error);
  process.exit(1);
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

export { env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, SESSION_VALIDATOR_ADDRESS, WEBAUTHN_VALIDATOR_ADDRESS };
