import { type Address, type Hash, type Hex } from "viem";
import { describe, expect, test } from "vitest";

import {
  getPeriodIdsForTransaction,
  LimitType,
  type SessionConfig,
  SessionErrorType,
  type SessionState,
  SessionStatus,
  validateSessionTransaction,
} from "./session.js";

describe("session utils", () => {
  // Common test session configuration
  const mockSessionConfig: SessionConfig = {
    signer: "0x1234567890123456789012345678901234567890" as Address,
    expiresAt: BigInt(Math.floor(Date.now() / 1000) + 86400), // expires in 1 day
    feeLimit: {
      limitType: LimitType.Allowance,
      limit: 100000000000000n,
      period: 3600n, // 1 hour
    },
    callPolicies: [
      {
        target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        selector: "0x12345678" as Hash,
        maxValuePerUse: 1000000000000n,
        valueLimit: {
          limitType: LimitType.Allowance,
          limit: 10000000000000n,
          period: 86400n, // 1 day
        },
        constraints: [
          {
            index: 4n,
            condition: 1, // Equal
            refValue: "0x0000000000000000000000000000000000000000000000000000000000000001" as Hash,
            limit: {
              limitType: LimitType.Lifetime,
              limit: 1000000000000n,
              period: 0n,
            },
          },
          {
            index: 36n,
            condition: 4, // GreaterEqual
            refValue: "0x0000000000000000000000000000000000000000000000000000000000000000" as Hash,
            limit: {
              limitType: LimitType.Allowance,
              limit: 5000000000000n,
              period: 43200n, // 12 hours
            },
          },
        ],
      },
    ],
    transferPolicies: [
      {
        target: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
        maxValuePerUse: 500000000000n,
        valueLimit: {
          limitType: LimitType.Allowance,
          limit: 5000000000000n,
          period: 604800n, // 1 week
        },
      },
    ],
  };

  // Sample active session state
  const mockActiveSessionState: SessionState = {
    status: SessionStatus.Active,
    feesRemaining: 90000000000000n, // 90% of the limit left
    transferValue: [
      {
        remaining: 4500000000000n, // 90% of the limit left
        target: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
        selector: "0x00000000" as Hash,
        index: 0n,
      },
    ],
    callValue: [
      {
        remaining: 9000000000000n, // 90% of the limit left
        target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        selector: "0x12345678" as Hash,
        index: 0n,
      },
    ],
    callParams: [
      {
        remaining: 900000000000n, // 90% of the first constraint limit left
        target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        selector: "0x12345678" as Hash,
        index: 4n,
      },
      {
        remaining: 4500000000000n, // 90% of the second constraint limit left
        target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        selector: "0x12345678" as Hash,
        index: 36n,
      },
    ],
  };

  describe("getPeriodIdsForTransaction", () => {
    test("returns correct period IDs for contract call", () => {
      const timestamp = 1714500000n; // Example fixed timestamp

      const periodIds = getPeriodIdsForTransaction({
        sessionConfig: mockSessionConfig,
        target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        selector: "0x12345678" as Hex,
        timestamp,
      });

      // Should return period IDs for fee limit, value limit, and both constraints
      expect(periodIds.length).toBe(4);

      // Check fee limit period ID (timestamp / hourly period)
      expect(periodIds[0]).toBe(timestamp / 3600n);

      // Check value limit period ID (timestamp / daily period)
      expect(periodIds[1]).toBe(timestamp / 86400n);

      // Check first constraint (lifetime, so period ID is 0)
      expect(periodIds[2]).toBe(0n);

      // Check second constraint (12 hour period)
      expect(periodIds[3]).toBe(timestamp / 43200n);
    });

    test("returns correct period IDs for transfer", () => {
      const timestamp = 1714500000n; // Example fixed timestamp

      const periodIds = getPeriodIdsForTransaction({
        sessionConfig: mockSessionConfig,
        target: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
        timestamp,
      });

      // Should return period IDs for fee limit and value limit only (no constraints for transfers)
      expect(periodIds.length).toBe(2);

      // Check fee limit period ID (timestamp / hourly period)
      expect(periodIds[0]).toBe(timestamp / 3600n);

      // Check value limit period ID (timestamp / weekly period)
      expect(periodIds[1]).toBe(timestamp / 604800n);
    });

    test("uses current timestamp if not provided", () => {
      const periodIds = getPeriodIdsForTransaction({
        sessionConfig: mockSessionConfig,
        target: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
      });

      // Should still return 2 period IDs for transfer
      expect(periodIds.length).toBe(2);

      // Period IDs should be calculated using current timestamp
      const now = BigInt(Math.floor(Date.now() / 1000));
      const expectedFeeId = now / 3600n;
      const expectedValueId = now / 604800n;

      // Allow a small margin for test execution time
      expect(periodIds[0]).toBeGreaterThanOrEqual(expectedFeeId - 1n);
      expect(periodIds[0]).toBeLessThanOrEqual(expectedFeeId + 1n);

      expect(periodIds[1]).toBe(expectedValueId);
    });

    test("throws if no matching policy is found", () => {
      expect(() => {
        getPeriodIdsForTransaction({
          sessionConfig: mockSessionConfig,
          target: "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC" as Address,
        });
      }).toThrow("Transaction does not fit any policy");

      expect(() => {
        getPeriodIdsForTransaction({
          sessionConfig: mockSessionConfig,
          target: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
          selector: "0x87654321" as Hex, // Different selector than the one in config
        });
      }).toThrow("Transaction does not fit any policy");
    });
  });

  describe("validateSessionTransaction", () => {
    test("validates a valid contract call transaction", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
          value: 500000000000n, // Half of maxValuePerUse
          data: "0x12345678000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001" as Hex,
          gas: 100000n,
          maxFeePerGas: 200000000n,
        },
      });

      expect(result.valid).toBe(true);
      expect(result.error).toBe(null);
    });

    test("validates a valid transfer transaction", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 300000000000n, // Less than maxValuePerUse
          gas: 21000n,
          maxFeePerGas: 200000000n,
        },
      });

      expect(result.valid).toBe(true);
      expect(result.error).toBe(null);
    });

    test("rejects transaction when session is not active", () => {
      const inactiveSessionState = {
        ...mockActiveSessionState,
        status: SessionStatus.Closed,
      };

      const result = validateSessionTransaction({
        sessionState: inactiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.SessionInactive);
    });

    test("rejects transaction when session has expired", () => {
      // Create a session config that has already expired
      const expiredSessionConfig = {
        ...mockSessionConfig,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) - 3600), // Expired 1 hour ago
      };

      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: expiredSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.SessionExpired);
    });

    test("rejects transaction when fee limit is exceeded", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
          gas: 1000000n,
          maxFeePerGas: 1000000000n, // Very high gas price, exceeding remaining fee limit
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.FeeLimitExceeded);
    });

    test("allows transaction with paymaster to bypass fee limit validation", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
          gas: 100000000000n,
          maxFeePerGas: 10000000000000n, // Very high gas price that would exceed fee limit
          paymaster: "0xdDdDddDdDdddDDddDDddDDDDdDdDDdDDdDDDDDDd" as Address,
        },
      });

      expect(result.valid).toBe(true);
      expect(result.error).toBe(null);
    });

    test("rejects transaction with no matching call policy", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
          data: "0x87654321" as Hex, // Selector not in policy
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.NoCallPolicy);
    });

    test("rejects transaction with no matching transfer policy", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC" as Address, // Address not in policy
          value: 100000000000n,
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.NoTransferPolicy);
    });

    test("rejects transaction when value exceeds maxValuePerUse", () => {
      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 600000000000n, // Exceeds maxValuePerUse of 500000000000n
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.MaxValuePerUseExceeded);
    });

    test("rejects transaction when value exceeds remaining limit", () => {
      // Create a session state with low remaining value
      const lowValueSessionState = {
        ...mockActiveSessionState,
        transferValue: [
          {
            remaining: 200000000000n, // Only 200000000000n left
            target: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
            selector: "0x00000000" as Hash,
            index: 0n,
          },
        ],
      };

      const result = validateSessionTransaction({
        sessionState: lowValueSessionState,
        sessionConfig: mockSessionConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 300000000000n, // Exceeds remaining value
        },
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.ValueLimitExceeded);
    });

    test("validates transaction with custom timestamp", () => {
      // Create a session config that expires at a specific time
      const specificExpiryConfig = {
        ...mockSessionConfig,
        expiresAt: 1714500000n, // Specific expiry timestamp
      };

      // Transaction at a timestamp before expiry
      const resultBefore = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: specificExpiryConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
        },
        currentTimestamp: 1714499999n, // 1 second before expiry
      });

      expect(resultBefore.valid).toBe(true);

      // Transaction at a timestamp after expiry
      const resultAfter = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: specificExpiryConfig,
        transaction: {
          to: "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB" as Address,
          value: 100000000000n,
        },
        currentTimestamp: 1714500001n, // 1 second after expiry
      });

      expect(resultAfter.valid).toBe(false);
      expect(resultAfter.error?.type).toBe(SessionErrorType.SessionExpired);
    });

    test("validates constraint conditions in contract calls", () => {
      // Create transaction with constraint violation
      const constraintViolationTransaction = {
        to: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa" as Address,
        value: 100000000000n,
        data: "0x12345678000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001" as Hex,
        // The constraint requires the value at index 4 to be equal to 0x01, but we have 0x02
      };

      const result = validateSessionTransaction({
        sessionState: mockActiveSessionState,
        sessionConfig: mockSessionConfig,
        transaction: constraintViolationTransaction,
      });

      expect(result.valid).toBe(false);
      expect(result.error?.type).toBe(SessionErrorType.ConstraintEqualViolated);
    });
  });
});
