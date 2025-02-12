import { defineEventHandler, getHeader } from 'h3';
import jwt from 'jsonwebtoken';
import axios from 'axios';
import jwkToPem from 'jwk-to-pem';

const GOOGLE_JWKS_URL = 'https://www.googleapis.com/oauth2/v3/certs';

async function getGooglePublicKey(kid: string) {
  const { data } = await axios.get(GOOGLE_JWKS_URL);
  const jwk = data.keys.find((key: any) => key.kid === kid);

  if (!jwk) {
    throw new Error('Public key not found');
  }

  return jwkToPem(jwk);
}

export default defineEventHandler(async (event) => {
  const authHeader = getHeader(event, 'Authorization');

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw createError({
      statusCode: 401,
      message: 'Unauthorized - Missing or invalid token',
    });
  }

  const token = authHeader.split(' ')[1];

  try {
    const decoded = jwt.decode(token, { complete: true }) as any;
    if (!decoded?.header?.kid) {
      throw new Error('JWT missing "kid"');
    }

    const publicKey = await getGooglePublicKey(decoded.header.kid);

    const verifiedToken = jwt.verify(token, publicKey, {
      algorithms: ['RS256'],
    });

    const iss = verifiedToken.iss;
    const aud = verifiedToken.aud;
    const sub = verifiedToken.sub;

    return {iss, aud, sub};
  } catch (error) {
    throw createError({
      statusCode: 401,
      message: 'Unauthorized - Invalid token or verification failed',
    });
  }
});