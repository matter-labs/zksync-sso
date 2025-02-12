import crypto from "crypto";
import { defineEventHandler, getHeader } from "h3";
import * as jose from 'jose';

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


export default defineEventHandler(async (event) => {
  const authHeader = getHeader(event, "Authorization");

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    throw createError({
      statusCode: 401,
      message: "Unauthorized - Missing or invalid token",
    });
  }

  const jwt = authHeader.split(" ")[1];

  try {
    const JWKS = jose.createRemoteJWKSet(GOOGLE_JWKS_URL)

    const { payload } = await jose.jwtVerify(jwt, JWKS, {
      issuer: GOOGLE_ISSUER,
      audience: APP_AUD,
    });

    const iss = payload.iss;
    const aud = payload.aud;
    const sub = payload.sub;

    const data = `${iss}${aud}${sub}${SALT_ENTROPY}`;

    const hash = crypto.createHash("sha256").update(data).digest("hex");

    return { salt: hash, data };
  } catch {
    throw createError({
      statusCode: 401,
      message: "Unauthorized - Invalid token",
    });
  }
});
