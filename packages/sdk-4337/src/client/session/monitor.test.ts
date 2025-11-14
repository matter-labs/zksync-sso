import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SessionState } from "../actions/getSessionState.js";
import { SessionStatus } from "../actions/getSessionState.js";
import { shouldWarnAboutSession } from "./monitor.js";
import type { SessionSpec } from "./types.js";
import { LimitType } from "./types.js";

describe("Session Monitoring", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  describe("shouldWarnAboutSession", () => {
    const mockSessionSpec: SessionSpec = {
      signer: "0x1234567890123456789012345678901234567890" as `0x${string}`,
      expiresAt: BigInt(Math.floor(Date.now() / 1000) + 7200), // 2 hours from now
      feeLimit: {
        limitType: LimitType.Lifetime,
        limit: 1000000n,
        period: 0n,
      },
      callPolicies: [],
      transferPolicies: [],
    };

    const mockSessionState: SessionState = {
      status: SessionStatus.Active,
      feesRemaining: 500000n,
      transferValue: [],
      callValue: [],
      callParams: [],
    };

    it("should not warn for healthy session", () => {
      const result = shouldWarnAboutSession(mockSessionSpec, mockSessionState);

      expect(result.shouldWarn).toBe(false);
      expect(result.reason).toBeUndefined();
    });

    it("should warn when session is expiring soon", () => {
      const expiringSession: SessionSpec = {
        ...mockSessionSpec,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 1800), // 30 minutes from now
      };

      const result = shouldWarnAboutSession(expiringSession, mockSessionState, {
        expirationWarningThresholdSeconds: 3600,
      });

      expect(result.shouldWarn).toBe(true);
      expect(result.reason).toContain("expire");
    });

    it("should warn when fee limit is nearly exhausted", () => {
      const lowFeesState: SessionState = {
        ...mockSessionState,
        feesRemaining: 100000n, // 10% remaining
      };

      const result = shouldWarnAboutSession(mockSessionSpec, lowFeesState, {
        feeLimitWarningThresholdPercent: 20,
      });

      expect(result.shouldWarn).toBe(true);
      expect(result.reason).toContain("Fee limit");
    });

    it("should warn when fee limit is at threshold", () => {
      const expiringSession: SessionSpec = {
        ...mockSessionSpec,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 1800),
      };

      const lowFeesState: SessionState = {
        ...mockSessionState,
        feesRemaining: 100000n,
      };

      const expirationResult = shouldWarnAboutSession(
        expiringSession,
        mockSessionState,
        {
          expirationWarningThresholdSeconds: 3600,
        },
      );

      const feeResult = shouldWarnAboutSession(mockSessionSpec, lowFeesState, {
        feeLimitWarningThresholdPercent: 20,
      });

      expect(expirationResult.shouldWarn).toBe(true);
      expect(feeResult.shouldWarn).toBe(true);
    });

    it("should not warn for unlimited fee limit", () => {
      const unlimitedSession: SessionSpec = {
        ...mockSessionSpec,
        feeLimit: {
          limitType: LimitType.Unlimited,
          limit: 0n,
          period: 0n,
        },
      };

      const result = shouldWarnAboutSession(unlimitedSession, mockSessionState);

      expect(result.shouldWarn).toBe(false);
    });

    it("should use custom warning thresholds", () => {
      const expiringSession: SessionSpec = {
        ...mockSessionSpec,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 5400), // 1.5 hours from now
      };

      // With default threshold (1 hour), should not warn
      const defaultResult = shouldWarnAboutSession(
        expiringSession,
        mockSessionState,
      );
      expect(defaultResult.shouldWarn).toBe(false);

      // With custom threshold (2 hours), should warn
      const customResult = shouldWarnAboutSession(
        expiringSession,
        mockSessionState,
        { expirationWarningThresholdSeconds: 7200 },
      );
      expect(customResult.shouldWarn).toBe(true);
    });

    it("should warn when session is already expired", () => {
      const expiredSession: SessionSpec = {
        ...mockSessionSpec,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) - 100), // Expired 100 seconds ago
      };

      const result = shouldWarnAboutSession(expiredSession, mockSessionState);

      expect(result.shouldWarn).toBe(false); // Already expired, past warning threshold
    });

    it("should warn when fee limit is completely exhausted", () => {
      const exhaustedState: SessionState = {
        ...mockSessionState,
        feesRemaining: 0n,
      };

      const result = shouldWarnAboutSession(mockSessionSpec, exhaustedState);

      expect(result.shouldWarn).toBe(true);
      expect(result.reason).toContain("100% exhausted");
    });
  });
});
