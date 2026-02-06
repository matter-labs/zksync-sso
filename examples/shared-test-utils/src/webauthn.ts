import type { Page } from "@playwright/test";

import type { WebAuthnCredential } from "./types";

/**
 * Virtual authenticator options for WebAuthn CDP
 */
const AUTHENTICATOR_OPTIONS = {
  protocol: "ctap2" as const,
  transport: "usb" as const,
  hasResidentKey: true,
  hasUserVerification: true,
  isUserVerified: true,
  automaticPresenceSimulation: true,
};

/**
 * Sets up a virtual WebAuthn authenticator for initial signup/account creation.
 * Listens for credential creation and returns a getter to retrieve the credential.
 *
 * @param page - The popup page where WebAuthn will be used
 * @returns Object with authenticatorId, CDP client, and getCredential function
 */
export async function setupWebAuthnForSignup(page: Page): Promise<{
  authenticatorId: string;
  client: Awaited<ReturnType<typeof page.context>["newCDPSession"]>;
  getCredential: () => WebAuthnCredential | null;
}> {
  let credential: WebAuthnCredential | null = null;

  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");

  // Listen for credential creation BEFORE adding authenticator
  client.on("WebAuthn.credentialAdded", (event: { credential: WebAuthnCredential }) => {
    console.log("New Passkey credential added");
    console.log(`  credentialId: ${event.credential.credentialId}`);
    console.log(`  isResidentCredential: ${event.credential.isResidentCredential}`);
    console.log(`  privateKey length: ${event.credential.privateKey?.length || 0}`);
    console.log(`  signCount: ${event.credential.signCount}`);
    console.log(`  rpId: ${(event.credential as any).rpId || "not set"}`);
    console.log(`  userHandle: ${(event.credential as any).userHandle || "not set"}`);
    credential = event.credential;
  });

  const result = await client.send("WebAuthn.addVirtualAuthenticator", {
    options: AUTHENTICATOR_OPTIONS,
  });

  return {
    authenticatorId: result.authenticatorId,
    client,
    getCredential: () => credential,
  };
}

/**
 * Sets up WebAuthn with an existing credential for subsequent popup interactions.
 * Used for transaction approvals, session authorizations, etc.
 *
 * @param page - The popup page where WebAuthn will be used
 * @param credential - Previously captured credential from signup
 * @param rpId - The relying party ID (defaults to 'localhost')
 */
export async function reuseCredential(
  page: Page,
  credential: WebAuthnCredential,
  rpId = "localhost",
): Promise<{
    authenticatorId: string;
    client: Awaited<ReturnType<typeof page.context>["newCDPSession"]>;
  }> {
  console.log(`reuseCredential: Setting up WebAuthn for credential ${credential.credentialId}`);
  console.log(`reuseCredential: Credential signCount=${credential.signCount}, isResident=${credential.isResidentCredential}`);

  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");
  console.log("reuseCredential: WebAuthn enabled");

  const result = await client.send("WebAuthn.addVirtualAuthenticator", {
    options: AUTHENTICATOR_OPTIONS,
  });
  console.log(`reuseCredential: Virtual authenticator added: ${result.authenticatorId}`);

  // When adding a credential, rpId MUST be set (per CDP spec)
  const credentialWithRpId = {
    ...credential,
    rpId: credential.rpId || rpId,
  };
  console.log(`reuseCredential: Using rpId=${credentialWithRpId.rpId}`);

  await client.send("WebAuthn.addCredential", {
    authenticatorId: result.authenticatorId,
    credential: credentialWithRpId,
  });
  console.log("reuseCredential: Credential added to authenticator");

  return {
    authenticatorId: result.authenticatorId,
    client,
  };
}
