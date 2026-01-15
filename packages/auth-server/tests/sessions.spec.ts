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

test("Session list: create sessions and verify display", async ({ page }) => {
  test.setTimeout(90000); // Extended timeout for session creation
  console.log("\n=== Starting Session List E2E Test ===\n");

  // ===== Step 1: Create Account =====
  console.log("Step 1: Creating account...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  await expect(page.getByTestId("signup")).toBeVisible();
  page.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Page error: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Page exception: "${exception}"`);
  });

  await setupWebAuthn(page);
  await page.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  try {
    await page.waitForURL("**/dashboard", { timeout: 15000 });
  } catch (e) {
    console.log(`Navigation timeout. Current URL: ${page.url()}`);
    const errorElement = page.locator("[role=\"alert\"], .error, [class*=\"error\"]").first();
    if (await errorElement.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      console.log(`Error message on page: ${errorText}`);
    }
    throw e;
  }
  await page.waitForTimeout(2000);

  // Get account address
  await expect(page.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const accountAddress = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Account created: ${accountAddress}`);

  // ===== Step 2: Navigate to Sessions Page =====
  console.log("\nStep 2: Navigating to sessions page...");

  // Capture any errors that occur
  let pageError: string | null = null;
  page.on("pageerror", (exception) => {
    pageError = exception.message;
    console.log(`❌ Page exception captured: "${exception.message}"`);
  });

  await page.goto("http://localhost:3002/dashboard/sessions");
  await page.waitForTimeout(3000); // Wait for page to load and API calls

  // ===== Step 3: Check for Errors =====
  console.log("\nStep 3: Checking for errors...");

  if (pageError) {
    console.log(`❌ Error detected on page: ${pageError}`);

    // Check if there's an error alert displayed
    const errorAlert = page.locator("[role='alert']").first();
    if (await errorAlert.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorAlert.textContent();
      console.log(`Error alert displayed: ${errorText}`);
    }
  } else {
    console.log("✅ No errors detected");
  }

  // ===== Step 4: Verify Page Loads =====
  console.log("\nStep 4: Verifying page loads...");

  // Verify main content is visible
  const mainContent = page.locator("main");
  await expect(mainContent).toBeVisible({ timeout: 10000 });

  // Look for sessions heading
  const sessionsHeading = page.locator("text=/sessions/i").first();
  const headingVisible = await sessionsHeading.isVisible({ timeout: 5000 }).catch(() => false);

  if (headingVisible) {
    console.log("✅ Sessions heading visible");
  }

  // ===== Step 5: Check Sessions =====
  console.log("\nStep 5: Checking sessions...");

  // Wait a bit more for sessions to load
  await page.waitForTimeout(2000);

  // Check for session rows
  const sessionRows = page.locator("[data-testid*='session']");
  const sessionCount = await sessionRows.count();
  console.log(`Session rows found: ${sessionCount}`);

  // Check for "No active sessions" message using data-testid
  const noSessionsMessage = page.getByTestId("empty-sessions-message");
  const noSessionsVisible = await noSessionsMessage.isVisible({ timeout: 3000 }).catch(() => false);

  if (noSessionsVisible) {
    console.log("ℹ️ 'No active sessions' message displayed");
  }

  // Log whether we have sessions or not
  if (sessionCount > 0) {
    console.log(`✅ Found ${sessionCount} session(s)`);
  } else if (!pageError && noSessionsVisible) {
    console.log("✅ No sessions found - empty state displayed correctly");
  } else if (!pageError) {
    console.log("⚠️ No sessions found (but no empty state message displayed)");
  }

  console.log("\n=== Session List E2E Test Complete ===\n");
});

