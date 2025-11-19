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

test("Find addresses by passkey credential ID", async ({ page }) => {
  // Use Anvil account #4 for this test to avoid nonce conflicts
  await page.goto("/web-sdk-test?fundingAccount=4");
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

  // Get the deployed account address
  const deployedAddressElement = page.getByTestId("deployed-account-address");
  await expect(deployedAddressElement).toBeVisible({ timeout: 10000 });
  const deployedAddress = await deployedAddressElement.textContent();
  console.log(`Deployed account address: ${deployedAddress}`);

  // Step 4: Find addresses by passkey
  console.log("Step 4: Finding addresses by passkey credential ID...");

  // Verify the "Find Addresses by Passkey" section is visible
  const findAddressesSection = page.getByTestId("find-addresses-section");
  await expect(findAddressesSection).toBeVisible();

  // Click the "Scan Passkey to Find Accounts" button - this will automatically find accounts
  const scanPasskeyButton = page.getByTestId("scan-passkey-button");
  await expect(scanPasskeyButton).toBeVisible();
  await scanPasskeyButton.click();

  // Wait for the results to appear (scanning + finding happens automatically)
  const foundAddressesResult = page.getByTestId("found-addresses-result");
  await expect(foundAddressesResult).toBeVisible({ timeout: 15000 });

  // Verify "Associated Accounts:" label is visible
  await expect(page.getByText("Associated Accounts:")).toBeVisible();

  // Get all found addresses
  const foundAddressList = page.getByTestId("found-addresses-list");
  await expect(foundAddressList).toBeVisible();

  const addressItems = foundAddressList.getByTestId("found-address-item");
  const addressCount = await addressItems.count();
  console.log(`Found ${addressCount} address(es)`);

  // Verify that the deployed address is in the list
  let foundDeployedAddress = false;
  for (let i = 0; i < addressCount; i++) {
    const addressText = await addressItems.nth(i).textContent();
    console.log(`  Address ${i + 1}: ${addressText}`);

    // Check if this address item contains the deployed address
    if (addressText && addressText.includes(deployedAddress || "")) {
      foundDeployedAddress = true;
      console.log("  ✓ Deployed address found in list!");
    }
  }

  // Assert that we found the deployed address
  expect(foundDeployedAddress, `Deployed address ${deployedAddress} should be in the found addresses list`).toBe(true);

  console.log("✓ Successfully found addresses by passkey");
  console.log("✓ Verified deployed address is in the found addresses list");

  console.log("✅ Find addresses by passkey test completed successfully!");
});

