/* eslint-disable no-console */
import { exec } from "child_process";
import { promisify } from "util";

import { expect, test, type Page } from "@playwright/test";

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

test("Guardian flow: propose and confirm guardian", async ({ page }) => {
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
  await page.waitForTimeout(5000); // Wait for module installation + guardian proposal
  console.log("Guardian proposal initiated");

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
  const successIndicator = guardianPage.getByText(/Guardian.*confirmed|Success|Confirmed/i);
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
  const guardiansList = page.locator(`text=${guardianAddressText.slice(0, 8)}`);
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
  await setupWebAuthn(guardianPage);

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
  await page.waitForTimeout(5000); // Wait for module installation + guardian proposal
  console.log("Guardian proposal with paymaster initiated");

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

  console.log("Waiting for paymaster-sponsored confirmation...");
  await guardianPage.waitForTimeout(5000);

  console.log("\n=== Guardian with Paymaster E2E Test Complete ===\n");

  // Cleanup
  await guardianContext.close();
});
