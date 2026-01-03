import type { Request } from "express";
import rateLimit from "express-rate-limit";

import { rateLimitConfig } from "../config.js";

/**
 * Rate limiter for the deploy-account endpoint.
 * Uses Prividium userId when available, falls back to IP address.
 */
export const deployLimiter = rateLimit({
  windowMs: rateLimitConfig.deployWindowMs,
  max: rateLimitConfig.deployMax,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Deployment rate limit exceeded, please try again later" },
  keyGenerator: (req: Request) => {
    // Use Prividium userId if available for per-user limiting
    if (req.prividiumUser?.userId) {
      return `user:${req.prividiumUser.userId}`;
    }
    // Fall back to IP address when Prividium is disabled
    return req.ip || req.socket.remoteAddress || "unknown";
  },
});
