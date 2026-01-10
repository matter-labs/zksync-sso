import type { NextFunction, Request, Response } from "express";

import type { PrividiumConfig } from "../config.js";
import { verifyUserAuth } from "../services/prividium/index.js";

// Extend Express Request type to include Prividium user info
declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Express {
    interface Request {
      prividiumUser?: {
        userId: string;
      };
    }
  }
}

/**
 * Creates Express middleware that verifies Prividium user authentication.
 *
 * When PRIVIDIUM_MODE is enabled:
 * - Requires Authorization header
 * - Verifies the token via Prividium permissions API
 * - Attaches userId to request for downstream use
 *
 * When PRIVIDIUM_MODE is disabled:
 * - Passes through without any checks
 *
 * @param config Prividium configuration
 * @returns Express middleware function
 */
export function prividiumAuthMiddleware(config: PrividiumConfig) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    // Skip if Prividium mode is disabled
    if (!config.enabled) {
      next();
      return;
    }

    // Check for Authorization header
    const authHeader = req.headers.authorization;
    if (!authHeader) {
      res.status(401).json({ error: "Authorization header required" });
      return;
    }

    // Verify user token with Prividium
    const authResult = await verifyUserAuth(authHeader, config.permissionsApiUrl);

    if (!authResult.valid || !authResult.userId) {
      if (authResult.error === "network_error") {
        res.status(503).json({ error: "Authentication service unavailable" });
        return;
      }
      if (authResult.error === "server_error") {
        res.status(502).json({ error: "Authentication service error" });
        return;
      }
      // invalid_token or unknown error
      res.status(401).json({ error: "Invalid or expired token" });
      return;
    }

    // Attach user info to request
    req.prividiumUser = {
      userId: authResult.userId,
    };

    next();
  };
}
