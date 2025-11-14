import { describe, expect, it } from "vitest";

import {
  type LegacyCallPolicy,
  legacyCallPolicyToCallPolicy,
  type LegacyConstraint,
  legacyConstraintToConstraint,
  type LegacyLimit,
  legacyLimitToUsageLimit,
  type LegacySessionConfig,
  legacySessionConfigToSpec,
  type LegacyTransferPolicy,
  legacyTransferPolicyToTransferPolicy,
} from "./compat.js";
import { ConstraintCondition, LimitType } from "./types.js";

describe("Compatibility Layer", () => {
  describe("legacyLimitToUsageLimit", () => {
    it("should convert legacy Unlimited limit", () => {
      const legacyLimit: LegacyLimit = {
        limitType: LimitType.Unlimited,
        limit: 0n,
        period: 0n,
      };

      const result = legacyLimitToUsageLimit(legacyLimit);

      expect(result.limitType).toBe(LimitType.Unlimited);
      expect(result.limit).toBe(0n);
      expect(result.period).toBe(0n);
    });

    it("should convert legacy Lifetime limit", () => {
      const legacyLimit: LegacyLimit = {
        limitType: LimitType.Lifetime,
        limit: 1000n,
        period: 0n,
      };

      const result = legacyLimitToUsageLimit(legacyLimit);

      expect(result.limitType).toBe(LimitType.Lifetime);
      expect(result.limit).toBe(1000n);
      expect(result.period).toBe(0n);
    });

    it("should convert legacy Allowance limit", () => {
      const legacyLimit: LegacyLimit = {
        limitType: LimitType.Allowance,
        limit: 5000n,
        period: 86400n,
      };

      const result = legacyLimitToUsageLimit(legacyLimit);

      expect(result.limitType).toBe(LimitType.Allowance);
      expect(result.limit).toBe(5000n);
      expect(result.period).toBe(86400n);
    });
  });

  describe("legacyConstraintToConstraint", () => {
    it("should convert constraint with Equal condition", () => {
      const legacyConstraint: LegacyConstraint = {
        condition: ConstraintCondition.Equal,
        index: 4n,
        refValue: `0x${"00".repeat(32)}` as `0x${string}`,
        limit: {
          limitType: LimitType.Unlimited,
          limit: 0n,
          period: 0n,
        },
      };

      const result = legacyConstraintToConstraint(legacyConstraint);

      expect(result.condition).toBe(ConstraintCondition.Equal);
      expect(result.index).toBe(4n);
      expect(result.refValue).toBe(legacyConstraint.refValue);
      expect(result.limit.limitType).toBe(LimitType.Unlimited);
    });

    it("should convert constraint with Greater condition", () => {
      const legacyConstraint: LegacyConstraint = {
        condition: ConstraintCondition.Greater,
        index: 8n,
        refValue: `0x${"ff".repeat(32)}` as `0x${string}`,
        limit: {
          limitType: LimitType.Lifetime,
          limit: 1000n,
          period: 0n,
        },
      };

      const result = legacyConstraintToConstraint(legacyConstraint);

      expect(result.condition).toBe(ConstraintCondition.Greater);
      expect(result.index).toBe(8n);
      expect(result.limit.limit).toBe(1000n);
    });
  });

  describe("legacyCallPolicyToCallPolicy", () => {
    it("should truncate 32-byte selector to 4 bytes", () => {
      const legacyPolicy: LegacyCallPolicy = {
        target: "0x1234567890123456789012345678901234567890" as `0x${string}`,
        selector: (`0x${"a9059cbb" + "00".repeat(28)}`) as `0x${string}`, // 32 bytes
        maxValuePerUse: 100n,
        valueLimit: {
          limitType: LimitType.Lifetime,
          limit: 1000n,
          period: 0n,
        },
        constraints: [],
      };

      const result = legacyCallPolicyToCallPolicy(legacyPolicy);

      expect(result.selector).toBe("0xa9059cbb"); // Truncated to 4 bytes
      expect(result.selector.length).toBe(10); // 0x + 8 hex chars
      expect(result.target).toBe(legacyPolicy.target);
      expect(result.maxValuePerUse).toBe(100n);
    });

    it("should handle already 4-byte selectors", () => {
      const legacyPolicy: LegacyCallPolicy = {
        target: "0x1234567890123456789012345678901234567890" as `0x${string}`,
        selector: "0xa9059cbb" as `0x${string}`, // Already 4 bytes
        maxValuePerUse: 100n,
        valueLimit: {
          limitType: LimitType.Lifetime,
          limit: 1000n,
          period: 0n,
        },
        constraints: [],
      };

      const result = legacyCallPolicyToCallPolicy(legacyPolicy);

      expect(result.selector).toBe("0xa9059cbb");
    });

    it("should convert call policy with constraints", () => {
      const legacyPolicy: LegacyCallPolicy = {
        target: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd" as `0x${string}`,
        selector: "0x70a08231" as `0x${string}`, // balanceOf
        maxValuePerUse: 0n,
        valueLimit: {
          limitType: LimitType.Unlimited,
          limit: 0n,
          period: 0n,
        },
        constraints: [
          {
            condition: ConstraintCondition.LessEqual,
            index: 4n,
            refValue: `0x${"00".repeat(31)}64` as `0x${string}`, // 100
            limit: {
              limitType: LimitType.Lifetime,
              limit: 10n,
              period: 0n,
            },
          },
        ],
      };

      const result = legacyCallPolicyToCallPolicy(legacyPolicy);

      expect(result.constraints).toHaveLength(1);
      expect(result.constraints[0].condition).toBe(ConstraintCondition.LessEqual);
      expect(result.constraints[0].index).toBe(4n);
    });
  });

  describe("legacyTransferPolicyToTransferPolicy", () => {
    it("should convert transfer policy with Unlimited limit", () => {
      const legacyPolicy: LegacyTransferPolicy = {
        target: "0x1234567890123456789012345678901234567890" as `0x${string}`,
        maxValuePerUse: 100n,
        valueLimit: {
          limitType: LimitType.Unlimited,
          limit: 0n,
          period: 0n,
        },
      };

      const result = legacyTransferPolicyToTransferPolicy(legacyPolicy);

      expect(result.target).toBe(legacyPolicy.target);
      expect(result.maxValuePerUse).toBe(100n);
      expect(result.valueLimit.limitType).toBe(LimitType.Unlimited);
    });

    it("should convert transfer policy with Allowance limit", () => {
      const legacyPolicy: LegacyTransferPolicy = {
        target: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd" as `0x${string}`,
        maxValuePerUse: 1000n,
        valueLimit: {
          limitType: LimitType.Allowance,
          limit: 10000n,
          period: 86400n, // Daily
        },
      };

      const result = legacyTransferPolicyToTransferPolicy(legacyPolicy);

      expect(result.target).toBe(legacyPolicy.target);
      expect(result.maxValuePerUse).toBe(1000n);
      expect(result.valueLimit.limitType).toBe(LimitType.Allowance);
      expect(result.valueLimit.period).toBe(86400n);
    });
  });

  describe("legacySessionConfigToSpec", () => {
    it("should convert minimal legacy SessionConfig", () => {
      const legacyConfig: LegacySessionConfig = {
        signer: "0x1234567890123456789012345678901234567890" as `0x${string}`,
        expiresAt: 1735689600n,
        feeLimit: {
          limitType: LimitType.Unlimited,
          limit: 0n,
          period: 0n,
        },
        callPolicies: [],
        transferPolicies: [],
      };

      const result = legacySessionConfigToSpec(legacyConfig);

      expect(result.signer).toBe(legacyConfig.signer);
      expect(result.expiresAt).toBe(legacyConfig.expiresAt);
      expect(result.feeLimit.limitType).toBe(LimitType.Unlimited);
      expect(result.callPolicies).toHaveLength(0);
      expect(result.transferPolicies).toHaveLength(0);
    });

    it("should convert complete legacy SessionConfig with policies", () => {
      const legacyConfig: LegacySessionConfig = {
        signer: "0x1234567890123456789012345678901234567890" as `0x${string}`,
        expiresAt: 1735689600n,
        feeLimit: {
          limitType: LimitType.Lifetime,
          limit: 1000000000000000n,
          period: 0n,
        },
        callPolicies: [
          {
            target: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd" as `0x${string}`,
            selector: "0xa9059cbb" as `0x${string}`,
            maxValuePerUse: 100n,
            valueLimit: {
              limitType: LimitType.Unlimited,
              limit: 0n,
              period: 0n,
            },
            constraints: [],
          },
        ],
        transferPolicies: [
          {
            target: "0x9876543210987654321098765432109876543210" as `0x${string}`,
            maxValuePerUse: 50n,
            valueLimit: {
              limitType: LimitType.Allowance,
              limit: 500n,
              period: 3600n,
            },
          },
        ],
      };

      const result = legacySessionConfigToSpec(legacyConfig);

      expect(result.signer).toBe(legacyConfig.signer);
      expect(result.expiresAt).toBe(legacyConfig.expiresAt);
      expect(result.feeLimit.limitType).toBe(LimitType.Lifetime);
      expect(result.feeLimit.limit).toBe(1000000000000000n);
      expect(result.callPolicies).toHaveLength(1);
      expect(result.callPolicies[0].selector).toBe("0xa9059cbb");
      expect(result.callPolicies[0].target).toBe(legacyConfig.callPolicies[0].target);
      expect(result.transferPolicies).toHaveLength(1);
      expect(result.transferPolicies[0].maxValuePerUse).toBe(50n);
      expect(result.transferPolicies[0].valueLimit.period).toBe(3600n);
    });

    it("should handle complex SessionConfig with multiple policies and constraints", () => {
      const legacyConfig: LegacySessionConfig = {
        signer: "0x1111111111111111111111111111111111111111" as `0x${string}`,
        expiresAt: 1800000000n,
        feeLimit: {
          limitType: LimitType.Allowance,
          limit: 50000n,
          period: 86400n,
        },
        callPolicies: [
          {
            target: "0x2222222222222222222222222222222222222222" as `0x${string}`,
            selector: "0x095ea7b3" as `0x${string}`, // approve
            maxValuePerUse: 0n,
            valueLimit: {
              limitType: LimitType.Unlimited,
              limit: 0n,
              period: 0n,
            },
            constraints: [
              {
                condition: ConstraintCondition.LessEqual,
                index: 36n,
                refValue: `0x${" 00".repeat(30)}03e8` as `0x${string}`, // 1000
                limit: {
                  limitType: LimitType.Lifetime,
                  limit: 5n,
                  period: 0n,
                },
              },
            ],
          },
          {
            target: "0x3333333333333333333333333333333333333333" as `0x${string}`,
            selector: "0xa9059cbb" as `0x${string}`, // transfer
            maxValuePerUse: 100n,
            valueLimit: {
              limitType: LimitType.Lifetime,
              limit: 1000n,
              period: 0n,
            },
            constraints: [],
          },
        ],
        transferPolicies: [
          {
            target: "0x4444444444444444444444444444444444444444" as `0x${string}`,
            maxValuePerUse: 10n,
            valueLimit: {
              limitType: LimitType.Lifetime,
              limit: 100n,
              period: 0n,
            },
          },
        ],
      };

      const result = legacySessionConfigToSpec(legacyConfig);

      expect(result.callPolicies).toHaveLength(2);
      expect(result.callPolicies[0].constraints).toHaveLength(1);
      expect(result.callPolicies[1].constraints).toHaveLength(0);
      expect(result.transferPolicies).toHaveLength(1);
      expect(result.feeLimit.limitType).toBe(LimitType.Allowance);
    });
  });
});
