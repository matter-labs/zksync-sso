import { test, expect, type Page } from "@playwright/test";

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
};

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

test("Create account with session and send ETH", async ({ page }) => {
  // Click the Connect button
  await page.getByRole("button", { name: "Connect with Session", exact: true }).click();

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

  // Setup webauthn a Chrome Devtools Protocol session
  // NOTE: This needs to be done for every page of every test that uses WebAuthn
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

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Add session
  await expect(popup.getByText("Authorize ZKsync SSO Demo")).toBeVisible();
  await expect(popup.getByText("Act on your behalf")).toBeVisible();
  await expect(popup.getByText("Expires tomorrow")).toBeVisible();
  await expect(popup.getByText("Permissions")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  // Send some eth
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be ~0.1 ETH less")
    .toBeGreaterThan(endBalance + 0.1);
});

test("Create account with session and send ETH with paymaster", async ({ page }) => {
  // Click the Connect button
  await page.getByRole("button", { name: "Connect with Session", exact: true }).click();

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

  // Setup webauthn a Chrome Devtools Protocol session
  // NOTE: This needs to be done for every page of every test that uses WebAuthn
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

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Add session
  await expect(popup.getByText("Authorize ZKsync SSO Demo")).toBeVisible();
  await expect(popup.getByText("Act on your behalf")).toBeVisible();
  await expect(popup.getByText("Expires tomorrow")).toBeVisible();
  await expect(popup.getByText("Permissions")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  // Send some eth with paymaster
  await page.getByRole("button", { name: "Send 0.1 ETH with Paymaster", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send 0.1 ETH with Paymaster", exact: true })).toBeEnabled();
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be 0.1 ETH less (no fees)")
    .toEqual(endBalance + 0.1);
});

test("Create passkey account and send ETH", async ({ page }) => {
  // Click the Connect button
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  // Ensure popup is displayed
  await page.waitForTimeout(2000);
  let popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();
  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup webauthn a Chrome Devtools Protocol session
  // NOTE: This needs to be done for every page of every test that uses WebAuthn
  let client = await popup.context().newCDPSession(popup);
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
  let newCredential = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    console.log(`Authenticator ID: ${credentialAdded.authenticatorId}`);
    console.log(`Credential: ${credentialAdded.credential}`);
    newCredential = credentialAdded.credential;
  });

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Confirm access to your account
  await expect(popup.getByText("Connect to ZKsync SSO Demo")).toBeVisible();
  await expect(popup.getByText("localhost:3004")).toBeVisible();
  await expect(popup.getByText("Let it see your address, balance and activity")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  // Send some eth
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();

  // Wait for Auth Server to pop back up
  await page.waitForTimeout(2000);
  popup = page.context().pages()[1];

  // We need to recreate the virtual authenticator to match the previous one
  client = await popup.context().newCDPSession(popup);
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
  await expect(popup.getByText("-0.1")).toBeVisible();
  await expect(popup.getByText("Sending to")).toBeVisible();
  await expect(popup.getByText("0x55b...4A6")).toBeVisible();
  await expect(popup.getByText("Fees")).toBeVisible();
  await popup.getByTestId("confirm").click();

  // Wait for confirmation to complete and popup to close
  await page.waitForTimeout(2000);

  // Confirm transfer completed and balance updated
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be ~0.1 ETH less")
    .toBeGreaterThan(endBalance + 0.1);
});

test("Create passkey account and send ETH with paymaster", async ({ page }) => {
  // Click the Connect button
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  // Ensure popup is displayed
  await page.waitForTimeout(2000);
  let popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();
  popup.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup webauthn a Chrome Devtools Protocol session
  // NOTE: This needs to be done for every page of every test that uses WebAuthn
  let client = await popup.context().newCDPSession(popup);
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
  let newCredential = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    console.log(`Authenticator ID: ${credentialAdded.authenticatorId}`);
    console.log(`Credential: ${credentialAdded.credential}`);
    newCredential = credentialAdded.credential;
  });

  // Click Sign Up
  await popup.getByTestId("signup").click();

  // Confirm access to your account
  await expect(popup.getByText("Connect to ZKsync SSO Demo")).toBeVisible();
  await expect(popup.getByText("localhost:3004")).toBeVisible();
  await expect(popup.getByText("Let it see your address, balance and activity")).toBeVisible();
  await popup.getByTestId("connect").click();

  // Waits for session to complete and popup to close
  await page.waitForTimeout(2000);

  // Check address/balance is shown
  await expect(page.getByText("Disconnect")).toBeVisible();
  await expect(page.getByText("Balance:")).toBeVisible();
  const startBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");

  // Send some eth with paymaster
  await page.getByRole("button", { name: "Send 0.1 ETH with Paymaster", exact: true }).click();

  // Wait for Auth Server to pop back up
  await page.waitForTimeout(2000);
  popup = page.context().pages()[1];

  // We need to recreate the virtual authenticator to match the previous one
  client = await popup.context().newCDPSession(popup);
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
  await expect(popup.getByText("-0.1")).toBeVisible();
  await expect(popup.getByText("Sending to")).toBeVisible();
  await expect(popup.getByText("0x55b...4A6")).toBeVisible();
  await expect(popup.getByText("Fees")).toBeVisible();
  await popup.getByTestId("confirm").click();

  // Wait for confirmation to complete and popup to close
  await page.waitForTimeout(2000);

  // Confirm transfer completed and balance updated
  await expect(page.getByRole("button", { name: "Send 0.1 ETH with Paymaster", exact: true })).toBeEnabled();
  const endBalance = +(await page.getByText("Balance:").innerText())
    .replace("Balance: ", "")
    .replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be 0.1 ETH less (no fees)")
    .toEqual(endBalance + 0.1);
});
