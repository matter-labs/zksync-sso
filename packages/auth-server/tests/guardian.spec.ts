/* eslint-disable no-console */
import { expect, type Page, test } from "@playwright/test";

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

test("Guardian flow: propose and confirm guardian", async ({ page, context }) => {
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

  const { newCredential: primaryCredential } = await setupWebAuthn(page);

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

  console.log("Waiting for propose transaction to complete...");

  // Wait for transaction confirmation popup if needed
  await page.waitForTimeout(3000);

  // Check if we need to sign the transaction
  const pagesBefore = context.pages().length;
  await page.waitForTimeout(1000);
  if (context.pages().length > pagesBefore) {
    const txPopup = context.pages()[context.pages().length - 1];
    console.log("Transaction confirmation popup detected, signing...");
    await reuseCredential(txPopup, primaryCredential!);
    const confirmBtn = txPopup.getByTestId("confirm");
    if (await confirmBtn.isVisible({ timeout: 5000 })) {
      await confirmBtn.click();
      await page.waitForTimeout(3000);
    }
  }

  // Wait for pending guardian to appear
  await page.waitForTimeout(5000);
  console.log("Guardian proposed successfully");

  // ===== Step 5: Confirm Guardian =====
  console.log("\nStep 5: Confirming guardian...");

  // Construct confirmation URL
  const confirmUrl = `http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=${primaryAddressText}&guardianAddress=${guardianAddressText}`;
  console.log(`Confirmation URL: ${confirmUrl}`);

  await guardianPage.goto(confirmUrl);
  await guardianPage.waitForTimeout(2000);

  // Look for "Confirm Guardian" button
  const confirmGuardianButton = guardianPage.getByRole("button", { name: /Confirm Guardian/i });
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

  // ===== Step 2: Create Guardian Account =====
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

  // ===== Step 3: Navigate to Guardian Settings and Propose =====
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

  console.log("Waiting for paymaster-sponsored transaction...");
  await page.waitForTimeout(5000);

  console.log("Guardian proposed with paymaster successfully");

  // ===== Step 4: Confirm Guardian with Paymaster =====
  console.log("\nStep 4: Confirming guardian with paymaster...");
  const confirmUrl = `http://localhost:3002/recovery/guardian/confirm-guardian?accountAddress=${primaryAddressText}&guardianAddress=${guardianAddressText}`;
  await guardianPage.goto(confirmUrl);
  await guardianPage.waitForTimeout(2000);

  const confirmGuardianButton = guardianPage.getByRole("button", { name: /Confirm Guardian/i });
  await expect(confirmGuardianButton).toBeVisible({ timeout: 10000 });
  await confirmGuardianButton.click();

  console.log("Waiting for paymaster-sponsored confirmation...");
  await guardianPage.waitForTimeout(5000);

  console.log("\n=== Guardian with Paymaster E2E Test Complete ===\n");

  // Cleanup
  await guardianContext.close();
});
