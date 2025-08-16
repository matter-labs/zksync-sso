import { createEnv } from "@t3-oss/env-core";
import cors from "cors";
import crypto from "crypto";
import { config } from "dotenv";
import express from "express";
import { rateLimit } from "express-rate-limit";
import * as jose from "jose";
import client from "prom-client";
import { bytesToHex } from "viem";
import { z } from "zod";

import { GOOGLE_CERTS_URL } from "./constants.js";

config();

const env = createEnv({
  server: {
    APP_AUD: z.string(),
    SALT_SERVICE_PORT: z.string().optional(),
    SALT_ENTROPY: z.string().regex(/0x[0-9a-fA-F]+/),
    AUTH_SERVER_URL: z.string(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});
const GOOGLE_JWKS_URL = new URL(GOOGLE_CERTS_URL);
const GOOGLE_ISSUER = "https://accounts.google.com";

const JwtPayloadSchema = z.object({
  iss: z.string(),
  aud: z.string(),
  sub: z.string(),
});

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  limit: 100, // 100 requests per window.
  standardHeaders: "draft-8",
  legacyHeaders: false,
});

const app = express();

// ----- Metrics setup -----
// Collect default metrics (Node.js process metrics, etc.)
client.collectDefaultMetrics();

// Normalize a path to reduce metrics label cardinality. Any dynamic-looking
// segment (numbers, long hex/base64-ish tokens, UUIDs) is replaced with a
// placeholder. If Express matched a route we prefer route.path which is
// already a pattern (e.g. "/user/:id").
function normalizePath(req: express.Request): string {
  // If Express has a route pattern, use it (lowest cardinality already)
  const routePath = (req as any).route?.path; // eslint-disable-line @typescript-eslint/no-explicit-any
  if (routePath) return routePath;

  const raw = req.path || "/";
  const uuidRe = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
  const hexRe = /^[0-9a-fA-F]{8,}$/; // long-ish hex
  const base64ishRe = /^[0-9a-zA-Z_-]{10,}$/; // tokens
  return raw
    .split("/")
    .map((seg) => {
      if (!seg) return seg;
      if (/^[0-9]+$/.test(seg)) return ":int";
      if (uuidRe.test(seg)) return ":uuid";
      if (hexRe.test(seg)) return ":hex";
      if (base64ishRe.test(seg) && seg.length > 16) return ":tok";
      return seg;
    })
    .join("/") || "/";
}

// Custom metrics
const requestCounter = new client.Counter({
  name: "salt_service_requests_total",
  help: "Total number of requests received by path and method",
  labelNames: ["method", "path", "status"],
});
const requestDuration = new client.Histogram({
  name: "salt_service_request_duration_seconds",
  help: "Histogram of request durations in seconds",
  labelNames: ["method", "path", "status"],
  buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2, 5],
});

// Middleware to record metrics
app.use((req, res, next) => {
  const end = requestDuration.startTimer();
  res.on("finish", () => {
    const labels = { method: req.method, path: normalizePath(req), status: res.statusCode.toString() };
    requestCounter.inc(labels);
    end(labels);
  });
  next();
});

app.use(cors({ origin: env.AUTH_SERVER_URL }));
app.use(limiter);

// Health check endpoint for Kubernetes
app.get("/health", (req, res) => {
  res.status(200).json({ status: "healthy", timestamp: new Date().toISOString() });
});

app.get("/salt", async (req, res): Promise<void> => {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    res.status(401).json({ error: "Unauthorized - Missing or invalid token" });
    return;
  }

  const jwt = authHeader.split(" ")[1];

  let payload;
  try {
    const jwks = jose.createRemoteJWKSet(GOOGLE_JWKS_URL);
    const parsedJwt = await jose.jwtVerify(jwt, jwks, {
      issuer: GOOGLE_ISSUER,
      audience: env.APP_AUD,
    });
    payload = parsedJwt.payload;
  } catch {
    res.status(401).json({ error: "Unauthorized - Invalid token" });
    return;
  }

  const { iss, aud, sub } = JwtPayloadSchema.parse(payload);

  const data = Buffer.concat([
    Buffer.from(iss, "ascii"),
    Buffer.from(aud, "ascii"),
    Buffer.from(sub, "ascii"),
    Buffer.from(env.SALT_ENTROPY, "hex"),
  ]);

  // We use 31 byte salt in order to make it fit in a field.
  const hash = crypto.createHash("sha256").update(data).digest().subarray(1);

  res.json({ salt: bytesToHex(hash) });
});

const mainPort = env.SALT_SERVICE_PORT || "3003";
app.listen(mainPort, () => {
  console.log(`Server listening on port ${mainPort}`);
});

// Separate metrics server on port 9090
const METRICS_PORT = process.env.METRICS_PORT || 9090;
const metricsApp = express();
metricsApp.get("/metrics", async (_req, res) => {
  try {
    res.set("Content-Type", client.register.contentType);
    res.end(await client.register.metrics());
  } catch (err) {
    res.status(500).end((err as Error).message);
  }
});
metricsApp.get("/health", async (_req, res) => {
  res.json({ status: "ok" });
});
metricsApp.listen(METRICS_PORT, () => {
  console.log(`Metrics server listening on port ${METRICS_PORT}`);
});
