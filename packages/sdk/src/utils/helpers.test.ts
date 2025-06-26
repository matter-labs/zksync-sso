import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { calculateMaxFee, findSmallestBigInt, getFavicon, getWebsiteName, noThrow } from "./helpers.js";

describe("helpers", () => {
  describe("findSmallestBigInt", () => {
    test("returns the smallest bigint in an array", () => {
      expect(findSmallestBigInt([10n, 5n, 20n])).toBe(5n);
      expect(findSmallestBigInt([100n, 200n, 50n])).toBe(50n);
      expect(findSmallestBigInt([-10n, 5n, 20n])).toBe(-10n);
      expect(findSmallestBigInt([0n, -5n, -10n])).toBe(-10n);
    });

    test("returns the only element if array has only one element", () => {
      expect(findSmallestBigInt([42n])).toBe(42n);
    });

    test("throws error if array is empty", () => {
      expect(() => findSmallestBigInt([])).toThrow("Array must not be empty");
    });
  });

  describe("calculateMaxFee", () => {
    test("returns 0 if gas is not provided", () => {
      expect(calculateMaxFee({})).toBe(0n);
      expect(calculateMaxFee({ maxFeePerGas: 100n })).toBe(0n);
      expect(calculateMaxFee({ gasPrice: 100n })).toBe(0n);
      expect(calculateMaxFee({ maxPriorityFeePerGas: 100n })).toBe(0n);
    });

    test("calculates fee correctly with gasPrice", () => {
      expect(calculateMaxFee({ gas: 100000n, gasPrice: 50n })).toBe(5000000n);
    });

    test("calculates fee correctly with maxFeePerGas", () => {
      expect(calculateMaxFee({ gas: 100000n, maxFeePerGas: 60n })).toBe(6000000n);
    });

    test("calculates fee correctly with maxPriorityFeePerGas", () => {
      expect(calculateMaxFee({ gas: 100000n, maxPriorityFeePerGas: 40n })).toBe(4000000n);
    });

    test("prioritizes gasPrice over other gas parameters", () => {
      expect(calculateMaxFee({
        gas: 100000n,
        gasPrice: 50n,
        maxFeePerGas: 60n,
        maxPriorityFeePerGas: 40n,
      })).toBe(5000000n);
    });

    test("prioritizes maxFeePerGas over maxPriorityFeePerGas", () => {
      expect(calculateMaxFee({
        gas: 100000n,
        maxFeePerGas: 60n,
        maxPriorityFeePerGas: 40n,
      })).toBe(6000000n);
    });
  });

  describe("noThrow", () => {
    test("returns result of function if it doesn't throw", () => {
      expect(noThrow(() => 42)).toBe(42);
      expect(noThrow(() => "hello")).toBe("hello");
      expect(noThrow(() => ({ value: true }))).toEqual({ value: true });
    });

    test("returns null if function throws", () => {
      expect(noThrow(() => {
        throw new Error("Test error");
      })).toBe(null);
      expect(noThrow(() => {
        throw "string error";
      })).toBe(null);
    });
  });

  // For DOM-dependent functions, we need to mock the document
  describe("getWebsiteName", () => {
    const originalDocument = global.document;

    beforeEach(() => {
      // Create a mock document object
      global.document = {
        title: "",
      } as any;
    });

    afterEach(() => {
      // Restore the original document
      global.document = originalDocument;
    });

    test("returns null if document title is empty", () => {
      global.document.title = "";
      expect(getWebsiteName()).toBe(null);
    });

    test("extracts website name using delimiter - ", () => {
      global.document.title = "Website Name - Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("extracts website name using delimiter | ", () => {
      global.document.title = "Website Name | Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("extracts website name using delimiter : ", () => {
      global.document.title = "Website Name : Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("extracts website name using delimiter · ", () => {
      global.document.title = "Website Name · Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("extracts website name using delimiter — ", () => {
      global.document.title = "Website Name — Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("returns full title if no delimiter is found", () => {
      global.document.title = "Website Name";
      expect(getWebsiteName()).toBe("Website Name");
    });

    test("trims whitespace from extracted name", () => {
      global.document.title = "  Website Name  | Page Title";
      expect(getWebsiteName()).toBe("Website Name");
    });
  });

  describe("getFavicon", () => {
    const originalDocument = global.document;

    beforeEach(() => {
      // Create a mock document object
      global.document = {
        location: { href: "https://example.com" },
        querySelector: vi.fn(),
      } as any;
    });

    afterEach(() => {
      // Restore the original document
      global.document = originalDocument;
    });

    test("returns null if no favicon element is found", () => {
      (global.document.querySelector as any).mockReturnValue(null);
      expect(getFavicon()).toBe(null);
    });

    test("returns null if favicon element has no href", () => {
      (global.document.querySelector as any).mockReturnValue({
        getAttribute: () => null,
      });
      expect(getFavicon()).toBe(null);
    });

    test("returns absolute URL for absolute favicon path", () => {
      (global.document.querySelector as any).mockReturnValue({
        getAttribute: () => "https://cdn.example.com/favicon.ico",
      });
      expect(getFavicon()).toBe("https://cdn.example.com/favicon.ico");
    });

    test("resolves relative URLs against document location", () => {
      (global.document.querySelector as any).mockReturnValue({
        getAttribute: () => "relative-path/favicon.ico",
      });
      expect(getFavicon()).toBe("https://example.com/relative-path/favicon.ico");
    });

    test("returns null for non-http/https protocols", () => {
      (global.document.querySelector as any).mockReturnValue({
        getAttribute: () => "javascript:alert('xss')",
      });
      expect(getFavicon()).toBe(null);
    });
  });
});
