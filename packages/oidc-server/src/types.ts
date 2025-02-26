export interface Key {
  issHash: string; // Issuer hash (as bytes32)
  kid: string; // Key ID (as bytes32)
  n: string; // RSA modulus (as bytes)
  e: string; // RSA exponent (as bytes)
}

export interface KeyFetcher {
  fetchKeys(): Promise<Key[]>;
}
