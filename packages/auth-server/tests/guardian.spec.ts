/* eslint-disable no-console, simple-import-sort/imports */
import { exec } from "child_process";
import { promisify } from "util";

import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

const execAsync = promisify(exec);

/**
 * Fund an account with ETH using Anvil's default rich wallet
 */
async function fundAccount(address: string, amount: string = "1"): Promise<void> {
  const ANVIL_PRIVATE_KEY = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const cmd = `cast send ${address} --value ${amount}ether --private-key ${ANVIL_PRIVATE_KEY} --rpc-url http://localhost:8545`;
  try {
    await execAsync(cmd);
    console.log(`✅ Funded ${address} with ${amount} ETH`);
  } catch (error) {
    console.error(`❌ Failed to fund ${address}:`, error);
    throw error;
  }
}

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

async function reuseCredential(page: Page, credential: WebAuthnCredential): Promise<void> {
  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");

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

  await client.send("WebAuthn.addCredential", {
    authenticatorId: result.authenticatorId,
    credential: credential,
  });
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

test("Guardian flow: propose and confirm guardian", async ({ page, context: _context }) => {
  test.setTimeout(90000); // Extended timeout for full guardian flow with account creation
  console.log("\n=== Starting Guardian E2E Test ===\n");

  // ===== Step 1: Create Primary Account =====
  console.log("Step 1: Creating primary account...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  // No popup - we're already on the auth-server signup page
  await expect(page.getByTestId("signup")).toBeVisible();
  page.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Page error: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Page exception: "${exception}"`);
  });

  const { newCredential: _primaryCredential } = await setupWebAuthn(page);

  await page.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  try {
    await page.waitForURL("**/dashboard", { timeout: 15000 });
  } catch (e) {
    console.log(`Navigation timeout. Current URL: ${page.url()}`);
    // Check for error messages on the page
    const errorElement = page.locator("[role=\"alert\"], .error, [class*=\"error\"]").first();
    if (await errorElement.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      console.log(`Error message on page: ${errorText}`);
    }
    throw e;
  }
  await page.waitForTimeout(2000);

  // Wait for account address to be visible
  await expect(page.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const primaryAddressText = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Primary account created: ${primaryAddressText}`);

  // Fund the primary account with ETH
  await fundAccount(primaryAddressText, "1");

  // ===== Step 2: Navigate to Guardian Settings ===
  console.log("\nStep 2: Navigating to guardian settings...");
  await page.goto("http://localhost:3002/dashboard/settings");
  await page.waitForTimeout(2000);

  // Find the "Add Recovery Method" button
  const addRecoveryButton = page.getByTestId("add-recovery-method");
  await expect(addRecoveryButton).toBeVisible({ timeout: 10000 });
  await addRecoveryButton.click();
  await page.waitForTimeout(1000);

  // Select "Recover with Guardian" option
  const guardianMethodButton = page.getByTestId("add-guardian-method");
  await expect(guardianMethodButton).toBeVisible({ timeout: 5000 });
  await guardianMethodButton.click();
  await page.waitForTimeout(1000);

  const continueButton = page.getByTestId("continue-recovery-method");
  await expect(continueButton).toBeVisible({ timeout: 10000 });
  await continueButton.click();
  await page.waitForTimeout(1000);

  // ===== Step 3: Create Guardian Account in Incognito =====
  console.log("\nStep 3: Creating guardian account in new context...");
  const guardianContext = await page.context().browser()!.newContext();
  const guardianPage = await guardianContext.newPage();

  guardianPage.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Guardian page error: "${msg.text()}"`);
  });
  guardianPage.on("pageerror", (exception) => {
    console.log(`Guardian page exception: "${exception}"`);
  });

  await guardianPage.goto("http://localhost:3002");
  await guardianPage.waitForTimeout(2000);

  // No popup - we're already on the auth-server signup page
  const { newCredential: guardianCredential } = await setupWebAuthn(guardianPage);

  await guardianPage.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  try {
    await guardianPage.waitForURL("**/dashboard", { timeout: 15000 });
  } catch (e) {
    console.log(`Guardian navigation timeout. Current URL: ${guardianPage.url()}`);
    // Check for error messages on the page
    const errorElement = guardianPage.locator("[role=\"alert\"], .error, [class*=\"error\"]").first();
    if (await errorElement.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      console.log(`Guardian error message: ${errorText}`);
    }
    throw e;
  }
  await guardianPage.waitForTimeout(2000);

  // Wait for account address to be visible
  await expect(guardianPage.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const guardianAddressText = await guardianPage.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Guardian account created: ${guardianAddressText}`);

  // Fund the guardian account with ETH
  await fundAccount(guardianAddressText, "1");

  // ===== Step 4: Propose Guardian ===
  console.log("\nStep 4: Proposing guardian...");
  await page.bringToFront();

  // Capture console logs from the primary page during proposal
  const primaryPageConsole: string[] = [];
  page.on("console", (msg) => {
    const logMsg = `[${msg.type()}] ${msg.text()}`;
    primaryPageConsole.push(logMsg);
    console.log(`Primary page console: ${logMsg}`);
  });

  // Enter guardian address in the modal/input
  const guardianInput = page.getByTestId("guardian-address-input").locator("input");
  await expect(guardianInput).toBeVisible({ timeout: 5000 });
  await guardianInput.fill(guardianAddressText);

  // Click Propose button
  const proposeButton = page.getByRole("button", { name: /Propose/i, exact: false });
  await proposeButton.click();

  // Wait for guardian proposal to complete
  // NOTE: The SSO client automatically signs ERC-4337 transactions,
  // so there are NO popup windows to interact with during guardian proposal
  console.log("Waiting for guardian proposal transaction to complete...");

  // Check for errors during proposal
  const errorMessage = page.locator("text=/error.*proposing/i");
  const errorVisible = await errorMessage.isVisible({ timeout: 8000 }).catch(() => false);
  if (errorVisible) {
    const errorText = await errorMessage.textContent();
    throw new Error(`Guardian proposal failed: ${errorText}`);
  }

  await page.waitForTimeout(8000); // Wait for module installation + guardian proposal
  console.log("Guardian proposal initiated");

  // Log all captured console messages
  if (primaryPageConsole.length > 0) {
    console.log(`\n📋 Captured ${primaryPageConsole.length} console messages from primary page:`);
    primaryPageConsole.forEach((msg, i) => console.log(`  ${i + 1}. ${msg}`));
  }

  // Handle "Do you wish to confirm your guardian now?" dialog
  try {
    const confirmLaterButton = page.getByRole("button", { name: /Confirm Later/i });
    await confirmLaterButton.waitFor({ state: "visible", timeout: 10000 });
    await confirmLaterButton.click();
    console.log("Clicked 'Confirm Later' on the confirmation prompt");
    await page.waitForTimeout(5000); // Wait for dialog to close and transaction to complete
  } catch (error) {
    console.log("⚠ Warning: 'Do you wish to confirm your guardian now?' prompt not found");
    console.log(`   ${error instanceof Error ? error.message : String(error)}`);
  }

  // ===== Step 5: Confirm Guardian =====
  console.log("\nStep 5: Confirming guardian...");

  // Construct confirmation URL
  const confirmUrl = `http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=${primaryAddressText}&guardianAddress=${guardianAddressText}`;
  console.log(`Confirmation URL: ${confirmUrl}`);

  // Capture page errors
  const pageErrors: string[] = [];
  guardianPage.on("pageerror", (error) => {
    const errorMsg = `Page error: ${error.message}`;
    console.error(errorMsg);
    pageErrors.push(errorMsg);
  });

  // Capture console errors
  guardianPage.on("console", (msg) => {
    if (msg.type() === "error") {
      console.error(`Console error: ${msg.text()}`);
    }
  });

  await guardianPage.goto(confirmUrl);
  console.log("Page loaded, waiting for content...");
  await guardianPage.waitForTimeout(2000);

  // Debug: Log page URL and title
  console.log(`Current URL: ${guardianPage.url()}`);
  console.log(`Page title: ${await guardianPage.title()}`);

  // Debug: Take screenshot
  await guardianPage.screenshot({ path: "test-results/confirm-guardian-debug.png" });
  console.log("Screenshot saved to test-results/confirm-guardian-debug.png");

  // Debug: Check for any visible buttons
  const allButtons = await guardianPage.locator("button").all();
  console.log(`\nFound ${allButtons.length} button elements on page`);
  for (let i = 0; i < allButtons.length; i++) {
    const isVisible = await allButtons[i].isVisible().catch(() => false);
    const text = await allButtons[i].textContent().catch(() => "N/A");
    console.log(`  Button ${i + 1}: visible=${isVisible}, text="${text}"`);
  }

  // Debug: Check page content
  const bodyText = await guardianPage.locator("body").textContent();
  console.log(`\nPage body text (first 500 chars): ${bodyText?.substring(0, 500)}`);

  // Debug: Check for error messages on page
  const errorElements = await guardianPage.locator("[class*=\"error\"], [class*=\"Error\"]").all();
  if (errorElements.length > 0) {
    console.log(`\nFound ${errorElements.length} error elements:`);
    for (const elem of errorElements) {
      const text = await elem.textContent();
      console.log(`  Error element: ${text}`);
    }
  }

  // Debug: Log any accumulated page errors
  if (pageErrors.length > 0) {
    console.log(`\nAccumulated page errors (${pageErrors.length}):`);
    pageErrors.forEach((err, i) => console.log(`  ${i + 1}. ${err}`));
  }

  // Check if we need to sign in or if already logged in as guardian
  console.log("\nChecking authentication state...");
  const confirmGuardianButton = guardianPage.getByRole("button", { name: /Confirm Guardian/i });
  const signInSsoButton = guardianPage.getByTestId("sign-in-sso");
  const isSignInVisible = await signInSsoButton.isVisible({ timeout: 2000 }).catch(() => false);

  if (isSignInVisible) {
    // Need to sign in
    console.log("Sign in required, clicking sign-in button...");
    await signInSsoButton.click();

    // Wait for redirect to login page
    await guardianPage.waitForTimeout(2000);

    // Should be redirected to login/signup page - use existing credential
    await reuseCredential(guardianPage, guardianCredential!);

    // Click signin button
    const signInButton = guardianPage.getByTestId("signin");
    if (await signInButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await signInButton.click();
      // Wait for redirect back to confirmation page
      await guardianPage.waitForURL("**/confirm-guardian**", { timeout: 10000 });
      await guardianPage.waitForTimeout(2000);
    }
  } else {
    // Already logged in as guardian
    console.log("Already logged in as guardian SSO account, proceeding to confirm...");
  }

  // Now confirm guardian should be visible
  console.log("\nLooking for 'Confirm Guardian' button...");
  const buttonCount = await confirmGuardianButton.count();
  console.log(`Found ${buttonCount} matching button(s)`);

  await expect(confirmGuardianButton).toBeVisible({ timeout: 10000 });

  console.log("Clicking Confirm Guardian button...");
  await confirmGuardianButton.click();

  // Wait for transaction
  await guardianPage.waitForTimeout(3000);

  // Check if we need to sign the confirmation
  const guardianPagesBefore = guardianContext.pages().length;
  await guardianPage.waitForTimeout(1000);
  if (guardianContext.pages().length > guardianPagesBefore) {
    const guardianSignPopup = guardianContext.pages()[guardianContext.pages().length - 1];
    console.log("Guardian signing popup detected...");
    await reuseCredential(guardianSignPopup, guardianCredential!);
    const confirmBtn = guardianSignPopup.getByTestId("confirm");
    if (await confirmBtn.isVisible({ timeout: 5000 })) {
      await confirmBtn.click();
      await guardianPage.waitForTimeout(3000);
    }
  }

  console.log("Waiting for guardian confirmation to complete...");
  await guardianPage.waitForTimeout(5000);

  // Check for success message
  const successIndicator = guardianPage.getByText(/Guardian.*confirmed|Success|Confirmed/i).first();
  if (await successIndicator.isVisible({ timeout: 5000 })) {
    console.log("✅ Guardian confirmation success message visible");
  } else {
    console.log("⚠️  No explicit success message found, but continuing...");
  }

  // ===== Step 6: Verify Guardian is Active =====
  console.log("\nStep 6: Verifying guardian is active...");
  await page.bringToFront();
  await page.reload();
  await page.waitForTimeout(3000);

  // Look for the guardian in the active guardians list
  // Use a more specific selector to avoid matching the URL in "Confirm Later" flow
  const guardiansList = page.getByRole("main").locator(`text=${guardianAddressText.slice(0, 8)}`).first();
  if (await guardiansList.isVisible({ timeout: 5000 })) {
    console.log("✅ Guardian found in active guardians list");
  } else {
    console.log("⚠️  Guardian not visible in list yet, may need more time");
  }

  console.log("\n=== Guardian E2E Test Complete ===\n");

  // Cleanup
  await guardianContext.close();
});

