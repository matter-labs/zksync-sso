import { type Address, encodeAbiParameters, type Hex, keccak256 } from "viem";

import { SessionKeyValidatorAbi } from "../../abi/SessionKeyValidator.js";
import { LimitType, type SessionSpec, type UsageLimit } from "./types.js";

/**
 * Utility type that converts all bigint values to strings recursively
 */
export type ConvertBigIntToString<T> = T extends bigint
  ? string
  : T extends Array<infer U>
    ? Array<ConvertBigIntToString<U>>
    : T extends object
      ? { [K in keyof T]: ConvertBigIntToString<T[K]> }
      : T;

/**
 * SessionSpec with all bigint values converted to strings for JSON serialization
 */
export type SessionSpecJSON = ConvertBigIntToString<SessionSpec>;

/**
 * Converts a SessionSpec to JSON string format expected by WASM bindings.
 * All bigint values are converted to strings for safe serialization.
 */
export function sessionSpecToJSON(spec: SessionSpec): string {
  const usageLimitToJSON = (limit: UsageLimit) => {
    let limitType = "Unlimited";
    if (limit.limitType === LimitType.Lifetime) limitType = "Lifetime";
    else if (limit.limitType === LimitType.Allowance) limitType = "Allowance";

    return {
      limitType,
      limit: limit.limit.toString(),
      period: limit.period.toString(),
    };
  };

  return JSON.stringify({
    signer: spec.signer,
    expiresAt: spec.expiresAt.toString(),
    feeLimit: usageLimitToJSON(spec.feeLimit),
    callPolicies: spec.callPolicies.map((policy) => ({
      target: policy.target,
      selector: policy.selector,
      maxValuePerUse: policy.maxValuePerUse.toString(),
      valueLimit: usageLimitToJSON(policy.valueLimit),
      constraints: policy.constraints.map((constraint) => ({
        condition: constraint.condition,
        index: constraint.index.toString(),
        refValue: constraint.refValue,
        limit: usageLimitToJSON(constraint.limit),
      })),
    })),
    transferPolicies: spec.transferPolicies.map((policy) => ({
      target: policy.target,
      maxValuePerUse: policy.maxValuePerUse.toString(),
      valueLimit: usageLimitToJSON(policy.valueLimit),
    })),
  });
}

/**
 * Extract function selector from calldata (first 4 bytes)
 */
export function extractSelector(callData: Hex): Hex | undefined {
  if (!callData || callData.length < 10) return undefined; // 0x + 8 hex chars = 10
  return callData.slice(0, 10) as Hex;
}

/**
 * Find matching policy for a transaction
 */
export function findMatchingPolicy(
  spec: SessionSpec,
  target: Address,
  selector?: Hex,
): { type: "call" | "transfer"; policy: SessionSpec["callPolicies"][0] | SessionSpec["transferPolicies"][0] } | null {
  // If selector provided, look for call policy
  if (selector) {
    const callPolicy = spec.callPolicies.find(
      (p) => p.target.toLowerCase() === target.toLowerCase() && p.selector.toLowerCase() === selector.toLowerCase(),
    );
    if (callPolicy) return { type: "call", policy: callPolicy };
  }

  // Otherwise look for transfer policy
  const transferPolicy = spec.transferPolicies.find(
    (p) => p.target.toLowerCase() === target.toLowerCase(),
  );
  if (transferPolicy) return { type: "transfer", policy: transferPolicy };

  return null;
}

/**
 * Validates that a transaction fits within session policies
 * Returns error message if invalid, null if valid
 */
export function validateTransactionAgainstSession(
  spec: SessionSpec,
  target: Address,
  value: bigint,
  callData?: Hex,
): string | null {
  const selector = callData ? extractSelector(callData) : undefined;
  const match = findMatchingPolicy(spec, target, selector);

  if (!match) {
    return selector
      ? `No call policy found for target ${target} with selector ${selector}`
      : `No transfer policy found for target ${target}`;
  }

  // Check max value per use
  if (value > match.policy.maxValuePerUse) {
    return `Value ${value} exceeds maxValuePerUse ${match.policy.maxValuePerUse}`;
  }

  return null; // Valid
}

/**
 * Checks if a session has expired
 */
export function isSessionExpired(
  spec: SessionSpec,
  currentTimestamp?: bigint,
): boolean {
  const now = currentTimestamp ?? BigInt(Math.floor(Date.now() / 1000));
  return now > spec.expiresAt;
}

/**
 * Gets human-readable expiry time
 */
export function getSessionExpiryDate(spec: SessionSpec): Date {
  return new Date(Number(spec.expiresAt) * 1000);
}

/**
 * Computes the hash of a session specification.
 * This hash is signed by the session key to prove ownership.
 */
export function getSessionHash(spec: SessionSpec): Hex {
  const createSessionFunction = SessionKeyValidatorAbi.find(
    (x) => x.type === "function" && x.name === "createSession",
  );
  if (!createSessionFunction) throw new Error("createSession function not found in ABI");

  const sessionSpecParam = createSessionFunction.inputs.find((x) => x.name === "sessionSpec");
  if (!sessionSpecParam) throw new Error("sessionSpec param not found in createSession ABI");

  const encoded = encodeAbiParameters(
    [sessionSpecParam],
    [spec as any],
  );
  return keccak256(encoded);
}
