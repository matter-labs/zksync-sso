import { describe, expect, it } from "vitest";

// For Node tests, use the Node-targeted WASM entry to avoid bundler-target quirks
import * as wasm from "./node";

// A simple helper to build a minimal SessionSpec JSON
function buildSessionSpecJSON() {
  // Keep numbers as strings to be safe for bigints in serde (U256/U48)
  return JSON.stringify({
    signer: "0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9",
    expiresAt: "2088558400",
    feeLimit: {
      limitType: "Lifetime",
      limit: "1000000000000000000",
      period: "0",
    },
    callPolicies: [],
    transferPolicies: [
      {
        target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
        maxValuePerUse: "1",
        valueLimit: {
          limitType: "Unlimited",
          limit: "0",
          period: "0",
        },
      },
    ],
  });
}

describe("WASM session bindings", () => {
  it("encodes a session execute call", () => {
    const target = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
    const value = "1000";
    const data = "0x"; // empty calldata

    const encoded = wasm.encode_session_execute_call_data(target, value, data);
    expect(typeof encoded).toBe("string");
    expect(encoded.startsWith("0x")).toBe(true);
    expect(encoded.length).toBeGreaterThan(2);
  });

  it("generates a session stub signature", () => {
    const sessionValidator = "0x1234567890123456789012345678901234567890";
    const specJson = buildSessionSpecJSON();
    const stub = wasm.generate_session_stub_signature_wasm(
      sessionValidator,
      specJson,
      undefined,
    );

    expect(typeof stub).toBe("string");
    expect(stub.startsWith("0x")).toBe(true);
    // Must include the 20-byte validator prefix => > 40 hex chars + payload
    expect(stub.length).toBeGreaterThan(100);
  });

  it("creates a real session signature without time validation", () => {
    const privateKey
      = "0xb1da23908ba44fb1c6147ac1b32a1dbc6e7704ba94ec495e588d1e3cdc7ca6f9";
    const sessionValidator = "0x1234567890123456789012345678901234567890";
    const specJson = buildSessionSpecJSON();
    const hash = "0x" + "00".repeat(32); // 32-byte zero hash for deterministic test

    const sig = wasm.session_signature_no_validation_wasm(
      privateKey,
      sessionValidator,
      specJson,
      hash,
      undefined,
    );

    expect(typeof sig).toBe("string");
    expect(sig.startsWith("0x")).toBe(true);
    // Must include the 20-byte validator prefix => > 40 hex chars + payload
    expect(sig.length).toBeGreaterThan(100);
  });

  it("computes keyed nonce from session signer address", () => {
    const signer = "0xCEbb58e4082Af6FaC6Ea275740f10073d1610ad9";
    const nonceDec = wasm.keyed_nonce_decimal(signer);

    expect(typeof nonceDec).toBe("string");
    // Should be a decimal string (no 0x prefix)
    expect(nonceDec.startsWith("0x")).toBe(false);
    // Non-empty and numeric
    expect(nonceDec.length).toBeGreaterThan(0);
    expect(/^[0-9]+$/.test(nonceDec)).toBe(true);
  });
});
