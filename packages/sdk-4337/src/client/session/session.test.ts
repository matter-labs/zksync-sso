import type { Address, Hex } from "viem";
import { describe, expect, it } from "vitest";

import {
  type CallPolicy,
  ConstraintCondition,
  LimitType,
  type SessionSpec,
  type TransferPolicy,
  type UsageLimit,
} from "./types.js";
import {
  extractSelector,
  findMatchingPolicy,
  isSessionExpired,
  sessionSpecToJSON,
  validateTransactionAgainstSession,
} from "./utils.js";

// Helper to create a basic usage limit
function createUnlimitedLimit(): UsageLimit {
  return {
    limitType: LimitType.Unlimited,
    limit: 0n,
    period: 0n,
  };
}

function createLifetimeLimit(limit: bigint): UsageLimit {
  return {
    limitType: LimitType.Lifetime,
    limit,
    period: 0n,
  };
}

// Helper to create a basic session spec
function createBasicSessionSpec(
  signer: Address,
  expiresAt: bigint = BigInt(Math.floor(Date.now() / 1000) + 3600), // 1 hour from now
): SessionSpec {
  return {
    signer,
    expiresAt,
    feeLimit: createUnlimitedLimit(),
    callPolicies: [],
    transferPolicies: [],
  };
}

describe("Session Types", () => {
  it("should create a session spec with proper types", () => {
    const signer: Address = "0x1234567890123456789012345678901234567890";
    const spec = createBasicSessionSpec(signer);

    expect(spec.signer).toBe(signer);
    expect(typeof spec.expiresAt).toBe("bigint");
    expect(spec.feeLimit.limitType).toBe(LimitType.Unlimited);
    expect(Array.isArray(spec.callPolicies)).toBe(true);
    expect(Array.isArray(spec.transferPolicies)).toBe(true);
  });

  it("should support different limit types", () => {
    const unlimited = createUnlimitedLimit();
    expect(unlimited.limitType).toBe(LimitType.Unlimited);
    expect(unlimited.limit).toBe(0n);

    const lifetime = createLifetimeLimit(1000000000000000000n); // 1 ETH
    expect(lifetime.limitType).toBe(LimitType.Lifetime);
    expect(lifetime.limit).toBe(1000000000000000000n);
  });

  it("should create transfer policy", () => {
    const policy: TransferPolicy = {
      target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
      maxValuePerUse: 1000000000000000n, // 0.001 ETH
      valueLimit: createLifetimeLimit(10000000000000000n), // 0.01 ETH total
    };

    expect(policy.target).toBeDefined();
    expect(typeof policy.maxValuePerUse).toBe("bigint");
    expect(policy.valueLimit.limitType).toBe(LimitType.Lifetime);
  });

  it("should create call policy with constraints", () => {
    const policy: CallPolicy = {
      target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
      selector: "0xa9059cbb" as Hex, // transfer(address,uint256)
      maxValuePerUse: 0n,
      valueLimit: createUnlimitedLimit(),
      constraints: [
        {
          condition: ConstraintCondition.Equal,
          index: 0n, // First parameter
          refValue:
            "0x0000000000000000000000003C44CdDdB6a900fa2b585dd299e03d12FA4293BC" as Hex,
          limit: createUnlimitedLimit(),
        },
      ],
    };

    expect(policy.selector).toBe("0xa9059cbb");
    expect(policy.constraints.length).toBe(1);
    expect(policy.constraints[0].condition).toBe(ConstraintCondition.Equal);
  });
});

