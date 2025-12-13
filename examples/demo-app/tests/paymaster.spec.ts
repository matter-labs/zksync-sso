import { expect, type Page, test } from "@playwright/test";
import contractsAnvil from "../contracts-anvil.json";

async function waitForServicesToLoad(page: Page): Promise<void> {
  const maxRetryAttempts = 10;
  let retryCount = 0;

  // Wait for demo-app to finish loading
  await page.goto("/");
  let demoHeader = page.getByText("ZKsync SSO Demo");
  while (!(await demoHeader.isVisible()) && retryCount < maxRetryAttempts) {
    await page.waitForTimeout(1000);
    demoHeader = page.getByText("ZKsync SSO Demo");
    retryCount++;

    console.log(`Waiting for demo app to load (retry ${retryCount})...`);
  }
  console.log("Demo App loaded");

  // Wait for auth server to finish loading
  retryCount = 0;
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

async function setupWebAuthn(page: Page) {
  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");
  await client.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "usb",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });
  return client;
}

test.beforeAll(async () => {
  // Check if paymaster is deployed
  if (!(contractsAnvil as unknown).testPaymaster) {
    throw new Error(
      "TestPaymaster not deployed. Run: pnpm --filter demo-app deploy:paymaster",
    );
  }
  console.log(`Using TestPaymaster at: ${(contractsAnvil as unknown).testPaymaster}`);
});

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Main page error console: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Main page uncaught exception: "${exception}"`);
  });

  await waitForServicesToLoad(page);
  await page.goto("/");
  await expect(page.getByText("ZKsync SSO Demo")).toBeVisible();
});

test("Unfunded account fails without paymaster", async ({ page }) => {
  // Add skipFunding query param to prevent auto-funding
  await page.goto("/?skipFunding=true");

  // Connect without paymaster - this should create an unfunded account
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  // Ensure popup is displayed
  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();

  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn
  await setupWebAuthn(popup);

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Confirm access to your account
  await expect(popup.getByText("Connect to ZKsync SSO Demo")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();

  // Check balance is ~0 (unfunded account)
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  expect(startBalance).toBeLessThan(0.01); // Should be essentially 0

  // Try to send ETH - this should fail with insufficient funds
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();

  // Wait for popup with transaction confirmation
  await page.waitForTimeout(2000);
  const confirmPopup = page.context().pages()[1];

  if (confirmPopup) {
    // Session flow doesn't need confirmation, regular flow does
    // Setup WebAuthn again for the new popup
    await setupWebAuthn(confirmPopup);

    // Confirm transaction
    await expect(confirmPopup.getByText("-0.1")).toBeVisible();
    await confirmPopup.getByTestId("confirm").click();

    // Wait for transaction processing
    await page.waitForTimeout(3000);
  } else {
    // Session flow - wait for transaction to process
    await page.waitForTimeout(3000);
  }

  // Should show error due to insufficient funds
  await expect(page.locator(".text-red-800")).toBeVisible();
  const errorText = await page.locator(".text-red-800").innerText();
  expect(errorText.toLowerCase()).toMatch(/insufficient|balance|funds/i);

  console.log(`Transaction failed as expected: ${errorText}`);
});

test("Unfunded account with paymaster succeeds", async ({ page }) => {
  // Add skipFunding query param to prevent auto-funding
  await page.goto("/?skipFunding=true");

  // Connect with paymaster
  await page.getByRole("button", { name: "Connect (Paymaster)", exact: true }).click();

  // Ensure popup is displayed
  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();

  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn
  let client = await setupWebAuthn(popup);
  let newCredential = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added for paymaster test");
    newCredential = credentialAdded.credential;
  });

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Confirm access to your account
  await expect(popup.getByText("Connect to ZKsync SSO Demo")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();

  // Check balance is ~0 (unfunded account)
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  expect(startBalance).toBeLessThan(0.01); // Should be essentially 0
  console.log(`Start balance (unfunded): ${startBalance} ETH`);

  // Try to send ETH - this should SUCCEED with paymaster sponsorship
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();

  // Wait for Auth Server to pop back up
  await page.waitForTimeout(2000);
  const confirmPopup = page.context().pages()[1];

  // We need to recreate the virtual authenticator to match the previous one
  client = await confirmPopup.context().newCDPSession(confirmPopup);
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
  await expect(newCredential).not.toBeNull();
  await client.send("WebAuthn.addCredential", {
    authenticatorId: result.authenticatorId,
    credential: newCredential!,
  });

  // Confirm the transfer
  await expect(confirmPopup.getByText("-0.1")).toBeVisible();
  await confirmPopup.getByTestId("confirm").click();

  // Wait for confirmation to complete and popup to close
  await page.waitForTimeout(3000);

  // Confirm transfer completed - button should be enabled again
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();

  // Balance should STILL be ~0 because paymaster covered gas fees!
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  console.log(`End balance (with paymaster): ${endBalance} ETH`);

  // Balance should not have changed much (paymaster paid the fees)
  expect(endBalance).toBeLessThan(0.01);

  // Verify no error message
  const errorVisible = await page.locator(".text-red-800").isVisible().catch(() => false);
  expect(errorVisible).toBe(false);
});

test("Session creation with paymaster on unfunded account", async ({ page }) => {
  // Add skipFunding query param to prevent auto-funding
  await page.goto("/?skipFunding=true");

  // Connect with session + paymaster
  await page.getByRole("button", { name: "Connect Session (Paymaster)", exact: true }).click();

  // Ensure popup is displayed
  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();

  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn
  await setupWebAuthn(popup);

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Session authorization screen
  await expect(popup.getByText("Authorize ZKsync SSO Demo")).toBeVisible();
  await expect(popup.getByText("Act on your behalf")).toBeVisible();
  await expect(popup.getByText("Expires tomorrow")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();

  // Check balance is ~0 (unfunded account)
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  expect(startBalance).toBeLessThan(0.01); // Should be essentially 0
  console.log(`Start balance (unfunded with session): ${startBalance} ETH`);

  // Send ETH using session - should succeed with paymaster, no popup needed
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();

  // Wait for transaction to process (no confirmation popup for session)
  await page.waitForTimeout(3000);

  // Confirm transfer completed
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();

  // Balance should STILL be ~0 because paymaster covered gas fees!
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  console.log(`End balance (session + paymaster): ${endBalance} ETH`);

  // Balance should not have changed (paymaster paid the fees)
  expect(endBalance).toBeLessThan(0.01);

  // Verify no error message
  const errorVisible = await page.locator(".text-red-800").isVisible().catch(() => false);
  expect(errorVisible).toBe(false);
});
