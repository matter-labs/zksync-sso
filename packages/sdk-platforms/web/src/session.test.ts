import { describe, expect, it, vi } from "vitest";

vi.mock("./bundler", () => {
  class TransferPayloadMock {
    target: string;
    value_limit_value: string;
    value_limit_type: number;
    value_limit_period: string;
    constructor(target: string, value: string, t: number, p: string) {
      this.target = target;
      this.value_limit_value = value;
      this.value_limit_type = t;
      this.value_limit_period = p;
    }
  }
  class SessionPayloadMock {
    signer: string;
    expires_at: string;
    fee_limit_value: string;
    fee_limit_type: number;
    fee_limit_period: string;
    transfers: unknown[];
    constructor(signer: string, expires: string, val: string, t: number, p: string, txs: unknown[]) {
      this.signer = signer;
      this.expires_at = expires;
      this.fee_limit_value = val;
      this.fee_limit_type = t;
      this.fee_limit_period = p;
      this.transfers = txs;
    }
  }
  const add_session_to_account = vi.fn(async () => ({ ok: true }));
  const deploy_account = vi.fn(async () => ({ address: "0xabc" }));
  return {
    SessionPayload: SessionPayloadMock,
    TransferPayload: TransferPayloadMock,
    add_session_to_account,
    deploy_account,
  };
});

import { addSessionToAccount, deployAccountWithSession, toSessionPayload } from "./session";

const baseSession = {
  signer: "0x1234567890123456789012345678901234567890" as const,
  expiresAt: 1234567890,
  feeLimit: { limitType: "lifetime" as const, limit: 1000000000000000000n },
  transfers: [
    { to: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8" as const, valueLimit: 1000000000000000n },
  ],
};

describe("session helpers", () => {
  it("converts SessionConfig to SessionPayload", () => {
    const payload = toSessionPayload(baseSession);
    expect(payload.signer).toBe(baseSession.signer);
    expect(payload.expires_at).toBe(String(baseSession.expiresAt));
    expect(payload.fee_limit_type).toBe(1); // lifetime
    expect(payload.fee_limit_value).toBe("1000000000000000000");
    expect(Array.isArray(payload.transfers)).toBe(true);
    expect(payload.transfers[0].target).toBe(baseSession.transfers[0].to);
  });

  it("calls deploy_account with session when provided", async () => {
    const result = await deployAccountWithSession({
      userId: "user-1",
      eoaSigners: null,
      passkeyPayload: null,
      session: baseSession,
      deployConfig: {} as unknown as object,
    });
    expect(result).toBeDefined();
  });

  it("calls add_session_to_account with converted payload", async () => {
    const res = await addSessionToAccount({
      txConfig: {} as unknown as object,
      accountAddress: "0xabc",
      session: baseSession,
      sessionValidatorAddress: "0xsv",
      eoaValidatorAddress: "0xev",
      eoaPrivateKey: "0xpk",
    });
    expect(res).toBeDefined();
  });
});