describe("Session Utils", () => {
  describe("sessionSpecToJSON", () => {
    it("should convert SessionSpec to JSON string", () => {
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const spec = createBasicSessionSpec(signer);
      const json = sessionSpecToJSON(spec);

      expect(typeof json).toBe("string");
      const parsed = JSON.parse(json);
      expect(parsed.signer).toBe(signer);
      expect(typeof parsed.expiresAt).toBe("string"); // bigints as strings
      expect(parsed.feeLimit.limitType).toBe(0); // LimitType.Unlimited = 0
    });

    it("should handle complex session spec with policies", () => {
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const spec: SessionSpec = {
        signer,
        expiresAt: 2088558400n,
        feeLimit: createLifetimeLimit(1000000000000000000n),
        callPolicies: [
          {
            target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
            selector: "0xa9059cbb" as Hex,
            maxValuePerUse: 0n,
            valueLimit: createUnlimitedLimit(),
            constraints: [],
          },
        ],
        transferPolicies: [
          {
            target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
            maxValuePerUse: 1000000n,
            valueLimit: createLifetimeLimit(10000000n),
          },
        ],
      };

      const json = sessionSpecToJSON(spec);
      const parsed = JSON.parse(json);

      expect(parsed.callPolicies).toHaveLength(1);
      expect(parsed.transferPolicies).toHaveLength(1);
      expect(parsed.feeLimit.limitType).toBe(1); // LimitType.Lifetime = 1
      expect(parsed.feeLimit.limit).toBe("1000000000000000000");
    });
  });

  describe("extractSelector", () => {
    it("should extract function selector from calldata", () => {
      const calldata: Hex
        = "0xa9059cbb0000000000000000000000003C44CdDdB6a900fa2b585dd299e03d12FA4293BC0000000000000000000000000000000000000000000000000000000000000001";
      const selector = extractSelector(calldata);

      expect(selector).toBe("0xa9059cbb");
    });

    it("should handle short calldata", () => {
      const calldata: Hex = "0x12";
      const selector = extractSelector(calldata);

      expect(selector).toBeUndefined();
    });

    it("should handle empty calldata", () => {
      const calldata: Hex = "0x";
      const selector = extractSelector(calldata);

      expect(selector).toBeUndefined();
    });
  });

  describe("findMatchingPolicy", () => {
    it("should find matching call policy", () => {
      const target: Address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
      const selector: Hex = "0xa9059cbb";
      const signer: Address = "0x1234567890123456789012345678901234567890";

      const spec: SessionSpec = {
        signer,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
        feeLimit: createUnlimitedLimit(),
        callPolicies: [
          {
            target,
            selector,
            maxValuePerUse: 0n,
            valueLimit: createUnlimitedLimit(),
            constraints: [],
          },
          {
            target: "0x0000000000000000000000000000000000000000",
            selector: "0x12345678" as Hex,
            maxValuePerUse: 0n,
            valueLimit: createUnlimitedLimit(),
            constraints: [],
          },
        ],
        transferPolicies: [],
      };

      const match = findMatchingPolicy(spec, target, selector);

      expect(match).toBeDefined();
      expect(match).not.toBeNull();
      expect(match?.type).toBe("call");
      if (match?.type === "call" && "selector" in match.policy) {
        expect(match.policy.target).toBe(target);
        expect(match.policy.selector).toBe(selector);
      }
    });

    it("should return null for non-matching policy", () => {
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const spec: SessionSpec = {
        signer,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
        feeLimit: createUnlimitedLimit(),
        callPolicies: [
          {
            target: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
            selector: "0xa9059cbb" as Hex,
            maxValuePerUse: 0n,
            valueLimit: createUnlimitedLimit(),
            constraints: [],
          },
        ],
        transferPolicies: [],
      };

      const match = findMatchingPolicy(
        spec,
        "0x0000000000000000000000000000000000000000",
        "0x12345678" as Hex,
      );

      expect(match).toBeNull();
    });
  });

  describe("isSessionExpired", () => {
    it("should return false for future expiry", () => {
      const futureTimestamp = BigInt(Math.floor(Date.now() / 1000) + 3600); // 1 hour from now
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const spec = createBasicSessionSpec(signer, futureTimestamp);

      expect(isSessionExpired(spec)).toBe(false);
    });

    it("should return true for past expiry", () => {
      const pastTimestamp = BigInt(Math.floor(Date.now() / 1000) - 3600); // 1 hour ago
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const spec = createBasicSessionSpec(signer, pastTimestamp);

      expect(isSessionExpired(spec)).toBe(true);
    });
  });

  describe("validateTransactionAgainstSession", () => {
    it("should validate simple transfer against transfer policy", () => {
      const target: Address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
      const signer: Address = "0x1234567890123456789012345678901234567890";

      const spec: SessionSpec = {
        signer,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
        feeLimit: createUnlimitedLimit(),
        callPolicies: [],
        transferPolicies: [
          {
            target,
            maxValuePerUse: 1000000000000000n, // 0.001 ETH
            valueLimit: createLifetimeLimit(10000000000000000n), // 0.01 ETH
          },
        ],
      };

      // Should pass: transfer within limits
      const result1 = validateTransactionAgainstSession(
        spec,
        target,
        500000000000000n, // 0.0005 ETH
        "0x",
      );
      expect(result1).toBeNull(); // null means valid

      // Should fail: exceeds maxValuePerUse
      const result2 = validateTransactionAgainstSession(
        spec,
        target,
        2000000000000000n, // 0.002 ETH
        "0x",
      );
      expect(result2).not.toBeNull(); // string error message
      expect(result2).toContain("maxValuePerUse");
    });

    it("should validate function call against call policy", () => {
      const target: Address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
      const signer: Address = "0x1234567890123456789012345678901234567890";
      const selector: Hex = "0xa9059cbb"; // transfer(address,uint256)

      const spec: SessionSpec = {
        signer,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
        feeLimit: createUnlimitedLimit(),
        callPolicies: [
          {
            target,
            selector,
            maxValuePerUse: 0n,
            valueLimit: createUnlimitedLimit(),
            constraints: [],
          },
        ],
        transferPolicies: [],
      };

      const calldata: Hex = `${selector}0000000000000000000000003C44CdDdB6a900fa2b585dd299e03d12FA4293BC0000000000000000000000000000000000000000000000000000000000000001`;

      const result = validateTransactionAgainstSession(
        spec,
        target,
        0n,
        calldata,
      );
      expect(result).toBeNull(); // null means valid
    });

    it("should reject transaction to unpermitted target", () => {
      const allowedTarget: Address
        = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
      const forbiddenTarget: Address
        = "0x0000000000000000000000000000000000000000";
      const signer: Address = "0x1234567890123456789012345678901234567890";

      const spec: SessionSpec = {
        signer,
        expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
        feeLimit: createUnlimitedLimit(),
        callPolicies: [],
        transferPolicies: [
          {
            target: allowedTarget,
            maxValuePerUse: 1000000000000000n,
            valueLimit: createUnlimitedLimit(),
          },
        ],
      };

      const result = validateTransactionAgainstSession(
        spec,
        forbiddenTarget,
        1n,
        "0x",
      );
      expect(result).not.toBeNull();
      expect(result).toContain("policy");
    });
  });
});

