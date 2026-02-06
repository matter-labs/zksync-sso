import type { Page } from "@playwright/test";

/**
 * Service configuration for waiting
 */
export interface ServiceConfig {
  name: string;
  url: string;
  selector: string;
}

/**
 * Default services used by example apps
 */
export const DEFAULT_SERVICES: ServiceConfig[] = [
  {
    name: "Auth Server",
    url: "http://localhost:3002",
    selector: "[data-testid='signup']",
  },
];

/**
 * Wait for a single service to be ready
 *
 * @param page - Playwright page
 * @param service - Service configuration
 * @param maxRetries - Maximum retry attempts (default 30)
 */
export async function waitForService(
  page: Page,
  service: ServiceConfig,
  maxRetries = 30,
): Promise<void> {
  let retryCount = 0;

  await page.goto(service.url);
  let element = page.locator(service.selector);

  while (!(await element.isVisible()) && retryCount < maxRetries) {
    await page.waitForTimeout(1000);
    element = page.locator(service.selector);
    retryCount++;
    console.log(`Waiting for ${service.name} to load (retry ${retryCount})...`);
  }

  if (retryCount >= maxRetries) {
    throw new Error(`${service.name} failed to load after ${maxRetries} retries`);
  }

  console.log(`${service.name} loaded`);
}

/**
 * Wait for multiple services to be ready
 *
 * @param page - Playwright page
 * @param services - Array of service configurations (default: auth server)
 * @param appUrl - The main app URL to check first
 * @param appSelector - Selector to verify app is ready
 */
export async function waitForServicesToLoad(
  page: Page,
  options: {
    services?: ServiceConfig[];
    appUrl?: string;
    appSelector?: string;
    maxRetries?: number;
  } = {},
): Promise<void> {
  const {
    services = DEFAULT_SERVICES,
    appUrl = "/",
    appSelector = "button",
    maxRetries = 30,
  } = options;

  // First wait for the main app
  let retryCount = 0;
  await page.goto(appUrl);
  let appElement = page.locator(appSelector).first();

  while (!(await appElement.isVisible()) && retryCount < maxRetries) {
    await page.waitForTimeout(1000);
    appElement = page.locator(appSelector).first();
    retryCount++;
    console.log(`Waiting for app to load (retry ${retryCount})...`);
  }

  if (retryCount >= maxRetries) {
    throw new Error(`App failed to load after ${maxRetries} retries`);
  }

  console.log("App loaded");

  // Then wait for each service
  for (const service of services) {
    await waitForService(page, service, maxRetries);
  }
}
