import dotenv from "dotenv";
import { writeFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";
import { z } from "zod";

// Load environment variables
dotenv.config();

const __dirname = dirname(fileURLToPath(import.meta.url));

// Environment schema with defaults from the local zksync-os bundler config.
const envSchema = z.object({
  EXECUTOR_PRIVATE_KEY: z
    .string()
    .default("0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"),
  UTILITY_PRIVATE_KEY: z
    .string()
    .default("0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"),
  RPC_URL: z.string().default("http://localhost:3050"),
});

// Parse and validate environment variables
let env: z.infer<typeof envSchema>;
try {
  env = envSchema.parse(process.env);
} catch (error) {
  console.error("Environment validation failed:", error);
  process.exit(1);
}

// Generate Alto config JSON file
const altoConfig = {
  entrypoints: "0x4337084D9E255Ff0702461CF8895CE9E3b5Ff108",
  "executor-private-keys": env.EXECUTOR_PRIVATE_KEY,
  "utility-private-key": env.UTILITY_PRIVATE_KEY,
  "rpc-url": env.RPC_URL,
  port: 4338,
  "safe-mode": false,
};

// Write config file (will be read by Alto bundler)
const configPath = join(__dirname, "..", "alto-config.json");
try {
  writeFileSync(configPath, JSON.stringify(altoConfig, null, 2));
  console.log(`Generated Alto config at: ${configPath}`);
} catch (error) {
  console.error("Failed to write Alto config:", error);
  process.exit(1);
}

export { altoConfig, env };
