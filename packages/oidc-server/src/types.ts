export interface BaseKey {
  kid: string; // Key ID (as bytes32)
  n: string; // RSA modulus (as bytes)
  e: string; // RSA exponent (as bytes)
}

export interface Key extends BaseKey {
  issHash: string; // Issuer hash (as bytes32)
}

export interface KeyFetcher {
  fetchKeys(): Promise<BaseKey[]>;
}
