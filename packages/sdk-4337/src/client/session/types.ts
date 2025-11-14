import type { Address, Hex } from "viem";

/**
 * Limit types for usage tracking
 * Uses numeric values for on-chain compatibility
 */
export enum LimitType {
  Unlimited = 0,
  Lifetime = 1,
  Allowance = 2,
}

/**
 * Period constants for time-based allowances (in seconds)
 */
export const LIMIT_PERIODS = {
  Hourly: 3600n,
  Daily: 86400n,
  Weekly: 604800n,
  Monthly: 2592000n, // 30 days
  Yearly: 31536000n, // 365 days
} as const;

/**
 * Usage limit structure
 * @member limitType - Type of limit (Unlimited, Lifetime, or time-based allowance)
 * @member limit - Maximum value allowed (as bigint for U256 compatibility)
 * @member period - Period in seconds for allowance limits (0 for Unlimited/Lifetime)
 */
export type UsageLimit = {
  limitType: LimitType;
  limit: bigint;
  period: bigint;
};

/**
 * Constraint condition for parameter validation
 * Uses numeric values for on-chain compatibility
 */
export enum ConstraintCondition {
  Unconstrained = 0,
  Equal = 1,
  Greater = 2,
  Less = 3,
  GreaterEqual = 4,
  LessEqual = 5,
  NotEqual = 6,
}

/**
 * Constraint for validating transaction parameters
 * @member condition - Comparison condition
 * @member index - Parameter index in calldata
 * @member refValue - Reference value for comparison (as hex string)
 * @member limit - Usage limit for this constraint
 */
export type Constraint = {
  condition: ConstraintCondition;
  index: bigint;
  refValue: Hex;
  limit: UsageLimit;
};

/**
 * Policy for contract function calls
 * @member target - Contract address
 * @member selector - Function selector (4 bytes)
 * @member maxValuePerUse - Maximum value per transaction
 * @member valueLimit - Total value limit with tracking period
 * @member constraints - Parameter validation constraints
 */
export type CallPolicy = {
  target: Address;
  selector: Hex;
  maxValuePerUse: bigint;
  valueLimit: UsageLimit;
  constraints: Constraint[];
};

/**
 * Policy for simple value transfers (no calldata)
 * @member target - Recipient address
 * @member maxValuePerUse - Maximum value per transaction
 * @member valueLimit - Total value limit with tracking period
 */
export type TransferPolicy = {
  target: Address;
  maxValuePerUse: bigint;
  valueLimit: UsageLimit;
};

/**
 * Complete session specification
 * @member signer - Session key public address (can sign transactions within policy limits)
 * @member expiresAt - Unix timestamp when session expires
 * @member feeLimit - Maximum cumulative fees the session can pay
 * @member callPolicies - Policies for contract calls (requires calldata)
 * @member transferPolicies - Policies for value transfers (no calldata)
 */
export type SessionSpec = {
  signer: Address;
  expiresAt: bigint;
  feeLimit: UsageLimit;
  callPolicies: CallPolicy[];
  transferPolicies: TransferPolicy[];
};

/**
 * Helper to create unlimited limit
 */
export const LimitUnlimited: UsageLimit = {
  limitType: LimitType.Unlimited,
  limit: 0n,
  period: 0n,
};

/**
 * Helper to create zero (disabled) limit
 */
export const LimitZero: UsageLimit = {
  limitType: LimitType.Lifetime,
  limit: 0n,
  period: 0n,
};

/**
 * Helper to create lifetime limit (no period tracking)
 */
export const createLifetimeLimit = (limit: bigint): UsageLimit => ({
  limitType: LimitType.Lifetime,
  limit,
  period: 0n,
});

/**
 * Helper to create time-based allowance limit
 * @param period - Period in seconds for the allowance window
 * @param limit - Maximum value allowed within the period
 */
export const createAllowanceLimit = (
  period: bigint,
  limit: bigint,
): UsageLimit => ({
  limitType: LimitType.Allowance,
  limit,
  period,
});

// ============================================================================
// Backward Compatibility Type Aliases (for legacy SDK migration)
// ============================================================================

/**
 * @deprecated Use SessionSpec instead. Provided for backward compatibility with legacy SDK.
 */
export type SessionConfig = SessionSpec;

/**
 * @deprecated Use UsageLimit instead. Provided for backward compatibility with legacy SDK.
 */
export type Limit = UsageLimit;