test("Deploy with session support and send transaction using session key", async ({ page }) => {
  // Capture console messages from the browser
  page.on("console", (msg) => {
    console.log(`[BROWSER ${msg.type()}]:`, msg.text());
  });

  // Capture page errors
  page.on("pageerror", (err) => {
    console.error("[BROWSER ERROR]:", err);
  });

  // Use Anvil account #7 for session tests (renumbered to avoid conflict with passkey test #4)
  await page.goto("/web-sdk-test?fundingAccount=7");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  console.log("Step 0.5: Enabling session support before deployment...");

  // Enable "Deploy with Session Support" BEFORE deploying
  const deployWithSessionCheckbox = page.getByLabel("Deploy with Session Support");
  await expect(deployWithSessionCheckbox).toBeVisible();
  await deployWithSessionCheckbox.check();

  console.log("✓ Session support enabled for deployment");

  console.log("Step 1: Deploying smart account...");
  await page.getByRole("button", { name: "Deploy Account" }).click();

  // Wait for deployment to complete
  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });

  // Verify deployment result
  await expect(page.getByText("Account Address:")).toBeVisible();
  await expect(page.getByText("EOA Signer:")).toBeVisible();

  console.log("✓ Smart account deployed successfully");

  // Step 2: Fund Smart Account
  console.log("Step 2: Funding smart account...");

  const amountInput = page.locator("input[placeholder=\"0.1\"]");
  await expect(amountInput).toBeVisible();
  await amountInput.fill("0.1");

  await page.getByRole("button", { name: "Fund Smart Account" }).click();

  // Wait for funding transaction to complete
  await expect(page.getByText("Transaction Hash:")).toBeVisible({ timeout: 30000 });

  console.log("✓ Smart account funded successfully");

  // Step 3: Enable Session Configuration
  console.log("Step 3: Enabling session configuration...");

  // Find and enable the session checkbox
  const sessionCheckbox = page.getByLabel("Enable Session Configuration");
  await expect(sessionCheckbox).toBeVisible();
  await sessionCheckbox.check();

  // Verify session configuration UI is visible
  await expect(page.getByText("Session Validator Address")).toBeVisible();
  await expect(page.getByText("Session Signer (auto-generated)")).toBeVisible();
  await expect(page.getByText("Expires At (timestamp)")).toBeVisible();
  await expect(page.getByText("Max Fee (wei)")).toBeVisible();

  // Verify session signer was auto-generated
  const sessionSignerInput = page.getByText("Session Signer (auto-generated)").locator("..").locator("input");
  await expect(sessionSignerInput).toBeVisible();
  const sessionSignerValue = await sessionSignerInput.inputValue();
  expect(sessionSignerValue).toMatch(/^0x[a-fA-F0-9]{40}$/);

  console.log("✓ Session configuration enabled, session signer auto-generated:", sessionSignerValue);

  // Step 3.5: Create Session on-chain
  console.log("Step 3.5: Creating session on-chain...");

  // Ensure allowed recipient matches the target we will send to
  await expect(page.getByText("Allowed Recipient (address)")).toBeVisible();
  await page.getByText("Allowed Recipient (address)").locator("..").locator("input").fill(
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
  );

  // Wait for the Create Session button to appear
  await expect(page.getByRole("button", { name: "Create Session" })).toBeVisible();

  // Click Create Session button
  await page.getByRole("button", { name: "Create Session" }).click();

  // Wait a bit for the request to process
  await page.waitForTimeout(5000);

  // Check what's visible on the page
  const pageContent = await page.content();
  if (pageContent.includes("Error Creating Session")) {
    const errorDiv = page.locator(".bg-red-50").first();
    if (await errorDiv.isVisible()) {
      const errorText = await errorDiv.textContent();
      console.error("❌ Session creation error:", errorText);
      throw new Error(`Session creation failed: ${errorText}`);
    }
  }

  // Wait for session creation to complete
  await expect(page.getByText("Session Created Successfully!")).toBeVisible({ timeout: 55000 });
  console.log("✓ Session created on-chain");

  // Step 4: Send Transaction Using Session Key
  console.log("Step 4: Sending transaction using session key...");

  // Wait for the Send Session Transaction section to appear
  await expect(page.getByRole("button", { name: "Send Session Transaction" })).toBeVisible();

  // Fill in target address for session transaction
  const sessionTargetInput = page.locator("input[placeholder=\"0x...\"]").nth(0);
  await expect(sessionTargetInput).toBeVisible();
  await sessionTargetInput.fill("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC");

  // Fill in session transaction amount
  const sessionAmountInput = page.locator("input[placeholder=\"0.001\"]").last();
  await expect(sessionAmountInput).toBeVisible();
  await sessionAmountInput.fill("0.001");

  // Click Send Session Transaction button
  await page.getByRole("button", { name: "Send Session Transaction" }).click();

  // Wait for transaction to complete
  await expect(page.getByText("Success!")).toBeVisible({ timeout: 60000 });
  await expect(page.getByText("UserOp Hash:")).toBeVisible();

  // Verify we have a UserOp hash for the session transaction
  const sessionUserOpHash = page.locator("code").filter({ hasText: /^0x[a-fA-F0-9]{64}/ }).last();
  await expect(sessionUserOpHash).toBeVisible();

  console.log("✓ Transaction sent using session key successfully");

  console.log("✅ All session steps completed successfully!");
});

