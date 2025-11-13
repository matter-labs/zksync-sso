/* eslint-disable no-console */
import { expect, type Page, test } from "@playwright/test";

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

test.beforeEach(async ({ page }) => {
  page.on("console", (msg) => {
    if (msg.type() === "error")
      console.log(`Main page error console: "${msg.text()}"`);
  });
  page.on("pageerror", (exception) => {
    console.log(`Main page uncaught exception: "${exception}"`);
  });

  await waitForServicesToLoad(page);
  // Note: Each test will navigate with its own fundingAccount parameter
});

test("Deploy, fund, and transfer from smart account", async ({ page }) => {
  // Use Anvil account #2 for this test to avoid nonce conflicts
  await page.goto("/web-sdk-test?fundingAccount=2");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  // Step 1: Deploy Account
  console.log("Step 1: Deploying smart account...");
  await page.getByRole("button", { name: "Deploy Account" }).click();

  // Wait for deployment to complete - look for the success message
  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });

  // Verify deployment result has required information
  await expect(page.getByText("Account Address:")).toBeVisible();
  await expect(page.getByText("EOA Signer:")).toBeVisible();

  console.log("✓ Smart account deployed successfully");

  // Step 2: Fund Smart Account
  console.log("Step 2: Funding smart account...");

  // Find and fill the amount input
  const amountInput = page.locator("input[placeholder=\"0.1\"]");
  await expect(amountInput).toBeVisible();
  await amountInput.fill("0.1");

  // Click the Fund Smart Account button
  await page.getByRole("button", { name: "Fund Smart Account" }).click();

  // Wait for funding transaction to complete - look for transaction hash
  await expect(page.getByText("Transaction Hash:")).toBeVisible({ timeout: 30000 });

  // Verify we have a transaction hash displayed
  const fundTxHash = page.locator("code").filter({ hasText: /^0x[a-fA-F0-9]{64}/ }).first();
  await expect(fundTxHash).toBeVisible();

  console.log("✓ Smart account funded successfully");

  // Step 3: Send Transaction from Smart Account
  console.log("Step 3: Sending transaction from smart account...");

  // Wait for the Send Transaction section to appear
  await expect(page.getByText("Step 2: Send Transaction from Smart Account")).toBeVisible();

  // Fill in recipient address - find input by locating the label first
  await expect(page.getByText("Recipient Address:")).toBeVisible();
  const recipientInput = page.getByText("Recipient Address:").locator("..").locator("input");
  await expect(recipientInput).toBeVisible();
  await recipientInput.fill("0x70997970C51812dc3A010C7d01b50e0d17dc79C8"); // Anvil account #2

  // Fill in transfer amount - find input by locating the label first
  await expect(page.getByText("Amount (ETH):")).toBeVisible();
  const transferAmountInput = page.getByText("Amount (ETH):").locator("..").locator("input");
  await expect(transferAmountInput).toBeVisible();
  await transferAmountInput.fill("0.001");

  // Click the Send with EOA button (default signing method)
  await page.getByRole("button", { name: "Send with EOA" }).click();

  // Wait for transaction to complete - look for transaction hash in the send section
  // We need to find the second occurrence of "Transaction Hash:" since fund also has one
  await expect(page.locator("strong:has-text(\"Transaction Hash:\")").nth(1)).toBeVisible({ timeout: 30000 });
  await expect(page.getByText("Transaction failed: Failed to submit UserOperation:")).not.toBeVisible();

  // Verify we have a transaction hash for the send
  const sendTxHash = page.locator("code").filter({ hasText: /^0x[a-fA-F0-9]{64}/ }).nth(1);
  await expect(sendTxHash).toBeVisible();

  console.log("✓ Transaction sent from smart account successfully");

  console.log("✅ All steps completed successfully!");
});

test("Deploy with passkey and send transaction using passkey", async ({ page }) => {
  // Use Anvil account #3 for this test to avoid nonce conflicts with the first test
  await page.goto("/web-sdk-test?fundingAccount=3");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  console.log("Step 1: Enabling passkey configuration...");

  // Enable passkey deployment
  const passkeyCheckbox = page.getByLabel("Enable Passkey Deployment");
  await expect(passkeyCheckbox).toBeVisible();
  await passkeyCheckbox.check();

  console.log("Step 2: Creating WebAuthn passkey...");

  // Create a virtual authenticator for testing
  const client = await page.context().newCDPSession(page);
  await client.send("WebAuthn.enable");
  await client.send("WebAuthn.addVirtualAuthenticator", {
    options: {
      protocol: "ctap2",
      transport: "usb",
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
    },
  });

  // Click Create New WebAuthn Passkey button
  await page.getByRole("button", { name: "Create New WebAuthn Passkey" }).click();

  // Wait for passkey creation to complete
  await expect(page.getByText("Passkey created successfully!")).toBeVisible({ timeout: 10000 });

  console.log("✓ WebAuthn passkey created successfully");

  // Step 3: Deploy Account with Passkey
  console.log("Step 3: Deploying smart account with passkey...");
  await page.getByRole("button", { name: "Deploy Account" }).click();

  // Wait for deployment to complete
  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });

  // Verify passkey is enabled in deployment result
  await expect(page.getByText("Passkey Enabled: Yes")).toBeVisible();

  console.log("✓ Smart account with passkey deployed successfully");

  // Step 4: Fund Smart Account
  console.log("Step 4: Funding smart account...");

  const amountInput = page.locator("input[placeholder=\"0.1\"]");
  await expect(amountInput).toBeVisible();
  await amountInput.fill("0.1");

  await page.getByRole("button", { name: "Fund Smart Account" }).click();

  // Wait for funding transaction to complete
  await expect(page.getByText("Transaction Hash:"), "Funding failed").toBeVisible({ timeout: 30000 });

  console.log("✓ Smart account funded successfully");

  // Step 5: Send Transaction using Passkey
  console.log("Step 5: Sending transaction using passkey...");

  // Select passkey as signing method
  const passkeyRadio = page.getByLabel("WebAuthn Passkey (Hardware Key)");
  await expect(passkeyRadio).toBeVisible();
  await passkeyRadio.check();

  // Fill in recipient address
  await expect(page.getByText("Recipient Address:")).toBeVisible();
  const recipientInput = page.getByText("Recipient Address:").locator("..").locator("input");
  await expect(recipientInput).toBeVisible();
  await recipientInput.fill("0x70997970C51812dc3A010C7d01b50e0d17dc79C8");

  // Fill in transfer amount
  await expect(page.getByText("Amount (ETH):")).toBeVisible();
  const transferAmountInput = page.getByText("Amount (ETH):").locator("..").locator("input");
  await expect(transferAmountInput).toBeVisible();
  await transferAmountInput.fill("0.001");

  // Click Send with Passkey button
  await page.getByRole("button", { name: "Send with Passkey" }).click();

  // Wait for transaction to complete
  await expect(page.getByText("Transaction successful! Tx hash:")).toBeVisible({ timeout: 60000 });
  await expect(page.getByText("Transaction failed: Failed to submit UserOperation:")).not.toBeVisible();

  // Verify we have a transaction hash for the send
  const sendTxHash = page.locator("code").filter({ hasText: /^0x[a-fA-F0-9]{64}/ }).nth(1);
  await expect(sendTxHash).toBeVisible();

  console.log("✓ Transaction sent using passkey successfully");

  console.log("✅ All passkey steps completed successfully!");
});
