/**
 * Compatibility layer for migrating from legacy SDK (packages/sdk) to new SDK (packages/sdk-4337)
 *
 * This module provides type conversions and helpers for users migrating from the old
 * createZksyncSessionClient API to the new toSessionSmartAccount API.
 *
 * The legacy SDK used the same numeric enum values as the new SDK, so conversion is
 * mainly a matter of type aliases and ensuring compatibility.
 */

import type { Address, Hash, Hex } from "viem";

import type {
  CallPolicy,
  Constraint,
  ConstraintCondition,
  LimitType,
  SessionSpec,
  TransferPolicy,
  UsageLimit,
} from "./types.js";

// ============================================================================
// Legacy SDK Type Definitions (for reference)
// ============================================================================

/**
 * Legacy SessionConfig type from packages/sdk
 * This is structurally identical to SessionSpec
 */
export type LegacySessionConfig = {
  signer: Address;
  expiresAt: bigint;
  feeLimit: LegacyLimit;
  callPolicies: LegacyCallPolicy[];
  transferPolicies: LegacyTransferPolicy[];
};

/**
 * Legacy Limit type from packages/sdk
 * This is structurally identical to UsageLimit
 */
export type LegacyLimit = {
  limitType: LimitType;
  limit: bigint;
  period: bigint;
};

/**
 * Legacy CallPolicy type from packages/sdk
 */
export type LegacyCallPolicy = {
  target: Address;
  selector: Hash; // Legacy uses Hash (32 bytes), new uses Hex (4 bytes)
  maxValuePerUse: bigint;
  valueLimit: LegacyLimit;
  constraints: LegacyConstraint[];
};

/**
 * Legacy TransferPolicy type from packages/sdk
 */
export type LegacyTransferPolicy = {
  target: Address;
  maxValuePerUse: bigint;
  valueLimit: LegacyLimit;
};

/**
 * Legacy Constraint type from packages/sdk
 */
export type LegacyConstraint = {
  index: bigint;
  condition: ConstraintCondition;
  refValue: Hash;
  limit: LegacyLimit;
};

// ============================================================================
// Conversion Functions
// ============================================================================

/**
 * Convert legacy Limit to new UsageLimit
 * Since both use the same structure, this is a simple type cast
 */
export function legacyLimitToUsageLimit(limit: LegacyLimit): UsageLimit {
  return {
    limitType: limit.limitType,
    limit: limit.limit,
    period: limit.period,
  };
}

/**
 * Convert legacy Constraint to new Constraint
 */
export function legacyConstraintToConstraint(
  constraint: LegacyConstraint,
): Constraint {
  return {
    condition: constraint.condition,
    index: constraint.index,
    refValue: constraint.refValue,
    limit: legacyLimitToUsageLimit(constraint.limit),
  };
}

/**
 * Convert legacy CallPolicy to new CallPolicy
 * Note: The legacy selector is a Hash (32 bytes), but new SDK expects Hex (4 bytes)
 * This function truncates to the first 4 bytes (function selector)
 */
export function legacyCallPolicyToCallPolicy(
  policy: LegacyCallPolicy,
): CallPolicy {
  // Ensure selector is 4 bytes (0x + 8 hex chars)
  const selector
    = policy.selector.length > 10
      ? (policy.selector.slice(0, 10) as Hex)
      : (policy.selector as Hex);

  return {
    target: policy.target,
    selector,
    maxValuePerUse: policy.maxValuePerUse,
    valueLimit: legacyLimitToUsageLimit(policy.valueLimit),
    constraints: policy.constraints.map(legacyConstraintToConstraint),
  };
}

/**
 * Convert legacy TransferPolicy to new TransferPolicy
 */
export function legacyTransferPolicyToTransferPolicy(
  policy: LegacyTransferPolicy,
): TransferPolicy {
  return {
    target: policy.target,
    maxValuePerUse: policy.maxValuePerUse,
    valueLimit: legacyLimitToUsageLimit(policy.valueLimit),
  };
}

/**
 * Convert legacy SessionConfig to new SessionSpec
 *
 * This is the main conversion function for migrating from the legacy SDK.
 *
 * @example
 * ```typescript
 * // Legacy SDK code
 * const legacyConfig: LegacySessionConfig = {
 *   signer: sessionKeyAddress,
 *   expiresAt: 1735689600n,
 *   feeLimit: { limitType: LimitType.Unlimited, limit: 0n, period: 0n },
 *   callPolicies: [...],
 *   transferPolicies: [],
 * };
 *
 * // Convert to new SDK
 * const sessionSpec = legacySessionConfigToSpec(legacyConfig);
 *
 * // Use with new SDK
 * const sessionAccount = await toSessionSmartAccount(client, {
 *   sessionSpec,
 *   sessionPrivateKey,
 * });
 * ```
 */
export function legacySessionConfigToSpec(
  config: LegacySessionConfig,
): SessionSpec {
  return {
    signer: config.signer,
    expiresAt: config.expiresAt,
    feeLimit: legacyLimitToUsageLimit(config.feeLimit),
    callPolicies: config.callPolicies.map(legacyCallPolicyToCallPolicy),
    transferPolicies: config.transferPolicies.map(
      legacyTransferPolicyToTransferPolicy,
    ),
  };
}