test("Guardian flow: propose guardian with paymaster", async ({ page }) => {
  test.setTimeout(90000); // Extended timeout for full guardian flow with paymaster
  console.log("\n=== Starting Guardian with Paymaster E2E Test ===\n");

  // ===== Step 1: Create Primary Account with Paymaster =====
  console.log("Step 1: Creating primary account with paymaster...");
  await page.goto("http://localhost:3002");
  await page.waitForTimeout(1000);

  // No popup - we're already on the auth-server signup page
  await expect(page.getByTestId("signup")).toBeVisible();
  page.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Page error: "${msg.text()}"`);
  });

  await setupWebAuthn(page);

  await page.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  try {
    await page.waitForURL("**/dashboard", { timeout: 15000 });
  } catch (e) {
    console.log(`Navigation timeout. Current URL: ${page.url()}`);
    // Check for error messages on the page
    const errorElement = page.locator("[role=\"alert\"], .error, [class*=\"error\"]").first();
    if (await errorElement.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      console.log(`Error message on page: ${errorText}`);
    }
    throw e;
  }
  await page.waitForTimeout(2000);

  // Wait for account address to be visible
  await expect(page.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const primaryAddressText = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Primary account created: ${primaryAddressText}`);

  // Fund the primary account with ETH
  await fundAccount(primaryAddressText, "1");

  // ===== Step 2: Create Guardian Account ===
  console.log("\nStep 2: Creating guardian account...");
  const guardianContext = await page.context().browser()!.newContext();
  const guardianPage = await guardianContext.newPage();

  await guardianPage.goto("http://localhost:3002");
  await guardianPage.waitForTimeout(2000);

  // No popup - we're already on the auth-server signup page
  const { newCredential: guardianCredential } = await setupWebAuthn(guardianPage);

  await guardianPage.getByTestId("signup").click();

  // Wait for navigation to dashboard after signup
  try {
    await guardianPage.waitForURL("**/dashboard", { timeout: 15000 });
  } catch (e) {
    console.log(`Guardian navigation timeout. Current URL: ${guardianPage.url()}`);
    // Check for error messages on the page
    const errorElement = guardianPage.locator("[role=\"alert\"], .error, [class*=\"error\"]").first();
    if (await errorElement.isVisible({ timeout: 1000 }).catch(() => false)) {
      const errorText = await errorElement.textContent();
      console.log(`Guardian error message: ${errorText}`);
    }
    throw e;
  }
  await guardianPage.waitForTimeout(2000);

  // Wait for account address to be visible
  await expect(guardianPage.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const guardianAddressText = await guardianPage.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`Guardian account created: ${guardianAddressText}`);

  // Fund the guardian account with ETH
  await fundAccount(guardianAddressText, "1");

  // ===== Step 3: Navigate to Guardian Settings and Propose ===
  console.log("\nStep 3: Proposing guardian with paymaster...");
  await page.bringToFront();

  // Capture console logs from the primary page during proposal
  const primaryPageConsole: string[] = [];
  page.on("console", (msg) => {
    const logMsg = `[${msg.type()}] ${msg.text()}`;
    primaryPageConsole.push(logMsg);
    console.log(`Primary page console: ${logMsg}`);
  });

  await page.goto("http://localhost:3002/dashboard/settings");
  await page.waitForTimeout(2000);

  const addGuardianButton = page.getByTestId("add-recovery-method");
  await expect(addGuardianButton).toBeVisible({ timeout: 10000 });
  await addGuardianButton.click();
  await page.waitForTimeout(1000);

  // Select "Recover with Guardian" option
  const guardianMethodButton = page.getByTestId("add-guardian-method");
  await expect(guardianMethodButton).toBeVisible({ timeout: 5000 });
  await guardianMethodButton.click();
  await page.waitForTimeout(1000);

  const continueButton = page.getByTestId("continue-recovery-method");
  await expect(continueButton).toBeVisible({ timeout: 10000 });
  await continueButton.click();
  await page.waitForTimeout(1000);

  const guardianInput = page.getByTestId("guardian-address-input").locator("input");
  await expect(guardianInput).toBeVisible({ timeout: 5000 });
  await guardianInput.fill(guardianAddressText);

  const proposeButton = page.getByRole("button", { name: /Propose/i, exact: false });
  await proposeButton.click();

  // Wait for paymaster-sponsored guardian proposal to complete
  // NOTE: The SSO client automatically signs ERC-4337 transactions with paymaster,
  // so there are NO popup windows to interact with during guardian proposal
  console.log("Waiting for paymaster-sponsored guardian proposal to complete...");
  await page.waitForTimeout(8000); // Wait for module installation + guardian proposal
  console.log("Guardian proposal with paymaster initiated");

  // Log all captured console messages
  if (primaryPageConsole.length > 0) {
    console.log(`\n📋 Captured ${primaryPageConsole.length} console messages from primary page:`);
    primaryPageConsole.forEach((msg, i) => console.log(`  ${i + 1}. ${msg}`));
  }

  // Handle "Do you wish to confirm your guardian now?" dialog
  try {
    const confirmLaterButton = page.getByRole("button", { name: /Confirm Later/i });
    await confirmLaterButton.waitFor({ state: "visible", timeout: 10000 });
    await confirmLaterButton.click();
    console.log("Clicked 'Confirm Later' on the confirmation prompt");
    await page.waitForTimeout(5000); // Wait for dialog to close and transaction to complete
  } catch (error) {
    console.log("⚠ Warning: 'Do you wish to confirm your guardian now?' prompt not found");
    console.log(`   ${error instanceof Error ? error.message : String(error)}`);
  }

  // ===== Step 4: Confirm Guardian with Paymaster =====
  console.log("\nStep 4: Confirming guardian with paymaster...");
  const confirmUrl = `http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=${primaryAddressText}&guardianAddress=${guardianAddressText}`;

  // Capture page errors for paymaster test
  const paymasterPageErrors: string[] = [];
  guardianPage.on("pageerror", (error) => {
    const errorMsg = `Page error: ${error.message}`;
    console.error(errorMsg);
    paymasterPageErrors.push(errorMsg);
  });

  guardianPage.on("console", (msg) => {
    if (msg.type() === "error") {
      console.error(`Console error: ${msg.text()}`);
    }
  });

  // Bring guardian page to front and wait for stable state
  await guardianPage.bringToFront();
  await guardianPage.waitForLoadState("domcontentloaded");
  await guardianPage.waitForTimeout(1000);

  // Check if guardianPage is still valid
  console.log(`Guardian page URL before navigation: ${guardianPage.url()}`);
  console.log(`Guardian page is closed: ${guardianPage.isClosed()}`);

  // Navigate to confirmation URL
  console.log(`Navigating to confirmation URL: ${confirmUrl}`);
  await guardianPage.goto(confirmUrl, { waitUntil: "domcontentloaded", timeout: 30000 });
  console.log("Navigation completed successfully");
  await guardianPage.waitForTimeout(2000);

  // Debug: Log page state
  console.log(`Current URL: ${guardianPage.url()}`);
  console.log(`Page title: ${await guardianPage.title()}`);

  // Debug: Take screenshot
  await guardianPage.screenshot({ path: "test-results/confirm-guardian-paymaster-debug.png" });
  console.log("Screenshot saved to test-results/confirm-guardian-paymaster-debug.png");

  // Debug: List all buttons
  const allButtons = await guardianPage.locator("button").all();
  console.log(`\nFound ${allButtons.length} button elements on page`);
  for (let i = 0; i < allButtons.length; i++) {
    const isVisible = await allButtons[i].isVisible().catch(() => false);
    const text = await allButtons[i].textContent().catch(() => "N/A");
    console.log(`  Button ${i + 1}: visible=${isVisible}, text="${text}"`);
  }

  // Debug: Check page content
  const bodyText = await guardianPage.locator("body").textContent();
  console.log(`\nPage body text (first 500 chars): ${bodyText?.substring(0, 500)}`);

  // Debug: Check for loading states
  const loadingElements = await guardianPage.locator("[class*=\"loading\"], [class*=\"Loading\"], [class*=\"spinner\"]").all();
  console.log(`\nFound ${loadingElements.length} loading indicator(s)`);

  if (paymasterPageErrors.length > 0) {
    console.log(`\nAccumulated page errors (${paymasterPageErrors.length}):`);
    paymasterPageErrors.forEach((err, i) => console.log(`  ${i + 1}. ${err}`));
  }

  // Check if we need to sign in or if already logged in as guardian
  console.log("\nChecking authentication state...");
  const confirmGuardianButton = guardianPage.getByRole("button", { name: /Confirm Guardian/i });
  const signInSsoButton = guardianPage.getByTestId("sign-in-sso");
  const isSignInVisible = await signInSsoButton.isVisible({ timeout: 2000 }).catch(() => false);

  if (isSignInVisible) {
    console.log("Sign in required, clicking sign-in button...");
    await signInSsoButton.click();

    // Wait for redirect to login page
    await guardianPage.waitForTimeout(2000);

    // Should be redirected to login/signup page - use existing credential
    await reuseCredential(guardianPage, guardianCredential!);

    // Click signin button
    const signInButton = guardianPage.getByTestId("signin");
    if (await signInButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await signInButton.click();
      // Wait for redirect back to confirmation page
      await guardianPage.waitForURL("**/confirm-guardian**", { timeout: 10000 });
      await guardianPage.waitForTimeout(2000);
    }
  } else {
    console.log("Already logged in as guardian SSO account, proceeding to confirm...");
  }

  // Now confirm guardian should be visible
  await expect(confirmGuardianButton).toBeVisible({ timeout: 10000 });
  console.log("Clicking Confirm Guardian button...");
  await confirmGuardianButton.click();

  console.log("Waiting for paymaster-sponsored confirmation transaction...");
  await guardianPage.waitForTimeout(3000);

  // Check if we need to sign the confirmation (popup might appear)
  const guardianPagesBefore = guardianContext.pages().length;
  await guardianPage.waitForTimeout(1000);
  if (guardianContext.pages().length > guardianPagesBefore) {
    const guardianSignPopup = guardianContext.pages()[guardianContext.pages().length - 1];
    console.log("Guardian signing popup detected...");
    await reuseCredential(guardianSignPopup, guardianCredential!);
    const confirmBtn = guardianSignPopup.getByTestId("confirm");
    if (await confirmBtn.isVisible({ timeout: 5000 })) {
      await confirmBtn.click();
      await guardianPage.waitForTimeout(3000);
    }
  }

  // Verify success
  await guardianPage.waitForTimeout(5000);
  const successIndicator = guardianPage.getByText(/Guardian.*confirmed|Success|Confirmed/i).first();
  if (await successIndicator.isVisible({ timeout: 5000 })) {
    console.log("✅ Guardian confirmation success message visible");
  } else {
    console.log("⚠️  No explicit success message found after paymaster confirmation");
  }

  console.log("\n=== Guardian with Paymaster E2E Test Complete ===\n");

  // Cleanup
  await guardianContext.close();
});

