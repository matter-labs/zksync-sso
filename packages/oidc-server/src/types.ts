export interface Key {
  kid: string; // Key ID (as bytes32)
  n: string; // RSA modulus (as bytes)
  e: string; // RSA exponent (as bytes)
}

export interface KeyFetcher {
  fetchKeys(): Promise<Key[]>;
}