test("Deploy account, enable session, modify session config, and send transaction", async ({ page }) => {
  // Use Anvil account #8 for this test (renumbered)
  await page.goto("/web-sdk-test?fundingAccount=8");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  console.log("Step 0.5: Enabling session support before deployment...");

  // Enable "Deploy with Session Support" BEFORE deploying
  const deployWithSessionCheckbox = page.getByLabel("Deploy with Session Support");
  await expect(deployWithSessionCheckbox).toBeVisible();
  await deployWithSessionCheckbox.check();

  console.log("✓ Session support enabled for deployment");

  console.log("Step 1: Deploying smart account...");
  await page.getByRole("button", { name: "Deploy Account" }).click();

  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });
  console.log("✓ Smart account deployed");

  // Step 2: Fund Smart Account
  console.log("Step 2: Funding smart account...");
  const amountInput = page.locator("input[placeholder=\"0.1\"]");
  await amountInput.fill("0.15");
  await page.getByRole("button", { name: "Fund Smart Account" }).click();
  await expect(page.getByText("Transaction Hash:")).toBeVisible({ timeout: 30000 });
  console.log("✓ Smart account funded with 0.15 ETH");

  // Step 3: Enable Session and Modify Configuration
  console.log("Step 3: Configuring session with custom parameters...");

  const sessionCheckbox = page.getByLabel("Enable Session Configuration");
  await sessionCheckbox.check();

  // Verify session UI is visible
  await expect(page.getByText("Session Validator Address")).toBeVisible();

  // Modify expiry timestamp (set to 2 hours from now)
  const expiryInput = page.getByText("Expires At (timestamp)").locator("..").locator("input");
  await expect(expiryInput).toBeVisible();
  const twoHoursFromNow = Math.floor(Date.now() / 1000) + 7200;
  await expiryInput.fill(twoHoursFromNow.toString());

  // Modify fee limit (set to 0.002 ETH in wei)
  const feeLimitInput = page.getByText("Max Fee (wei)").locator("..").locator("input");
  await expect(feeLimitInput).toBeVisible();
  await feeLimitInput.fill("2000000000000000"); // 0.002 ETH

  console.log("✓ Session configured with custom expiry and fee limit");

  // Step 3.5: Create Session on-chain
  console.log("Step 3.5: Creating session on-chain...");

  // Ensure allowed recipient matches the target we will send to
  await expect(page.getByText("Allowed Recipient (address)")).toBeVisible();
  await page.getByText("Allowed Recipient (address)").locator("..").locator("input").fill(
    "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
  );

  await expect(page.getByRole("button", { name: "Create Session" })).toBeVisible();
  await page.getByRole("button", { name: "Create Session" }).click();
  await expect(page.getByText("Session Created Successfully!")).toBeVisible({ timeout: 60000 });

  console.log("✓ Session created on-chain with custom configuration");

  // Step 4: Send Transaction Using Session
  console.log("Step 4: Sending transaction using configured session...");

  await expect(page.getByRole("button", { name: "Send Session Transaction" })).toBeVisible();

  const sessionTargetInput = page.locator("input[placeholder=\"0x...\"]").nth(0);
  await sessionTargetInput.fill("0x90F79bf6EB2c4f870365E785982E1f101E93b906");

  const sessionAmountInput = page.locator("input[placeholder=\"0.001\"]").last();
  await sessionAmountInput.fill("0.001"); // Match Rust test value

  await page.getByRole("button", { name: "Send Session Transaction" }).click();

  // Wait for success
  await expect(page.getByText("Success!")).toBeVisible({ timeout: 60000 });
  await expect(page.getByText("UserOp Hash:")).toBeVisible();

  console.log("✓ Transaction sent successfully with custom session configuration");

  console.log("✅ Session configuration and transaction test completed!");
});

