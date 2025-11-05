import { describe, expect, test } from "vitest";

/**
 * Unit tests for deployAccountWebSdk
 *
 * Note: These tests verify the API shape. Full functionality is tested in:
 * - integration.test.ts (with actual web-SDK)
 * - E2E tests in examples/demo-app/tests/web-sdk-test.spec.ts
 *
 * The dynamic import of web-SDK happens before validation, so these tests
 * will fail if web-SDK is not built. To test error handling, see integration tests.
 */

describe("deployAccountWebSdk", () => {
  test("should be exported from module", async () => {
    const { deployAccountWebSdk } = await import("./deployAccountWebSdk.js");

    expect(deployAccountWebSdk).toBeDefined();
    expect(typeof deployAccountWebSdk).toBe("function");
  });

  test("should have correct type signature", async () => {
    const { deployAccountWebSdk } = await import("./deployAccountWebSdk.js");

    // Function should accept 2 parameters: client and args
    expect(deployAccountWebSdk.length).toBe(2);
  });

  test.skip("error handling tests require web-SDK to be built", () => {
    // These tests are skipped because the dynamic import happens before validation.
    // Error handling is tested in:
    // - packages/sdk/src/client/passkey/integration.test.ts
    // - examples/demo-app/tests/web-sdk-test.spec.ts
  });
});
