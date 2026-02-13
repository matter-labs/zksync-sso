import type { BrowserContext, Locator, Page } from "@playwright/test";

/**
 * Click a locator and wait for a popup to open.
 * Sets up the listener BEFORE clicking to avoid race conditions.
 *
 * @param locator - The element to click that triggers the popup
 * @param timeout - Max time to wait for popup (default 30s)
 * @returns The popup page
 */
export async function clickAndWaitForPopup(
  locator: Locator,
  timeout = 30000,
): Promise<Page> {
  const context = locator.page().context();
  const popupPromise = context.waitForEvent("page", { timeout });
  await locator.click();
  const popup = await popupPromise;
  return popup;
}

/**
 * Wait for an existing popup in the context (when popup already opened).
 * Useful when the click has already happened.
 *
 * @param context - Browser context to check for popups
 * @param timeout - Max time to wait (default 5s)
 * @returns The popup page or null if not found
 */
export async function getExistingPopup(
  context: BrowserContext,
  timeout = 5000,
): Promise<Page | null> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    const pages = context.pages();
    if (pages.length > 1) {
      return pages[1];
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  return null;
}

/**
 * Set up console logging on a popup for debugging
 *
 * @param popup - The popup page
 * @param prefix - Prefix for log messages (default "Popup")
 */
export function setupPopupLogging(popup: Page, prefix = "Popup"): void {
  popup.on("console", (msg) => {
    if (msg.type() === "error") {
      console.log(`[${prefix}] ERROR: ${msg.text()}`);
    } else if (msg.type() === "log") {
      console.log(`[${prefix}] ${msg.text()}`);
    }
  });
  popup.on("pageerror", (exception) => {
    console.log(`[${prefix}] Uncaught exception: ${exception}`);
  });
}

/**
 * Wait for popup to close
 *
 * @param popup - The popup page
 * @param timeout - Max time to wait (default 10s)
 */
export async function waitForPopupToClose(
  popup: Page,
  timeout = 10000,
): Promise<void> {
  await popup.waitForEvent("close", { timeout });
}
