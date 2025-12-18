/* eslint-disable no-console */
import { expect, type Page, test } from "@playwright/test";

type WebAuthnCredential = {
  credentialId: string;
  isResidentCredential: boolean;
  privateKey: string;
  signCount: number;
};

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
  // Step 1: regular connect to create account with passkey
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText("Connect to")).toBeVisible();
  popup.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Prepare WebAuthn for passkey creation and capture the credential for later reuse
  const client = await popup.context().newCDPSession(popup);
  await client.send("WebAuthn.enable");
  let newCredential: WebAuthnCredential | null = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    newCredential = credentialAdded.credential;
  });
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
  await expect(popup.getByText("Connect to ZKsync SSO Demo")).toBeVisible();
  await popup.getByTestId("connect").click();

  await page.waitForTimeout(2000);
  await expect(page.getByText("Disconnect")).toBeVisible();
  const initialBalanceText = await page.getByText("Balance:").innerText();
  const initialBalance = +initialBalanceText.replace("Balance: ", "").replace(" ETH", "");
  await expect(initialBalance, "Balance should be non-zero after initial funding").toBeGreaterThan(0);

  // Step 2: disconnect then reconnect with session using the same passkey credential
  await page.getByRole("button", { name: "Disconnect", exact: true }).click();
  await expect(page.getByRole("button", { name: "Connect with Session", exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Connect with Session", exact: true }).click();
  await page.waitForTimeout(2000);
  const sessionPopup = page.context().pages()[1];
  await expect(sessionPopup.getByText("Act on your behalf")).toBeVisible();

  const sessionClient = await sessionPopup.context().newCDPSession(sessionPopup);
  await sessionClient.send("WebAuthn.enable");
  const sessionAuthenticator = await sessionClient.send("WebAuthn.addVirtualAuthenticator", {
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
  await sessionClient.send("WebAuthn.addCredential", {
    authenticatorId: sessionAuthenticator.authenticatorId,
    credential: newCredential!,
  });

  await expect(sessionPopup.getByText("Authorize ZKsync SSO Demo")).toBeVisible();
  await sessionPopup.getByTestId("connect").click();

  await page.waitForTimeout(2000);
  await expect(page.getByText("Disconnect")).toBeVisible();
  const sessionBalanceText = await page.getByText("Balance:").innerText();
  const sessionStartBalance = +sessionBalanceText.replace("Balance: ", "").replace(" ETH", "");

  // Step 3: send ETH under session (no extra signing step expected)
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();
  const sessionEndBalanceText = await page.getByText("Balance:").innerText();
  const sessionEndBalance = +sessionEndBalanceText.replace("Balance: ", "").replace(" ETH", "");

  await expect(sessionStartBalance, "Balance after transfer should be ~0.1 ETH less").toBeGreaterThan(sessionEndBalance + 0.09);
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
  let newCredential: WebAuthnCredential | null = null;
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
  await expect(popup.getByText("localhost:3005")).toBeVisible();
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

test("Create account with Paymaster and send ETH (no session)", async ({ page }) => {
  // Create a basic passkey account without session
  await page.getByRole("button", { name: "Connect", exact: true }).click();

  // Ensure popup is displayed and prepare logging
  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText(/Connect to|Act on your behalf/)).toBeVisible();
  popup.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn virtual authenticator for passkey creation
  const client = await popup.context().newCDPSession(popup);
  await client.send("WebAuthn.enable");
  let newCredential: WebAuthnCredential | null = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    newCredential = credentialAdded.credential;
  });
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

  // Complete signup
  const signupBtn = popup.getByTestId("signup");
  if (await signupBtn.isVisible()) {
    await signupBtn.click();
    await expect(popup.getByText(/Connect to ZKsync SSO Demo|Authorize ZKsync SSO Demo/)).toBeVisible();
  }
  await popup.getByTestId("connect").click();

  await expect(newCredential).not.toBeNull();

  // Account created
  await page.waitForTimeout(2000);
  await expect(page.getByText("Disconnect")).toBeVisible();

  // Capture starting balance
  const startBalanceText = await page.getByText("Balance:").innerText();
  const startBalance = +startBalanceText.replace("Balance: ", "").replace(" ETH", "");

  // Send 0.1 ETH (paymaster should sponsor gas)
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();

  // Validate balance decreased by ~0.1 ETH
  const endBalanceText = await page.getByText("Balance:").innerText();
  const endBalance = +endBalanceText.replace("Balance: ", "").replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be ~0.1 ETH less").toBeGreaterThan(endBalance + 0.09);
});

test("Create session account with Paymaster and send ETH", async ({ page }) => {
  // Trigger session connection with paymaster sponsorship
  await page.getByRole("button", { name: "Connect Session (Paymaster)", exact: true }).click();

  // Ensure popup is displayed and prepare logging
  await page.waitForTimeout(2000);
  const popup = page.context().pages()[1];
  await expect(popup.getByText(/Connect to|Act on your behalf/)).toBeVisible();
  popup.on("console", (msg) => {
    if (msg.type() === "error") console.log(`Auth server error console: "${msg.text()}"`);
  });
  popup.on("pageerror", (exception) => {
    console.log(`Auth server uncaught exception: "${exception}"`);
  });

  // Setup WebAuthn virtual authenticator for passkey creation (first-time session)
  const client = await popup.context().newCDPSession(popup);
  await client.send("WebAuthn.enable");
  let newCredential: WebAuthnCredential | null = null;
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    newCredential = credentialAdded.credential;
  });
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

  // If signup is shown, complete signup before connect; otherwise proceed to authorize/connect
  const signupBtn = popup.getByTestId("signup");
  if (await signupBtn.isVisible()) {
    await signupBtn.click();
    await expect(popup.getByText(/Connect to ZKsync SSO Demo|Authorize ZKsync SSO Demo/)).toBeVisible();
  } else {
    await expect(popup.getByText(/Authorize ZKsync SSO Demo/)).toBeVisible();
  }
  await popup.getByTestId("connect").click();

  await expect(newCredential).not.toBeNull();

  // Connected via session + paymaster
  await page.waitForTimeout(2000);
  await expect(page.getByText("Disconnect")).toBeVisible();

  // Capture starting balance
  const startBalanceText = await page.getByText("Balance:").innerText();
  const startBalance = +startBalanceText.replace("Balance: ", "").replace(" ETH", "");

  // Send under session (paymaster should sponsor fees)
  await page.getByRole("button", { name: "Send 0.1 ETH", exact: true }).click();
  await expect(page.getByRole("button", { name: "Send 0.1 ETH", exact: true })).toBeEnabled();

  // Validate balance decreased by ~0.1 ETH
  const endBalanceText = await page.getByText("Balance:").innerText();
  const endBalance = +endBalanceText.replace("Balance: ", "").replace(" ETH", "");
  await expect(startBalance, "Balance after transfer should be ~0.1 ETH less").toBeGreaterThan(endBalance + 0.09);
});
