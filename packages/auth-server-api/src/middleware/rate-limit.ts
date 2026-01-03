import type { Request } from "express";
import rateLimit from "express-rate-limit";

import { rateLimitConfig } from "../config.js";

/**
 * Rate limiter for the deploy-account endpoint.
 * Runs before authentication to avoid expensive operations when rate limited.
 * Uses IP-based limiting.
 */
export const deployLimiter = rateLimit({
  windowMs: rateLimitConfig.deployWindowMs,
  max: rateLimitConfig.deployMax,
  standardHeaders: true,
  legacyHeaders: false,
  message: { error: "Deployment rate limit exceeded, please try again later" },
  keyGenerator: (req: Request) => {
    return req.ip || req.socket.remoteAddress || "unknown";
  },
});
