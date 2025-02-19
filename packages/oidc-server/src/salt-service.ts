import { createEnv } from "@t3-oss/env-core";
import crypto from "crypto";
import { config } from "dotenv";
import express from "express";
import * as jose from "jose";
import { z } from "zod";

config();

const env = createEnv({
  server: {
    APP_AUD: z.string(),
    SALT_SERVICE_PORT: z.string().optional(),
    SALT_ENTROPY: z.string(),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});

const app = express();
const PORT = process.env.SALT_SERVICE_PORT || 3003;

const GOOGLE_JWKS_URL = new URL("https://www.googleapis.com/oauth2/v3/certs");
const GOOGLE_ISSUER = "https://accounts.google.com";

app.get("/salt", async (req, res) => {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    return res.status(401).json({ error: "Unauthorized - Missing or invalid token" });
  }

  const jwt = authHeader.split(" ")[1];

  try {
    const JWKS = jose.createRemoteJWKSet(GOOGLE_JWKS_URL);

    const { payload } = await jose.jwtVerify(jwt, JWKS, {
      issuer: GOOGLE_ISSUER,
      audience: env.APP_AUD,
    });

    const iss = payload.iss as string;
    const aud = payload.aud as string;
    const sub = payload.sub as string;

    const data = Buffer.from(`${iss}${aud}${sub}${env.SALT_ENTROPY}`, "ascii");
    const hash = crypto.createHash("sha256").update(data).digest("hex").slice(0, 62);

    return res.json({ salt: hash });
  } catch {
    return res.status(401).json({ error: "Unauthorized - Invalid token" });
  }
});

app.listen(PORT, () => {
  console.log(`Server listening on port ${env.SALT_SERVICE_PORT}`);
});