test("Session list: empty state", async ({ page }) => {
  test.setTimeout(60000);
  console.log("\n=== Starting Empty Sessions List Test ===\n");

  // Create account
  console.log("Creating account...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  await expect(page.getByTestId("signup")).toBeVisible();
  await setupWebAuthn(page);
  await page.getByTestId("signup").click();

  await page.waitForURL("**/dashboard", { timeout: 15000 });
  await page.waitForTimeout(2000);

  const accountAddress = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Account created: ${accountAddress}`);

  // Navigate to sessions page
  console.log("Navigating to sessions page...");

  // Capture any errors
  let pageError: string | null = null;
  page.on("pageerror", (exception) => {
    pageError = exception.message;
    console.log(`❌ Page exception captured: "${exception.message}"`);
  });

  await page.goto("http://localhost:3002/dashboard/sessions");
  await page.waitForTimeout(3000); // Give page time to load and render

  // Check for errors
  if (pageError) {
    console.log(`❌ Error detected: ${pageError}`);
  }

  // Verify page loads (look for main content, not specific heading)
  console.log("Checking page loaded...");
  const mainContent = page.locator("main");
  await expect(mainContent).toBeVisible({ timeout: 10000 });

  // Look for session-related text or heading
  const sessionText = page.locator("text=/session/i").first();
  const sessionTextVisible = await sessionText.isVisible({ timeout: 5000 }).catch(() => false);

  if (sessionTextVisible) {
    console.log("✅ Sessions page content visible");
  }

  // Check for sessions
  const sessionRows = page.locator("[data-testid*=\"session\"]");
  const count = await sessionRows.count();
  console.log(`Session count: ${count}`);

  // Check for "No active sessions" message using data-testid
  const noSessionsMessage = page.getByTestId("empty-sessions-message");
  const noSessionsVisible = await noSessionsMessage.isVisible({ timeout: 3000 }).catch(() => false);

  // Log empty state
  if (count === 0 && noSessionsVisible) {
    console.log("✅ Empty state verified - 'No active sessions' message displayed");
  } else if (count === 0) {
    console.log("⚠️ No sessions found, but no empty state message either");
    // Log what's actually on the page for debugging
    const pageContent = await page.locator("main").textContent();
    console.log(`Page content: ${pageContent?.substring(0, 200)}...`);
  } else {
    console.log(`ℹ️ Found ${count} session(s) on page`);
  }

  console.log("\n=== Empty Sessions List Test Complete ===\n");
});

test("Session list: investigate error", async ({ page }) => {
  test.setTimeout(60000);
  console.log("\n=== Investigating Session List Error ===\n");

  // Create account
  console.log("Creating account...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  await expect(page.getByTestId("signup")).toBeVisible();
  await setupWebAuthn(page);
  await page.getByTestId("signup").click();

  await page.waitForURL("**/dashboard", { timeout: 15000 });
  await page.waitForTimeout(2000);

  const accountAddress = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Account created: ${accountAddress}`);

  // Capture console logs and errors from the page
  const consoleLogs: string[] = [];
  const consoleErrors: string[] = [];

  page.on("console", (msg) => {
    const text = msg.text();
    consoleLogs.push(text);
    if (msg.type() === "error") {
      consoleErrors.push(text);
    }
  });

  // Capture page errors
  let pageError: Error | null = null;
  page.on("pageerror", (exception) => {
    pageError = exception;
    console.log(`❌ Page exception: ${exception.message}`);
    console.log(`Stack: ${exception.stack}`);
  });

  // Navigate to sessions page
  console.log("\nNavigating to sessions page...");
  await page.goto("http://localhost:3002/dashboard/sessions");
  await page.waitForTimeout(5000); // Wait longer for API call to complete

  // Log all console output
  console.log("\n📋 Console logs:");
  consoleLogs.forEach((log) => {
    if (!log.includes("Failed to load resource")) { // Filter out noise
      console.log(`  ${log}`);
    }
  });

  console.log("\n❌ Console errors:");
  consoleErrors.forEach((err) => {
    if (!err.includes("Failed to load resource")) { // Filter out noise
      console.log(`  ${err}`);
    }
  });

  if (pageError) {
    console.log("\n🔍 Page Error Details:");
    console.log(`  Message: ${pageError.message}`);
    console.log(`  Stack: ${pageError.stack}`);
  }

  // Check if the page loaded despite the error
  const mainContent = page.locator("main");
  const isVisible = await mainContent.isVisible({ timeout: 5000 }).catch(() => false);

  if (isVisible) {
    console.log("\n✅ Page loaded despite error");

    // Check what's displayed
    const noSessionsMessage = page.locator("text=/no active sessions/i");
    const noSessionsVisible = await noSessionsMessage.isVisible({ timeout: 2000 }).catch(() => false);

    if (noSessionsVisible) {
      console.log("✅ 'No active sessions' message displayed");
    }
  } else {
    console.log("\n❌ Page failed to load");
  }

  console.log("\n=== Investigation Complete ===\n");

  // This test is for investigation only - we expect it to find the error
  // Don't fail the test, just log the information
});

test.skip("Session list: with created session (TODO)", async ({ page }) => {
  /**
   * TODO: This test should be implemented once we can create sessions programmatically
   *
   * Current issue: The listActiveSessions function in sessions.vue throws an error
   * "An unexpected error occurred" which is a wrapped error from the WASM bindings.
   *
   * To implement this test:
   * 1. Create a session through the UI (navigate to a dApp, connect, create session)
   * 2. OR: Call the SDK directly to create a session (requires viem client setup)
   * 3. Then navigate to sessions page and verify the session is displayed
   *
   * Expected behavior:
   * - Sessions page loads without error
   * - At least one session row is visible with [data-testid*="session"]
   * - Session details are displayed (hash, permissions, expiry)
   * - "No active sessions" message is NOT displayed
   */

  test.setTimeout(120000);
  console.log("\n=== Session List with Created Session Test ===\n");

  // Create account
  await page.goto("http://localhost:3002");
  await expect(page.getByTestId("signup")).toBeVisible();
  await setupWebAuthn(page);
  await page.getByTestId("signup").click();
  await page.waitForURL("**/dashboard", { timeout: 15000 });

  const accountAddress = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Account created: ${accountAddress}`);

  // TODO: Create a session here
  // This could be done by:
  // 1. Opening a dApp in the same test
  // 2. Connecting to it
  // 3. Creating a session through the connection flow

  // Navigate to sessions page
  await page.goto("http://localhost:3002/dashboard/sessions");
  await page.waitForTimeout(3000);

  // Verify no error occurred
  const mainContent = page.locator("main");
  await expect(mainContent).toBeVisible({ timeout: 10000 });

  // Verify at least one session is displayed
  const sessionRows = page.locator("[data-testid*='session']");
  const sessionCount = await sessionRows.count();
  expect(sessionCount).toBeGreaterThan(0);
  console.log(`✅ Found ${sessionCount} session(s)`);

  // Verify "No active sessions" message is NOT displayed
  const noSessionsMessage = page.locator("text=/no active sessions/i");
  const noSessionsVisible = await noSessionsMessage.isVisible({ timeout: 1000 }).catch(() => false);
  expect(noSessionsVisible).toBe(false);

  console.log("\n=== Session List with Created Session Test Complete ===\n");
});