describe("Session Integration Scenarios", () => {
  it("should create a session for ERC20 token transfers", () => {
    const sessionSigner: Address
      = "0x1234567890123456789012345678901234567890";
    const tokenAddress: Address
      = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
    const recipient: Address = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";

    // Create a session that allows ERC20 transfers to a specific recipient
    const spec: SessionSpec = {
      signer: sessionSigner,
      expiresAt: BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours
      feeLimit: createLifetimeLimit(1000000000000000n), // 0.001 ETH for gas
      callPolicies: [
        {
          target: tokenAddress,
          selector: "0xa9059cbb" as Hex, // transfer(address,uint256)
          maxValuePerUse: 0n, // No ETH value in token transfers
          valueLimit: createUnlimitedLimit(),
          constraints: [
            {
              // Constrain first parameter (recipient) to specific address
              condition: ConstraintCondition.Equal,
              index: 0n,
              refValue: recipient.toLowerCase().padEnd(66, "0") as Hex, // Pad to 32 bytes
              limit: createUnlimitedLimit(),
            },
          ],
        },
      ],
      transferPolicies: [],
    };

    expect(spec.callPolicies).toHaveLength(1);
    expect(spec.callPolicies[0].constraints).toHaveLength(1);

    const json = sessionSpecToJSON(spec);
    expect(json).toBeTruthy();
  });

  it("should create a session for limited ETH transfers", () => {
    const sessionSigner: Address
      = "0x1234567890123456789012345678901234567890";
    const allowedRecipient: Address
      = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";

    // Create a session that allows limited ETH transfers
    const spec: SessionSpec = {
      signer: sessionSigner,
      expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600), // 1 hour
      feeLimit: createLifetimeLimit(100000000000000n), // 0.0001 ETH for gas
      callPolicies: [],
      transferPolicies: [
        {
          target: allowedRecipient,
          maxValuePerUse: 10000000000000000n, // 0.01 ETH per transfer
          valueLimit: createLifetimeLimit(100000000000000000n), // 0.1 ETH total
        },
      ],
    };

    // Validate transfers within limits
    const validTransfer = validateTransactionAgainstSession(
      spec,
      allowedRecipient,
      5000000000000000n, // 0.005 ETH
      "0x",
    );
    expect(validTransfer).toBeNull(); // null means valid

    // Validate transfer exceeding per-use limit
    const invalidTransfer = validateTransactionAgainstSession(
      spec,
      allowedRecipient,
      20000000000000000n, // 0.02 ETH
      "0x",
    );
    expect(invalidTransfer).not.toBeNull(); // error message
  });

  it("should create a comprehensive session with multiple policies", () => {
    const sessionSigner: Address
      = "0x1234567890123456789012345678901234567890";
    const usdcAddress: Address = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720";
    const daiAddress: Address = "0x0000000000000000000000000000000000000001";
    const recipient: Address = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";

    const spec: SessionSpec = {
      signer: sessionSigner,
      expiresAt: BigInt(Math.floor(Date.now() / 1000) + 86400), // 24 hours
      feeLimit: createLifetimeLimit(5000000000000000n), // 0.005 ETH for gas
      callPolicies: [
        {
          // Allow USDC transfers
          target: usdcAddress,
          selector: "0xa9059cbb" as Hex,
          maxValuePerUse: 0n,
          valueLimit: createUnlimitedLimit(),
          constraints: [],
        },
        {
          // Allow DAI approvals
          target: daiAddress,
          selector: "0x095ea7b3" as Hex, // approve(address,uint256)
          maxValuePerUse: 0n,
          valueLimit: createUnlimitedLimit(),
          constraints: [],
        },
      ],
      transferPolicies: [
        {
          // Allow ETH transfers to specific recipient
          target: recipient,
          maxValuePerUse: 1000000000000000n, // 0.001 ETH
          valueLimit: createLifetimeLimit(10000000000000000n), // 0.01 ETH total
        },
      ],
    };

    expect(spec.callPolicies).toHaveLength(2);
    expect(spec.transferPolicies).toHaveLength(1);

    const json = sessionSpecToJSON(spec);
    const parsed = JSON.parse(json);
    expect(parsed.callPolicies).toHaveLength(2);
    expect(parsed.transferPolicies).toHaveLength(1);
  });
});
