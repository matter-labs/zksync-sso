import crypto from "crypto";
import axios from "axios";
import { defineEventHandler, getHeader } from "h3";
import jwt from "jsonwebtoken";
import jwkToPem from "jwk-to-pem";

const GOOGLE_JWKS_URL = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUERS = [
  "https://accounts.google.com",
  "accounts.google.com",
];
const SALT_ENTROPY = process.env.SALT_ENTROPY || "entropy";

async function getGooglePublicKey(kid: string) {
  const { data } = await axios.get(GOOGLE_JWKS_URL);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const jwk = data.keys.find((key: any) => key.kid === kid);

  if (!jwk) {
    throw new Error("Public key not found");
  }

  return jwkToPem(jwk);
}

export default defineEventHandler(async (event) => {
  const authHeader = getHeader(event, "Authorization");

  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    throw createError({
      statusCode: 401,
      message: "Unauthorized - Missing or invalid token",
    });
  }

  const token = authHeader.split(" ")[1];

  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const decoded = jwt.decode(token, { complete: true }) as any;
    if (!decoded?.payload?.iss || !GOOGLE_ISSUERS.includes(decoded.payload.iss)) {
      throw new Error("Invalid issuer");
    }

    if (!decoded?.header?.kid) {
      throw new Error("JWT missing \"kid\"");
    }

    const publicKey = await getGooglePublicKey(decoded.header.kid);

    const verifiedToken = jwt.verify(token, publicKey, {
      algorithms: ["RS256"],
    });

    const iss = verifiedToken.iss;
    const aud = verifiedToken.aud;
    const sub = verifiedToken.sub;

    const data = { iss, aud, sub, entropy: SALT_ENTROPY };
    const hash = crypto.createHash("sha256").update(JSON.stringify(data)).digest("hex");

    return { salt: hash };
  } catch {
    throw createError({
      statusCode: 401,
      message: "Unauthorized - Invalid token or verification failed",
    });
  }
});
