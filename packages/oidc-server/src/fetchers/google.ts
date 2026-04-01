import { bytesToHex, pad } from "viem";
import { z } from "zod";

import { GOOGLE_CERTS_URL } from "../constants.js";
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
  async fetchKeys(): Promise<BaseKey[]> {
    const response = await fetch(GOOGLE_CERTS_URL);
    if (!response.ok) throw new Error(`Google API error: ${response.status}`);

    const data = await response.json().then((keys) => keyResponseSchema.parse(keys));
    return data.keys.map((key: JWK) => ({
      kid: this.toBytes32(key.kid),
      n: key.n,
    }));
  }

  private toBytes32(str: string): string {
    const bytes = Buffer.from(str, "hex");
    return bytesToHex(pad(bytes));
  }
}
