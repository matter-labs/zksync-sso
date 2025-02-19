import crypto from "crypto";
import { config } from "dotenv";
import express from "express";
import * as jose from "jose";

config();

const app = express();
const PORT = process.env.SALT_SERVICE_PORT || 3003;

const GOOGLE_JWKS_URL = new URL("https://www.googleapis.com/oauth2/v3/certs");
const GOOGLE_ISSUER = "https://accounts.google.com";

const APP_AUD = process.env.APP_AUD;
if (!APP_AUD) {
  throw new Error("APP_AUD environment variable is required but not set");
}

const SALT_ENTROPY = process.env.SALT_ENTROPY;
if (!SALT_ENTROPY) {
  throw new Error("SALT_ENTROPY environment variable is required but not set");
}

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
      audience: APP_AUD,
    });

    const iss = payload.iss as string;
    const aud = payload.aud as string;
    const sub = payload.sub as string;

    const data = Buffer.from(`${iss}${aud}${sub}${SALT_ENTROPY}`, "ascii");
    const hash = crypto.createHash("sha256").update(data).digest("hex").slice(0, 62);

    return res.json({ salt: hash });
  } catch {
    return res.status(401).json({ error: "Unauthorized - Invalid token" });
  }
});

app.listen(PORT, () => {
  console.log(`Server listening on port ${PORT}`);
});
