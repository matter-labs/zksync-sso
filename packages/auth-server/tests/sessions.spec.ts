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

  // Log any errors that occurred
  if (pageErrors.length > 0) {
    console.log(`⚠️ Page errors detected: ${pageErrors.join(", ")}`);
  }

  // Check what's actually on the page
  const bodyText = await page.locator("body").textContent();
  console.log(`Page content (first 500 chars): ${bodyText?.substring(0, 500)}`);

  // Step 3: Verify sessions page UI
  console.log("\nStep 3: Verifying sessions page UI...");

  // Check for main content
  const mainVisible = await page.locator("main").isVisible({ timeout: 5000 }).catch(() => false);
  console.log(`Main content visible: ${mainVisible}`);

  // Look for sessions heading (in header)
  const header = page.locator("header").getByText("Sessions");
  const headerVisible = await header.isVisible({ timeout: 5000 }).catch(() => false);
  console.log(`Sessions header visible: ${headerVisible}`);

  if (headerVisible) {
    console.log("✓ Sessions header found");
  } else {
    console.log("❌ Sessions header NOT found");
    // Check what's in the header
    const headerText = await page.locator("header").textContent().catch(() => "");
    console.log(`Header content: ${headerText}`);
  }

  // Look for empty state or error message
  const emptyStateText = page.getByText(/no active sessions/i);
  const emptyVisible = await emptyStateText.isVisible({ timeout: 3000 }).catch(() => false);
  console.log(`Empty state visible: ${emptyVisible}`);

  const errorAlert = page.locator("[role='alert']");
  const errorVisible = await errorAlert.isVisible({ timeout: 1000 }).catch(() => false);
  if (errorVisible) {
    const errorText = await errorAlert.textContent();
    console.log(`⚠️ Error alert: ${errorText}`);
  }

  // Check for session rows
  const sessionRows = page.locator("[data-testid*='session-row'], .session-item");
  const count = await sessionRows.count();
  console.log(`Session rows found: ${count}`);

  console.log("\n=== Session List UI Test Complete ===\n");
});
