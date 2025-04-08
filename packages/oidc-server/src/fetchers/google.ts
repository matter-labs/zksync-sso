import { z } from "zod";

import type { BaseKey, KeyFetcher } from "../types.js";

const jwkSchema = z.object({
  kid: z.string(),
  n: z.string(),
  e: z.string(),
});

const keyResponseSchema = z.object({
  keys: z.array(jwkSchema),
});

type JWK = z.infer<typeof jwkSchema>;

export class GoogleFetcher implements KeyFetcher {
  private apiUrl = "https://www.googleapis.com/oauth2/v3/certs";

  async fetchKeys(): Promise<BaseKey[]> {
    const response = await fetch(this.apiUrl);
    if (!response.ok) throw new Error(`Google API error: ${response.status}`);

    const data = await response.json().then((keys) => keyResponseSchema.parse(keys));
    return data.keys.map((key: JWK) => ({
      kid: this.toBytes32(key.kid),
      n: key.n,
      e: this.toHex(key.e),
    }));
  }

  private toBytes32(str: string): string {
    return `0x${str.padStart(64, "0")}`;
  }

  private toHex(str: string): string {
    return `0x${Buffer.from(str, "base64url").toString("hex")}`;
  }
}