test("Deploy account with session validator pre-installed", async ({ page }) => {
  // Use Anvil account #9 for this test (renumbered)
  await page.goto("/web-sdk-test?fundingAccount=9");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  console.log("Step 1: Enabling session deployment...");

  // Enable "Deploy with Session Support" before deployment
  const deployWithSessionCheckbox = page.getByLabel("Deploy with Session Support");
  await expect(deployWithSessionCheckbox).toBeVisible();
  await deployWithSessionCheckbox.check();

  // Verify deployment option is enabled
  await expect(deployWithSessionCheckbox).toBeChecked();

  console.log("Step 2: Deploying smart account with session validator...");
  await page.getByRole("button", { name: "Deploy Account" }).click();

  // Wait for deployment to complete
  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });

  // Verify deployment success message includes session validator (check the success message specifically)
  await expect(
    page.locator(".text-blue-600").filter({ hasText: /Session validator.*pre-installed/i }),
  ).toBeVisible();

  console.log("✓ Smart account deployed with session validator pre-installed");

  // Step 3: Fund Smart Account
  console.log("Step 3: Funding smart account...");
  const amountInput = page.locator("input[placeholder=\"0.1\"]");
  await amountInput.fill("0.1");
  await page.getByRole("button", { name: "Fund Smart Account" }).click();
  await expect(page.getByText("Transaction Hash:")).toBeVisible({ timeout: 30000 });
  console.log("✓ Smart account funded");

  // Step 4: Enable session configuration (validator already installed)
  console.log("Step 4: Configuring session after deployment...");

  // Enable session configuration
  const sessionCheckbox = page.getByLabel("Enable Session Configuration");
  await expect(sessionCheckbox).toBeVisible();
  await sessionCheckbox.check();

  // Verify session signer was auto-generated
  const sessionSignerInput = page.getByText("Session Signer (auto-generated)").locator("..").locator("input");
  await expect(sessionSignerInput).toBeVisible();
  const sessionSignerValue = await sessionSignerInput.inputValue();
  expect(sessionSignerValue).toMatch(/^0x[a-fA-F0-9]{40}$/);

  console.log("✓ Session configured with auto-generated signer:", sessionSignerValue);

  // Step 4.5: Create Session on-chain
  console.log("Step 4.5: Creating session on-chain...");

  // Ensure allowed recipient matches the target we will send to
  await expect(page.getByText("Allowed Recipient (address)")).toBeVisible();
  await page.getByText("Allowed Recipient (address)").locator("..").locator("input").fill(
    "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
  );

  await expect(page.getByRole("button", { name: "Create Session" })).toBeVisible();
  await page.getByRole("button", { name: "Create Session" }).click();
  await expect(page.getByText("Session Created Successfully!")).toBeVisible({ timeout: 60000 });

  console.log("✓ Session created on-chain using pre-installed validator");

  // Step 5: Send Transaction Using Session Key
  console.log("Step 5: Sending transaction using pre-installed session...");

  await expect(page.getByRole("button", { name: "Send Session Transaction" })).toBeVisible();

  const sessionTargetInput = page.locator("input[placeholder=\"0x...\"]").nth(0);
  await sessionTargetInput.fill("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65");

  const sessionAmountInput = page.locator("input[placeholder=\"0.001\"]").last();
  await sessionAmountInput.fill("0.001");

  await page.getByRole("button", { name: "Send Session Transaction" }).click();

  // Wait for success
  await expect(page.getByText("Success!")).toBeVisible({ timeout: 60000 });
  await expect(page.getByText("UserOp Hash:")).toBeVisible();

  console.log("✓ Transaction sent successfully using pre-installed session validator");

  console.log("✅ Deploy with session validator test completed!");
});

test("Sign and verify ERC-7739 typed data via ERC-1271", async ({ page }) => {
  // Use Anvil account #10 for this test to avoid nonce conflicts
  await page.goto("/web-sdk-test?fundingAccount=10");
  await expect(page.getByText("ZKSync SSO Web SDK Test")).toBeVisible();

  // Wait for SDK to load
  await expect(page.getByText("SDK Loaded:")).toBeVisible();
  await expect(page.getByText("Yes")).toBeVisible({ timeout: 10000 });

  // Ensure a smart account is deployed so we have an address to verify against
  await page.getByRole("button", { name: "Deploy Account" }).click();
  await expect(page.getByText("Account Deployed Successfully!")).toBeVisible({ timeout: 30000 });

  // Wait for typed-data section to be visible
  const typedDataSection = page.getByTestId("typed-data-section");
  await expect(typedDataSection).toBeVisible({ timeout: 10000 });

  // Sign the ERC-7739 typed data (no connection needed with new implementation)
  const tdSection = page.getByTestId("typed-data-section");
  await expect(tdSection).toBeVisible({ timeout: 15000 });
  // Ensure ERC1271 caller address is present before interacting (ensures readiness)
  const callerAddr = tdSection.getByTestId("erc1271-caller-address");
  await expect(callerAddr).toBeVisible({ timeout: 30000 });
  // Small settle wait to allow client-side hydration to render actions
  await page.waitForTimeout(250);
  // Use E2E bridge to trigger sign to avoid internal gating
  const signBridge = page.getByTestId("typeddata-sign-bridge");
  await expect(signBridge).toBeVisible({ timeout: 20000 });
  await signBridge.click();
  await expect(tdSection.getByTestId("typeddata-signature")).toBeVisible({ timeout: 20000 });

  // Verify on-chain via ERC-1271 helper contract
  const verifyBridge = page.getByTestId("typeddata-verify-bridge");
  await expect(verifyBridge).toBeVisible({ timeout: 20000 });
  await verifyBridge.click();
  const result = tdSection.getByTestId("typeddata-verify-result");
  await expect(result).toBeVisible({ timeout: 30000 });
  await expect(result).toContainText("Valid");

  console.log("✓ ERC-7739 typed-data signature verified on-chain via ERC-1271");
});
