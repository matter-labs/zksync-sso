import type { Key, KeyFetcher } from "../types";

export class GoogleFetcher implements KeyFetcher {
  private apiUrl = "https://www.googleapis.com/oauth2/v3/certs";

  async fetchKeys(): Promise<Key[]> {
    const response = await fetch(this.apiUrl);
    if (!response.ok) throw new Error(`Google API error: ${response.status}`);

    const data = await response.json();
    return data.keys.map((key: any) => ({
      kid: this.toBytes32(key.kid),
      n: this.toHex(key.n),
      e: this.toHex(key.e),
    }));
  }

  toBytes32(str: string): string {
    return `0x${str.padStart(64, '0')}`;
  }

  toHex(str: string): string {
    return `0x${Buffer.from(str, 'base64url').toString('hex')}`;
  }
}
