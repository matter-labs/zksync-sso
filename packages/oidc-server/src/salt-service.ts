import { createEnv } from "@t3-oss/env-core";
import cors from "cors";
import crypto from "crypto";
import { config } from "dotenv";
import express from "express";
import * as jose from "jose";
import { bytesToHex } from "viem";
import { z } from "zod";

config();

const env = createEnv({
  server: {
    APP_AUD: z.string(),
    SALT_SERVICE_PORT: z.string().optional(),
    SALT_ENTROPY: z.string(),
    AUTH_SERVER_URL: z.string(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});
const GOOGLE_JWKS_URL = new URL("https://www.googleapis.com/oauth2/v3/certs");
const GOOGLE_ISSUER = "https://accounts.google.com";

const JwtPayloadSchema = z.object({
  iss: z.string(),
  aud: z.string(),
  sub: z.string(),
});

const app = express();

app.use(cors({ origin: env.AUTH_SERVER_URL }));

app.get("/salt", async (req, res): Promise<void> => {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    res.status(401).json({ error: "Unauthorized - Missing or invalid token" });
    return;
  }

  const jwt = authHeader.split(" ")[1];

  try {
    const jwks = jose.createRemoteJWKSet(GOOGLE_JWKS_URL);

    const { payload } = await jose.jwtVerify(jwt, jwks, {
      issuer: GOOGLE_ISSUER,
      audience: env.APP_AUD,
    });

    const { iss, aud, sub } = JwtPayloadSchema.parse(payload);

    const data = Buffer.from(`${iss}${aud}${sub}${env.SALT_ENTROPY}`, "ascii");

    // We use 31 byte salt in order to make it fit in a field.
    const hash = crypto.createHash("sha256").update(data).digest().subarray(1);

    res.json({ salt: bytesToHex(hash) });
  } catch {
    res.status(401).json({ error: "Unauthorized - Invalid token" });
  }
});

app.listen(env.SALT_SERVICE_PORT, () => {
  console.log(`Server listening on port ${env.SALT_SERVICE_PORT}`);
});