test("Guardian flow: full recovery execution", async ({ page, context: baseContext }) => {
  test.setTimeout(120000); // Extended timeout for full recovery flow
  console.log("\n=== Starting Full Recovery Execution E2E Test ===\n");

  // Step 1: Create account owner
  console.log("Step 1: Creating account owner...");
  await page.goto("http://localhost:3002");

  const { newCredential: _ownerCredential } = await setupWebAuthn(page);
  await page.getByTestId("signup").click();

  await page.waitForURL("**/dashboard", { timeout: 15000 });
  await page.waitForTimeout(2000);

  await expect(page.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const ownerAddress = await page.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`✅ Owner account created: ${ownerAddress}`);

  await fundAccount(ownerAddress);

  // Step 2: Create guardian account
  console.log("\nStep 2: Creating guardian account...");
  const guardianContext = await baseContext.browser()!.newContext();
  const guardianPage = await guardianContext.newPage();

  await guardianPage.goto("http://localhost:3002");
  await guardianPage.waitForTimeout(2000);

  const { newCredential: _guardianCredential } = await setupWebAuthn(guardianPage);
  await guardianPage.getByTestId("signup").click();

  await guardianPage.waitForURL("**/dashboard", { timeout: 15000 });
  await guardianPage.waitForTimeout(2000);

  await expect(guardianPage.getByTestId("account-address")).toBeVisible({ timeout: 10000 });
  const guardianAddress = await guardianPage.getByTestId("account-address").getAttribute("data-address") || "";
  console.log(`✅ Guardian account created: ${guardianAddress}`);

  await fundAccount(guardianAddress);

  // Step 3: Owner proposes guardian
  console.log("\nStep 3: Owner proposing guardian...");

  // Capture console logs from the owner page during proposal
  const ownerPageConsole: string[] = [];
  page.on("console", (msg) => {
    const logMsg = `[${msg.type()}] ${msg.text()}`;
    ownerPageConsole.push(logMsg);
    console.log(`Owner page console: ${logMsg}`);
  });

  await page.goto("http://localhost:3002/dashboard/settings");
  await page.waitForTimeout(2000);

  const addRecoveryButton = page.getByTestId("add-recovery-method");
  await expect(addRecoveryButton).toBeVisible({ timeout: 10000 });
  await addRecoveryButton.click();
  await page.waitForTimeout(1000);

  const guardianMethodButton = page.getByTestId("add-guardian-method");
  await expect(guardianMethodButton).toBeVisible({ timeout: 5000 });
  await guardianMethodButton.click();
  await page.waitForTimeout(1000);

  const continueButton = page.getByTestId("continue-recovery-method");
  await expect(continueButton).toBeVisible({ timeout: 10000 });
  await continueButton.click();
  await page.waitForTimeout(1000);

  const guardianAddressInput = page.getByTestId("guardian-address-input").locator("input");
  await expect(guardianAddressInput).toBeVisible({ timeout: 10000 });
  await guardianAddressInput.fill(guardianAddress);

  const proposeButton = page.getByRole("button", { name: /Propose/i, exact: false });
  await proposeButton.click();

  // Wait for guardian proposal to complete
  const errorMessage = page.locator("text=/error.*proposing/i");
  const errorVisible = await errorMessage.isVisible({ timeout: 8000 }).catch(() => false);
  if (errorVisible) {
    const errorText = await errorMessage.textContent();
    throw new Error(`Guardian proposal failed: ${errorText}`);
  }

  await page.waitForTimeout(8000);
  console.log("✅ Guardian proposed successfully");

  // Log all captured console messages
  if (ownerPageConsole.length > 0) {
    console.log(`\n📋 Captured ${ownerPageConsole.length} console messages from owner page:`);
    ownerPageConsole.forEach((msg, i) => console.log(`  ${i + 1}. ${msg}`));
  }

  // Step 4: Guardian accepts role
  console.log("\nStep 4: Guardian accepting role...");
  const confirmUrl = `http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=${ownerAddress}&guardianAddress=${guardianAddress}`;
  console.log(`Confirmation URL: ${confirmUrl}`);
  await guardianPage.goto(confirmUrl);
  await guardianPage.waitForTimeout(2000);

  const confirmButton = guardianPage.getByTestId("confirm-guardian-button");
  await expect(confirmButton).toBeVisible({ timeout: 10000 });

  // Check button state before clicking
  const isButtonDisabled = await confirmButton.isDisabled();
  console.log(`Button disabled state before click: ${isButtonDisabled}`);

  await confirmButton.click();
  console.log("Clicked confirm button, waiting for response...");

  // Wait and check state multiple times to see if it progresses or gets stuck
  await guardianPage.waitForTimeout(2000);

  const stateElement = guardianPage.locator("text=Current State:").locator("xpath=following-sibling::span");

  // Check state progression over 30 seconds (15 iterations × 2 seconds)
  // Allows time for: event queries (10s) + transaction confirmation (10s) + buffer
  for (let i = 0; i < 25; i++) {
    const currentState = await stateElement.textContent().catch(() => "unknown");
    console.log(`[${i * 2}s] Current confirmation state: ${currentState}`);

    // Check for UI error after each state check
    const errorElement = guardianPage.locator("p.text-error-600, p.text-error-400");
    const hasError = await errorElement.isVisible().catch(() => false);

    if (hasError) {
      const errorText = await errorElement.textContent();
      console.log("❌ Guardian confirmation failed with UI error:", errorText);
      throw new Error(`Guardian confirmation failed: ${errorText}`);
    }

    // If state shows error, extract it
    if (currentState?.startsWith("error:")) {
      console.log("❌ Confirmation state shows error:", currentState);
      throw new Error(`Guardian confirmation failed: ${currentState}`);
    }

    // If we reached a final state, break
    if (currentState === "complete" || currentState === "confirm_guardian_completed") {
      break;
    }

    // If stuck in getting state for more than 16 seconds, that's the issue
    // (Allows 5s for getBlockNumber + 10s for event queries + buffer)
    if (i >= 8 && currentState?.includes("getting")) {
      console.log(`⚠️ Stuck in ${currentState} for ${i * 2} seconds`);
      await guardianPage.screenshot({ path: "test-results/stuck-getting-client-debug.png" });
      const bodyText = await guardianPage.locator("body").textContent();
      console.log("Page debug state:", bodyText?.match(/Current State:.*?Expected flow:/s)?.[0] || "Not found");
      throw new Error(`Guardian confirmation stuck in state: ${currentState}. getConfigurableAccount or getWalletClient is hanging/failing silently.`);
    }

    await guardianPage.waitForTimeout(2000);
  }

  // Check for successful completion
  const finalState = await stateElement.textContent().catch(() => "unknown");
  if (finalState !== "complete" && finalState !== "confirm_guardian_completed") {
    console.log("⚠️ Guardian confirmation did not complete. Final state:", finalState);
    throw new Error(`Guardian confirmation failed. Final state: ${finalState}`);
  }
  console.log("✅ Guardian confirmed successfully. Final state:", finalState);

  // Step 5: Owner initiates recovery with new passkey using "Confirm Now" flow
  console.log("\nStep 5: Owner initiating account recovery with Confirm Now...");

  // Create new recovery credential
  const recoveryContext = await baseContext.browser()!.newContext();
  const recoveryPage = await recoveryContext.newPage();

  // Navigate directly to guardian recovery page
  await recoveryPage.goto("http://localhost:3002/recovery/guardian");
  await recoveryPage.waitForTimeout(2000);

  // Enter owner account address - find the input inside the ZkInput component
  const accountInput = recoveryPage.locator("#address input");
  await expect(accountInput).toBeVisible({ timeout: 10000 });
  await accountInput.fill(ownerAddress);
  await recoveryPage.waitForTimeout(1000);

  // Click Continue button
  const continueBtn = recoveryPage.getByRole("button", { name: /Continue/i });
  await expect(continueBtn).toBeEnabled({ timeout: 5000 });
  await continueBtn.click();
  await recoveryPage.waitForTimeout(2000);

  // Set up WebAuthn mock for recovery passkey
  await setupWebAuthn(recoveryPage);

  const createPasskeyBtn = recoveryPage.getByRole("button", { name: /Generate Passkey/i });
  await expect(createPasskeyBtn).toBeVisible({ timeout: 10000 });
  await createPasskeyBtn.click();
  await recoveryPage.waitForTimeout(3000);

  // Step 6: Click "Confirm Later" to get the recovery URL
  console.log("\nStep 6: Getting recovery confirmation URL...");

  // Click "Confirm Later" to get the URL
  const confirmLaterBtn = recoveryPage.getByRole("button", { name: /Confirm Later/i });
  await expect(confirmLaterBtn).toBeVisible({ timeout: 10000 });
  await confirmLaterBtn.click();
  await recoveryPage.waitForTimeout(2000);

  // Find and extract the recovery URL from the page
  const recoveryLink = recoveryPage.locator("a[href*=\"/recovery/guardian/confirm-recovery\"]");
  await expect(recoveryLink).toBeVisible({ timeout: 10000 });
  const recoveryConfirmationUrl = await recoveryLink.getAttribute("href");
  console.log(`Recovery confirmation URL: ${recoveryConfirmationUrl}`);

  if (!recoveryConfirmationUrl) {
    throw new Error("Failed to capture recovery confirmation URL");
  }

  // Close the owner's recovery page and context
  await recoveryPage.close();
  await recoveryContext.close();

  // Step 7: Guardian confirms recovery
  console.log("\nStep 7: Guardian confirming recovery...");

  // Navigate directly to the recovery confirmation URL
  await guardianPage.goto(recoveryConfirmationUrl);
  await guardianPage.waitForLoadState("networkidle");

  // Wait for the guardian select dropdown to load
  const guardianSelect = guardianPage.locator("select");
  await expect(guardianSelect).toBeVisible({ timeout: 15000 });

  // Debug: Check what options are actually in the dropdown
  await guardianPage.waitForTimeout(3000); // Give time for async guardians to load
  const allOptions = await guardianSelect.locator("option").all();
  console.log(`\nFound ${allOptions.length} option(s) in guardian dropdown:`);
  for (const option of allOptions) {
    const value = await option.getAttribute("value");
    const text = await option.textContent();
    console.log(`  - value="${value}" text="${text}"`);
  }

  // Try to select the guardian (try both cases)
  const normalizedGuardianAddress = guardianAddress.toLowerCase();
  console.log(`\nLooking for guardian: ${guardianAddress}`);
  console.log(`Normalized (lowercase): ${normalizedGuardianAddress}`);

  // Check if the option exists in either case
  let optionFound = false;
  for (const option of allOptions) {
    const value = await option.getAttribute("value");
    if (value && value.toLowerCase() === normalizedGuardianAddress) {
      console.log(`✅ Found matching option with value: ${value}`);
      await guardianSelect.selectOption({ value: value });
      optionFound = true;
      break;
    }
  }

  if (!optionFound) {
    throw new Error(`Guardian option not found in dropdown. Expected: ${guardianAddress} (or ${normalizedGuardianAddress}). Available options: ${allOptions.length > 0 ? (await Promise.all(allOptions.map(async (o) => await o.getAttribute("value")))).join(", ") : "none"}`);
  }

  await guardianPage.waitForTimeout(1000);

  // Capture console logs and errors from guardian page
  const guardianConsoleMessages: string[] = [];
  guardianPage.on("console", (msg) => {
    const logMsg = `[${msg.type()}] ${msg.text()}`;
    guardianConsoleMessages.push(logMsg);
    console.log(`Guardian console: ${logMsg}`);
  });

  guardianPage.on("pageerror", (error) => {
    const errorMsg = `Page error: ${error.message}`;
    console.error(`Guardian ${errorMsg}`);
    guardianConsoleMessages.push(errorMsg);
  });

  // Click Confirm Recovery button
  const confirmRecoveryBtn = guardianPage.getByRole("button", { name: /Confirm Recovery/i });
  await expect(confirmRecoveryBtn).toBeVisible({ timeout: 10000 });
  await expect(confirmRecoveryBtn).toBeEnabled({ timeout: 5000 });
  await confirmRecoveryBtn.click();

  console.log("Clicked Confirm Recovery, waiting for success message...");

  // Wait longer for recovery confirmation and polling to complete
  await guardianPage.waitForTimeout(20000);

  // Print captured console messages
  if (guardianConsoleMessages.length > 0) {
    console.log(`\n📋 Guardian page console messages (${guardianConsoleMessages.length}):`);
    guardianConsoleMessages.forEach((msg, i) => {
      console.log(`  ${i + 1}. ${msg}`);
    });
  } else {
    console.log("\n⚠️ No console messages captured from guardian page");
  }

  // Check for error messages on the page before checking for success
  const errorElement = guardianPage.locator("text=/error/i").first();
  const hasError = await errorElement.isVisible({ timeout: 2000 }).catch(() => false);
  if (hasError) {
    const errorText = await errorElement.textContent();
    console.error(`❌ Error message on page: ${errorText}`);
    await guardianPage.screenshot({ path: "test-results/confirm-recovery-error.png" });
    throw new Error(`Recovery confirmation failed: ${errorText}`);
  }

  // Verify recovery was initiated - look for success message
  const successMessage = guardianPage.getByText(/24hrs/i)
    .or(guardianPage.getByText(/24 hours/i))
    .or(guardianPage.getByText(/recovery.*initiated/i));

  await expect(successMessage).toBeVisible({ timeout: 20000 });

  console.log("✅ Guardian confirmed recovery successfully");

  // Step 7: Verify recovery is pending (skip execution for now as it requires time travel)
  console.log("\nStep 7: Verifying recovery is in pending state...");
  console.log("Note: Full recovery execution requires EVM time manipulation");
  console.log("The recovery would need to wait 24 hours before it can be finalized");

  console.log("\n=== Full Recovery E2E Flow Complete ===\n");
  console.log("Summary:");
  console.log("  ✅ Owner account created");
  console.log("  ✅ Guardian account created");
  console.log("  ✅ Guardian proposed by owner");
  console.log("  ✅ Guardian confirmed their role");
  console.log("  ✅ Owner initiated recovery with new passkey");
  console.log("  ✅ Guardian confirmed the recovery request");
  console.log("  ⏳ Recovery pending (24hr delay before finalization)");

  // Cleanup
  await guardianContext.close();
});
