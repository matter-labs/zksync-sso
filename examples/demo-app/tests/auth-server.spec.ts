/* eslint-disable no-console */
import { expect, type Page, test } from "@playwright/test";
import contractsConfig from "../contracts-anvil.json" with { type: "json" };
import { validateContractConfig } from "../utils/validate-config";

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
}

test.beforeAll(async () => {
  // Validate contract configuration before running tests
  console.log("🔍 Validating contract configuration...");
  const result = await validateContractConfig("http://localhost:3005", {
    factory: contractsConfig.factory,
    webauthnValidator: contractsConfig.webauthnValidator,
    eoaValidator: contractsConfig.eoaValidator,
    sessionValidator: contractsConfig.sessionValidator,
    entryPoint: contractsConfig.entryPoint,
    chainId: contractsConfig.chainId,
  });

  if (!result.valid) {
    throw new Error(
      "Contract configuration mismatch! Auth-server-API is using different addresses than the local deployment. "
      + "Please restart the dev server: pnpm nx dev:erc4337 demo-app",
    );
  }
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
  await expect(page.getByText("ZKsync SSO Demo")).toBeVisible();
});

test("Connect with auth-server (no session)", async ({ page, context }) => {
  // Click the Connect button (without session)
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  // Wait for auth-server popup to open
  await page.waitForTimeout(2000);
  const popup = context.pages()[1];
  expect(popup).toBeDefined();

  await expect(popup.getByText("Connect to")).toBeVisible();

  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn virtual authenticator
  const client = await popup.context().newCDPSession(popup);
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

  // Click Sign Up to create passkey
  await popup.getByTestId("signup").click();

  // Wait for passkey creation and account deployment
  await page.waitForTimeout(5000);

  // Verify connection was successful - check for Disconnect button
  await expect(page.getByRole("button", { name: "Disconnect" })).toBeVisible({ timeout: 10000 });

  // Verify address and balance are displayed
  await expect(page.getByText("Connected Address:")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();

  console.log("✓ Successfully connected via auth-server");
});

test("Connect with auth-server and session", async ({ page, context }) => {
  // Click the Connect with Session button
  await page.getByRole("button", { name: "Connect with Session", exact: true }).click();

  // Wait for auth-server popup to open
  await page.waitForTimeout(2000);
  const popup = context.pages()[1];
  expect(popup).toBeDefined();

  await expect(popup.getByText("Connect to")).toBeVisible();

  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn virtual authenticator
  const client = await popup.context().newCDPSession(popup);
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

  // Click Sign Up to create passkey
  await popup.getByTestId("signup").click();

  // Should see session authorization screen
  await expect(popup.getByText("Authorize")).toBeVisible({ timeout: 10000 });
  await expect(popup.getByText("Act on your behalf")).toBeVisible();

  // Click Connect to authorize session
  await popup.getByTestId("connect").click();

  // Wait for session setup and popup to close
  await page.waitForTimeout(5000);

  // Verify connection was successful
  await expect(page.getByRole("button", { name: "Disconnect" })).toBeVisible({ timeout: 10000 });

  // Verify address and balance are displayed
  await expect(page.getByText("Connected Address:")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();

  console.log("✓ Successfully connected via auth-server with session");

  // Test sending ETH with session (should not require popup)
  const startBalanceText = await page.getByText("Balance:").innerText();
  const startBalance = parseFloat(startBalanceText.match(/[\d.]+/)?.[0] || "0");

  await page.getByRole("button", { name: "Send 0.1 ETH" }).click();

  // Wait for transaction to complete
  await page.waitForTimeout(8000);

  // Verify balance decreased
  const endBalanceText = await page.getByText("Balance:").innerText();
  const endBalance = parseFloat(endBalanceText.match(/[\d.]+/)?.[0] || "0");

  expect(endBalance).toBeLessThan(startBalance);
  console.log(`✓ Balance decreased from ${startBalance} to ${endBalance} ETH`);

  console.log("✓ Successfully sent ETH using session (no popup required)");
});

test("Reconnect to existing passkey account", async ({ page, context }) => {
  // First, create an account
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  await page.waitForTimeout(2000);
  const popup = context.pages()[1];

  const client = await popup.context().newCDPSession(popup);
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

  await popup.getByTestId("signup").click();
  await page.waitForTimeout(5000);

  // Get the connected address
  const addressText = await page.getByText(/Connected Address: 0x/).innerText();
  const firstAddress = addressText.replace("Connected Address: ", "");

  console.log(`✓ First connection address: ${firstAddress}`);

  // Disconnect
  await page.getByRole("button", { name: "Disconnect" }).click();
  await expect(page.getByRole("button", { name: "Connect", exact: true })).toBeVisible();

  // Reconnect using existing passkey
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  await page.waitForTimeout(2000);
  const popup2 = context.pages()[context.pages().length - 1];

  // Click Sign In (use existing passkey)
  await popup2.getByTestId("signin").click();

  await page.waitForTimeout(5000);

  // Verify we got the same address
  const reconnectedAddressText = await page.getByText(/Connected Address: 0x/).innerText();
  const secondAddress = reconnectedAddressText.replace("Connected Address: ", "");

  expect(secondAddress).toBe(firstAddress);
  console.log(`✓ Reconnected to same address: ${secondAddress}`);
});
