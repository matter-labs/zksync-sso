/* eslint-disable no-console, simple-import-sort/imports */
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

type WebAuthnCredential = {
  credentialId: string;
  isResidentCredential: boolean;
  privateKey: string;
  signCount: number;
};

async function waitForAuthServerToLoad(page: Page): Promise<void> {
  const maxRetryAttempts = 10;
  let retryCount = 0;

  // Wait for auth server to finish loading
  await page.goto("http://localhost:3002");
  let authServerHeader = page.getByTestId("signup");
  while (!(await authServerHeader.isVisible()) && retryCount < maxRetryAttempts) {
    await page.waitForTimeout(1000);
    authServerHeader = page.getByTestId("signup");
    retryCount++;

    console.log(`Waiting for auth server to load (retry ${retryCount})...`);
  }
  console.log("Auth Server loaded");
}

async function setupWebAuthn(page: Page): Promise<{
  authenticatorId: string;
  newCredential: WebAuthnCredential | null;
  client: unknown;
}> {
  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");

  let newCredential: WebAuthnCredential | null = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    newCredential = credentialAdded.credential as WebAuthnCredential;
    console.log(`Credential added: ${newCredential.credentialId}`);
  });

  const result = await client.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "usb",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });

  return { authenticatorId: result.authenticatorId, newCredential, client };
}

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Page error: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Page uncaught exception: "${exception}"`);
  });

  await waitForAuthServerToLoad(page);
});

test("Session list: verify sessions page loads and displays correctly", async ({ page }) => {
  console.log("\n=== Session List UI Test ===\n");

  // Step 1: Create account and login
  console.log("Step 1: Creating account...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  await expect(page.getByTestId("signup")).toBeVisible();
  await setupWebAuthn(page);
  await page.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  await page.waitForURL("**/dashboard", { timeout: 15000 });
  await page.waitForTimeout(1000);
  console.log("✓ Account created and logged in");

  // Step 2: Navigate to sessions page
  console.log("Step 2: Navigating to sessions page...");

  // Capture page errors
  const pageErrors: string[] = [];
  page.on("pageerror", (err) => {
    pageErrors.push(err.message);
  });

  await page.goto("http://localhost:3002/dashboard/sessions");

  // Wait for DOM to be ready instead of networkidle (which may fail due to API errors)
  await page.waitForLoadState("domcontentloaded");
  await page.waitForTimeout(3000);

  console.log("Page loaded, checking content...");

  // Assert no page errors occurred
  expect(pageErrors, "No page errors should occur during load").toHaveLength(0);

  // Step 3: Verify sessions page UI
  console.log("\nStep 3: Verifying sessions page UI...");

  // Assert main content is visible
  const main = page.locator("main");
  await expect(main).toBeVisible({ timeout: 5000 });

  // Assert sessions heading is present
  const header = page.locator("header").getByText("Sessions");
  await expect(header).toBeVisible({ timeout: 5000 });

  // Assert empty state is visible (since we haven't created any sessions yet)
  const emptyStateText = page.getByText(/no active sessions/i);
  await expect(emptyStateText).toBeVisible({ timeout: 3000 });

  // Assert no error alerts are shown
  const errorAlert = page.locator("[role='alert']");
  await expect(errorAlert).not.toBeVisible({ timeout: 1000 });

  // Assert no session rows are displayed (empty state)
  const sessionRows = page.locator(".session-row");
  const count = await sessionRows.count();
  expect(count, "No sessions should be displayed in empty state").toBe(0);

  console.log("✓ All assertions passed");
  console.log("\n=== Session List UI Test Complete ===\n");
});
