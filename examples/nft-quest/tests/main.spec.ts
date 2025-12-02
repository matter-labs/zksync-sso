import { expect, type Page, test } from "@playwright/test";

async function waitForAppToLoad(page: Page): Promise<void> {
  const maxRetryAttempts = 30;
  let retryCount = 0;

  // Wait for nft-quest to finish loading
  await page.goto("/");
  let demoButton = page.getByText("Let's Go");
  while (!(await demoButton.isVisible()) && retryCount < maxRetryAttempts) {
    await page.waitForTimeout(1000);
    demoButton = page.getByText("Let's Go");
    retryCount++;

    console.log(`Waiting for nft quest app to load (retry ${retryCount})...`);
  }
  console.log("NFT Quest App loaded");
}

async function setupWebAuthn(page: Page) {
  // Setup WebAuthn virtual authenticator via Chrome DevTools Protocol
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
  console.log(`WebAuthn Authenticator ID: ${result.authenticatorId}`);
  return { client, authenticatorId: result.authenticatorId };
}

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Main page error console: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Main page uncaught exception: "${exception}"`);
  });

  await waitForAppToLoad(page);
  await page.goto("/");
  await expect(page.getByText("Let's Go")).toBeVisible();
});

test("Create account with passkey and mint NFT", async ({ page }) => {
  // Setup WebAuthn virtual authenticator
  const { client } = await setupWebAuthn(page);
  // Track credential creation for debugging
  client.on("WebAuthn.credentialAdded", (credentialAdded) => {
    console.log("New Passkey credential added");
    console.log(`Authenticator ID: ${credentialAdded.authenticatorId}`);
    console.log(`Credential: ${credentialAdded.credential}`);
  });

  // Listen for console messages and errors

  page.on("console", (msg) => console.log(`BROWSER ${msg.type()}: ${msg.text()}`));

  page.on("pageerror", (error) => console.error(`PAGE ERROR: ${error.message}`));

  // Click the Let's Go button to start
  await page.getByRole("button", { name: "Let's Go" }).click();
  // With the workaround, clicking "Let's Go" triggers passkey registration
  // and automatically navigates to /mint page
  // Wait for navigation to mint page
  await page.waitForURL("**/mint", { timeout: 10000 });

  // Should now see the mint button on the /mint page
  await expect(page.getByRole("button", { name: "Mint 100% free NFT" })).toBeVisible({ timeout: 10000 });
  // Mint your NFT
  await page.getByRole("button", { name: "Mint 100% free NFT" }).click();

  // Wait for mint transaction to complete
  await expect(page.getByTestId("spinner")).not.toBeVisible({ timeout: 60000 });

  // Verify NFT was minted successfully
  await expect(page.getByText("You've got Zeek.")).toBeVisible();

  // Send a friend the NFT
  const randomRecipient = "0x1234567890123456789012345678901234567890";
  await page.getByPlaceholder("Wallet address").fill(randomRecipient);
  await page.getByRole("button", { name: "Mint and send" }).click();
  await expect(page.getByTestId("spinner")).not.toBeVisible({ timeout: 60000 });
  await expect(page.getByText("You've sent the minted copy to")).toBeVisible({ timeout: 15000 });

  console.log("Test completed successfully");
});
