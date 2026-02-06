import {
  clickAndWaitForPopup,
  reuseCredential,
  setupPopupLogging,
  setupWebAuthnForSignup,
  waitForServicesToLoad,
} from "@examples/shared-test-utils";
import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await waitForServicesToLoad(page, {
    appUrl: "/",
    appSelector: "text=Let's Go",
  });

  // Pre-warm auth-server dependencies by visiting it directly
  // This prevents Vite HMR reloads during the actual test
  await page.goto("http://localhost:3002/confirm?origin=http://localhost:3006");
  await page.waitForLoadState("networkidle");

  await page.goto("/");
  await expect(page.getByText("Let's Go")).toBeVisible();
});

test("Create account and mint NFT (no sessions)", async ({ page, context: _context }) => {
  // Set a longer timeout for this test - involves contract deployment and multiple transactions
  test.setTimeout(120000); // 2 minutes

  // Click the Let's Go button and wait for auth popup
  const popup = await clickAndWaitForPopup(
    page.getByRole("button", { name: "Let's Go" }),
  );

  // Set up popup logging
  setupPopupLogging(popup, "Auth Server");

  // Wait for popup to be fully loaded (including all Vite dependency optimizations)
  await popup.waitForLoadState("networkidle");
  await expect(popup.getByTestId("signup")).toBeVisible({ timeout: 60000 });

  // Setup WebAuthn for signup
  const { getCredential } = await setupWebAuthnForSignup(popup);

  // Click Sign Up - this will create account (no session)
  await popup.getByTestId("signup").click();

  // Wait for network to settle after account deployment
  await popup.waitForLoadState("networkidle");

  // Without sessions, auth-server navigates to /confirm/connect after account creation
  // We need to click the "Connect" button to complete the handshake
  await expect(popup.getByTestId("connect")).toBeVisible({ timeout: 30000 });
  await popup.getByTestId("connect").click();

  // Get the credential for later transaction approvals
  const credential = getCredential();

  // Wait for popup to close
  try {
    await popup.waitForEvent("close", { timeout: 15000 });
  } catch {
    // Popup might not close automatically - check if we navigated anyway
    if (!popup.isClosed()) {
      // Continue anyway, the handshake likely completed
    }
  }

  // Wait for the main page to navigate to /mint
  await page.waitForURL("**/mint", { timeout: 15000 });

  // Now handle the "Mint 100% free NFT" transaction
  // Without sessions, this will open an approval popup
  const mintButton = page.getByRole("button", { name: "Mint 100% free NFT" });

  // Check if we have a credential for transaction approval
  if (credential) {
    // Click and wait for approval popup
    const mintPopup = await clickAndWaitForPopup(mintButton);

    // Wait for popup to stabilize - give it time to fully load
    // This prevents issues with Vite HMR or slow dependency loading
    await mintPopup.waitForTimeout(500);

    if (mintPopup.isClosed()) {
      throw new Error("Mint popup closed unexpectedly before confirmation");
    }

    setupPopupLogging(mintPopup, "Mint Approval");

    // Wait for popup to fully load and confirm button to be visible
    await mintPopup.waitForLoadState("networkidle");
    await expect(mintPopup.getByTestId("confirm")).toBeVisible({ timeout: 10000 });

    // Now set up WebAuthn with the captured credential
    await reuseCredential(mintPopup, credential);

    // Click confirm button
    await mintPopup.getByTestId("confirm").click();

    // Wait for popup to close
    await mintPopup.waitForEvent("close", { timeout: 30000 });
  } else {
    // No credential - sessions might be enabled, just click
    await mintButton.click();
  }

  // Wait a moment for the transaction to be submitted
  await page.waitForTimeout(2000);

  // Wait for mint to complete
  await expect(page.getByTestId("spinner")).not.toBeVisible({ timeout: 30000 });

  // Verify we got the NFT
  await expect(page.getByText("You've got Zeek.")).toBeVisible({ timeout: 10000 });

  // Send a friend the NFT
  const richWallet0 = "0x36615Cf349d7F6344891B1e7CA7C72883F5dc049";
  await page.getByPlaceholder("Wallet address").fill(richWallet0);

  const sendButton = page.getByRole("button", { name: "Mint and send" });

  if (credential) {
    // Click and wait for approval popup
    const sendPopup = await clickAndWaitForPopup(sendButton);
    setupPopupLogging(sendPopup, "Send Approval");

    // Wait for popup to fully load (including Vite dependency optimization)
    await sendPopup.waitForLoadState("networkidle");
    await reuseCredential(sendPopup, credential);

    // Click confirm button
    await expect(sendPopup.getByTestId("confirm")).toBeVisible({ timeout: 10000 });
    await sendPopup.getByTestId("confirm").click();

    // Wait for popup to close
    await sendPopup.waitForEvent("close", { timeout: 30000 });
  } else {
    // No credential - sessions might be enabled, just click
    await sendButton.click();
  }

  // Wait for send to complete
  await expect(page.getByTestId("spinner")).not.toBeVisible({ timeout: 30000 });
  await expect(page.getByText("You've sent the minted copy to")).toBeVisible({ timeout: 15000 });
});
