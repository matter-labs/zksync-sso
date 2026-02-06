/**
 * WebAuthn credential type returned by Chrome DevTools Protocol
 */
export type WebAuthnCredential = {
  credentialId: string;
  isResidentCredential: boolean;
  privateKey: string;
  signCount: number;
  rpId?: string;
  userHandle?: string;
};

/**
 * Result from setting up WebAuthn for initial signup
 */
export type WebAuthnSetupResult = {
  authenticatorId: string;
  client: ReturnType<typeof import("@playwright/test").Page.prototype.context>;
  getCredential: () => WebAuthnCredential | null;
};
