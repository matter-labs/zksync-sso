import {
  clickAndWaitForPopup,
  reuseCredential,
  setupPopupLogging,
  setupWebAuthnForSignup,
  waitForServicesToLoad,
  type WebAuthnCredential,
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

test("Create account, then add session, then mint NFT", async ({ page }) => {
  // Set a longer timeout for this test - involves multiple transactions
  test.setTimeout(180000); // 3 minutes

  // Capture all console messages from the beginning to debug paymaster issues
  page.on("console", (msg) => {
    const text = msg.text();
    if (text.includes("[ConnectorStore]")) {
      console.log(`[Main Page] ${text}`);
    }
  });

  let capturedCredential: WebAuthnCredential | null = null;

  // ============================================
  // POPUP 1: Create account WITHOUT session
  // ============================================
  console.log("=== POPUP 1: Create account without session ===");

  const createAccountPopup = await clickAndWaitForPopup(
    page.getByRole("button", { name: "Let's Go" }),
  );

  setupPopupLogging(createAccountPopup, "Create Account");
  await createAccountPopup.waitForLoadState("networkidle");
  await expect(createAccountPopup.getByTestId("signup")).toBeVisible({ timeout: 60000 });

  // Setup WebAuthn for signup and capture the credential
  const { getCredential } = await setupWebAuthnForSignup(createAccountPopup);

  // Click Sign Up - this creates the passkey
  await createAccountPopup.getByTestId("signup").click();
  await createAccountPopup.waitForLoadState("networkidle");
  await createAccountPopup.waitForTimeout(2000);

  // Wait for the connect button to be visible
  await expect(createAccountPopup.getByTestId("connect")).toBeVisible({ timeout: 30000 });

  // The button should show "Create" since there's no session to authorize
  const connectButton = createAccountPopup.getByTestId("connect");
  const buttonText = await connectButton.textContent();
  console.log(`Connect button text: "${buttonText?.trim()}"`);

  // Handle "Continue" if scroll is needed, then click the action button
  if (buttonText?.trim() === "Continue") {
    await connectButton.click();
    await createAccountPopup.waitForTimeout(500);
  }

  // Click to create account (should be "Create" or "Connect")
  await connectButton.click();

  // Capture the credential before popup closes
  capturedCredential = getCredential();
  console.log("Captured credential:", capturedCredential?.credentialId ? "yes" : "no");

  // Wait for popup to signal completion (window.close() doesn't work in Playwright)
  // Instead, wait for the main page to navigate (which happens when popup sends result)
  // Then explicitly close the popup
  await page.waitForURL("**/mint", { timeout: 45000 });
  console.log("Main page navigated to /mint - popup completed its work");

  // Explicitly close the popup since window.close() doesn't work in automated tests
  if (!createAccountPopup.isClosed()) {
    await createAccountPopup.close();
  }
  console.log("Create account popup closed");

  // ============================================
  // POPUP 2: Add session to existing account
  // ============================================
  console.log("=== POPUP 2: Add session to existing account ===");

  // Verify the "Add Session" button is visible (since we connected without session)
  await expect(page.getByTestId("add-session")).toBeVisible({ timeout: 10000 });

  // Click "Add Session" button and wait for popup
  const sessionPopup = await clickAndWaitForPopup(
    page.getByTestId("add-session"),
  );

  setupPopupLogging(sessionPopup, "Add Session");
  await sessionPopup.waitForLoadState("networkidle");

  // Reuse the captured credential from signup for signing
  expect(capturedCredential).not.toBeNull();
  await reuseCredential(sessionPopup, capturedCredential!);
  console.log("Credential reused in session popup");

  // Wait for the session confirmation UI
  await expect(sessionPopup.getByTestId("connect")).toBeVisible({ timeout: 30000 });

  // The button should show "Connect" for existing logged-in users with session request
  const sessionConnectButton = sessionPopup.getByTestId("connect");
  let sessionButtonText = await sessionConnectButton.textContent();
  console.log(`Session connect button text: "${sessionButtonText?.trim()}"`);

  // Handle "Continue" if scroll is needed - need to scroll to bottom before button changes to "Connect"
  while (sessionButtonText?.trim() === "Continue") {
    await sessionConnectButton.click();
    // Wait for scroll animation and button text to update
    await sessionPopup.waitForTimeout(1000);
    sessionButtonText = await sessionConnectButton.textContent();
    console.log(`Session button text after scroll: "${sessionButtonText?.trim()}"`);
  }

  // Check for danger checkbox - ERC721 approve function triggers security warning
  // The NFT contract has an approve(address,uint256) function which has the same
  // signature as ERC20 approve, triggering the "dangerous action" check
  const dangerCheckbox = sessionPopup.getByText("I understand that by continuing, I risk losing my funds.");
  if (await dangerCheckbox.isVisible({ timeout: 2000 }).catch(() => false)) {
    console.log("Danger checkbox detected - checking it");
    await dangerCheckbox.click();
    await sessionPopup.waitForTimeout(500);
  }

  // Now button should say "Connect" - click to authorize session
  console.log("Clicking to authorize session...");
  await sessionConnectButton.click();

  // Wait for popup to signal completion (window.close() doesn't work in Playwright)
  // Wait for main page to detect session was added (Add Session button should disappear)
  await expect(page.getByTestId("add-session")).toBeHidden({ timeout: 45000 });
  console.log("Session added - Add Session button hidden");

  // Explicitly close the popup since window.close() doesn't work in automated tests
  if (!sessionPopup.isClosed()) {
    await sessionPopup.close();
  }
  console.log("Session popup closed");

  // ============================================
  // MINT NFT: Should work without popup
  // ============================================
  console.log("=== MINT NFT with session ===");

  // Add console listener to capture any errors during mint
  page.on("console", (msg) => {
    const type = msg.type();
    const text = msg.text();
    if (type === "error" || text.includes("Error") || text.includes("error")) {
      console.log(`[Mint Page ERROR] ${text}`);
    } else if (text.includes("mint") || text.includes("transaction") || text.includes("writeContract")) {
      console.log(`[Mint Page] ${text}`);
    }
  });

  // Give the page time to update after session is added
  await page.waitForTimeout(2000);

  // Diagnostic: check what paymaster the page actually sees
  const runtimePaymaster = await page.evaluate(() => {
    const nuxtApp = (window as unknown).__NUXT__;
    const payload = nuxtApp?.config?.public?.contracts;
    return { paymaster: payload?.paymaster, nft: payload?.nft, fullPayload: JSON.stringify(payload) };
  });
  console.log("[DIAGNOSTIC] Runtime paymaster from __NUXT__:", JSON.stringify(runtimePaymaster));

  // Now the "Mint 100% free NFT" button should be visible
  await expect(page.getByRole("button", { name: "Mint 100% free NFT" })).toBeVisible({ timeout: 10000 });
  console.log("Mint button visible, about to click...");

  const mintButton = page.getByRole("button", { name: "Mint 100% free NFT" });

  // Count pages before clicking mint
  const pageCountBefore = page.context().pages().length;

  // Click mint - no popup expected with sessions
  await mintButton.click();
  console.log("Clicked mint button");

  // Verify no new popup opened (session handles signing)
  await page.waitForTimeout(1000);
  const pageCountAfter = page.context().pages().length;
  expect(pageCountAfter).toBe(pageCountBefore);
  console.log("No popup opened for mint - session is working!");

  // Check for error message on page
  const errorVisible = await page.locator(".text-error-400").isVisible();
  if (errorVisible) {
    const errorText = await page.locator(".text-error-400").textContent();
    console.log(`[Mint Page Error UI] ${errorText}`);
  }

  // Wait for mint to complete (spinner should disappear or navigate to share page)
  console.log("Waiting for navigation to /mint/share...");
  await page.waitForURL("**/mint/share**", { timeout: 60000 });

  // Verify we got the NFT
  await expect(page.getByText("You've got Zeek.")).toBeVisible({ timeout: 10000 });
  console.log("NFT minted successfully!");

  // ============================================
  // SEND NFT: Should also work without popup
  // ============================================
  console.log("=== SEND NFT with session ===");

  // Fill recipient address
  const richWallet0 = "0x36615Cf349d7F6344891B1e7CA7C72883F5dc049";
  await page.getByPlaceholder("Wallet address").fill(richWallet0);

  // Click send - no popup expected with sessions
  const sendButton = page.getByRole("button", { name: "Mint and send" });
  await sendButton.click();

  // Wait for send to complete
  await expect(page.getByTestId("spinner")).not.toBeVisible({ timeout: 60000 });
  await expect(page.getByText("You've sent the minted copy to")).toBeVisible({ timeout: 15000 });
  console.log("NFT sent successfully!");
});
