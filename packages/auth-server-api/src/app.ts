import cors from "cors";
import express, { type NextFunction, type Request, type Response } from "express";

import { CHAIN_ID, ENTRYPOINT_ADDRESS, env, EOA_VALIDATOR_ADDRESS, FACTORY_ADDRESS, SESSION_VALIDATOR_ADDRESS, WEBAUTHN_VALIDATOR_ADDRESS } from "./config.js";
import { deployAccountHandler } from "./handlers/deploy-account.js";

// Initialize Express app
const app = express();

// Middleware
app.use(express.json());

// CORS configuration
const corsOrigins = env.CORS_ORIGINS.split(",").map((origin) => origin.trim());
app.use(
  cors({
    origin: corsOrigins,
    methods: ["GET", "POST"],
    credentials: true,
  }),
);

// Health check endpoint
app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ status: "ok", timestamp: new Date().toISOString() });
});

// Validation endpoint - returns current contract addresses
app.get("/api/config", (_req: Request, res: Response) => {
  res.json({
    chainId: CHAIN_ID,
    rpcUrl: env.RPC_URL,
    bundlerUrl: env.BUNDLER_URL,
    contracts: {
      factory: FACTORY_ADDRESS,
      webauthnValidator: WEBAUTHN_VALIDATOR_ADDRESS,
      eoaValidator: EOA_VALIDATOR_ADDRESS,
      sessionValidator: SESSION_VALIDATOR_ADDRESS,
      entryPoint: ENTRYPOINT_ADDRESS,
    },
    timestamp: new Date().toISOString(),
  });
});

// Deploy account endpoint
app.post("/api/deploy-account", deployAccountHandler);

// Global error handler
// eslint-disable-next-line @typescript-eslint/no-unused-vars
app.use((err: Error, _req: Request, res: Response, _next: NextFunction) => {
  console.error("Unhandled error:", err);
  res.status(500).json({ error: "Internal server error" });
});

// 404 handler
app.use((_req: Request, res: Response) => {
  res.status(404).json({ error: "Not found" });
});

export { app, corsOrigins };
