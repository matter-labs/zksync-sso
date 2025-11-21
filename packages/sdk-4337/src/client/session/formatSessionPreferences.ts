import type { Hex } from "viem";
import {
  type Abi,
  type AbiFunction,
  type AbiStateMutability,
  type Address,
  type ContractFunctionArgs,
  type ContractFunctionName,
  encodeAbiParameters,
  getAddress,
  toFunctionSelector,
  toHex,
} from "viem";

import {
  type CallPolicy,
  ConstraintCondition,
  LimitType,
  LimitUnlimited,
  LimitZero,
  type SessionSpec,
  type TransferPolicy,
  type UsageLimit,
} from "./types.js";

// ============================================================================
// Helper Types
// ============================================================================

/**
 * Partial limit type for user-friendly session configuration
 * Can be simplified bigint or full object with limitType
 */
export type PartialLimit =
  | bigint
  | {
    limit: bigint;
    period?: string | bigint;
  }
  | {
    limitType: "lifetime" | LimitType.Lifetime;
    limit: bigint;
  }
  | {
    limitType: "unlimited" | LimitType.Unlimited;
  }
  | {
    limitType: "allowance" | LimitType.Allowance;
    limit: bigint;
    period: string | bigint;
  };

/**
 * Partial call policy for user-friendly configuration
 */
export type PartialCallPolicy = {
  address: Address;
  abi: Abi;
  functionName: string;
  maxValuePerUse?: bigint;
  valueLimit?: PartialLimit;
  constraints?: {
    index: number;
    value?: unknown;
    condition?: keyof typeof ConstraintCondition;
    limit?: PartialLimit;
  }[];
};

/**
 * Partial transfer policy for user-friendly configuration
 */
export type PartialTransferPolicy = {
  to: Address;
  maxValuePerUse?: bigint;
  valueLimit?: PartialLimit;
};

/**
 * User-friendly session preferences
 * Simplified interface that gets converted to full SessionSpec
 */
export interface SessionPreferences {
  expiry?: string | bigint | Date;
  feeLimit?: PartialLimit;
  contractCalls?: PartialCallPolicy[];
  transfers?: PartialTransferPolicy[];
}

// ============================================================================
// Type Utilities
// ============================================================================

type ContractWriteMutability = Extract<AbiStateMutability, "nonpayable" | "payable">;

// Helper type to convert string indices to numeric indices
type ToNumber<S extends string> = S extends `${infer N extends number}` ? N : never;

// Extract numeric keys as strings
type NumericKeys<T extends unknown[]> = Extract<keyof T, `${number}`>;

// Updated IndexedValues type with numeric indices
export type IndexedValues<T extends unknown[]> = Array<{
  [K in NumericKeys<T>]: { index: ToNumber<K>; value?: T[K] };
}[NumericKeys<T>]>;

export interface TypesafePartialCallPolicy<
  abi extends Abi,
  functionName extends ContractFunctionName<abi, ContractWriteMutability>,
> extends PartialCallPolicy {
  abi: abi;
  functionName: functionName;
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  constraints?: (IndexedValues<ContractFunctionArgs<abi, ContractWriteMutability, functionName>>[number] & {
    condition?: keyof typeof ConstraintCondition;
    limit?: PartialLimit;
  })[];
}

// Typesafety helper function
export const callPolicy = <
  abi extends Abi,
  functionName extends ContractFunctionName<abi, ContractWriteMutability>,
>(
  policy: TypesafePartialCallPolicy<abi, functionName>,
): PartialCallPolicy => policy as PartialCallPolicy;

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Convert ms string format (e.g. "1h", "24h", "7d") to seconds as bigint
 * Supports formats: ms, s, m, h, d, w, y
 */
export const msStringToSeconds = (value: string): bigint => {
  const units: Record<string, number> = {
    ms: 0.001,
    s: 1,
    m: 60,
    h: 3600,
    d: 86400,
    w: 604800,
    y: 31536000,
  };

  const match = value.match(/^(\d+(?:\.\d+)?)(ms|s|m|h|d|w|y)$/);
  if (!match) {
    throw new Error(`Invalid date format: ${value}. Expected format like "1h", "24h", "7d"`);
  }

  const [, numStr, unit] = match;
  const num = parseFloat(numStr);
  const seconds = num * units[unit];

  if (seconds < 0) throw new Error(`Date can't be in the past: ${value}`);
  if (seconds === 0) throw new Error(`Date can't be zero: ${value}`);

  return BigInt(Math.floor(seconds));
};

const DYNAMIC_ABI_INPUT_TYPES = ["bytes", "string"];

export const isDynamicInputType = (inputType: string) => {
  return inputType.endsWith("[]") || DYNAMIC_ABI_INPUT_TYPES.includes(inputType);
};

type AbiParameter = {
  type: string;
  name?: string;
  components?: readonly AbiParameter[];
};

const includesDynamicInputType = (abiParameters: readonly AbiParameter[]): boolean => {
  return abiParameters.some((input) => {
    const isDynamicType = isDynamicInputType(input.type);
    if (isDynamicType) return true;

    if (input.type.startsWith("tuple")) {
      const components = input.components;
      if (!components) throw new Error("Tuple without components is unsupported");
      return includesDynamicInputType(components);
    }
    return false;
  });
};

export const isFollowedByDynamicInputType = (abiFunction: AbiFunction, targetInputIndex: number) => {
  if (targetInputIndex >= abiFunction.inputs.length) {
    throw new Error(`Input index ${targetInputIndex} is out of bounds`);
  }

  return includesDynamicInputType(abiFunction.inputs.slice(0, targetInputIndex));
};

export const encodedInputToAbiChunks = (encodedInput: string) => {
  if (!encodedInput.startsWith("0x")) {
    throw new Error("Input is not a valid hex string");
  }
  return (encodedInput.slice(2).match(/.{1,64}/g) || []).map((e) => `0x${e}`) as Hex[]; // 32 bytes abi chunks
};

const getDummyBytesValue = (type: string) => {
  const size = parseInt(type.slice(5)) || 32;
  return toHex("", { size });
};

// Function to generate dummy values for ABI types
const getDummyValue = (type: string) => {
  if (type === "address") return "0x36615Cf349d7F6344891B1e7CA7C72883F5dc049" as Address;
  if (type.startsWith("uint") || type.startsWith("int")) return 0n; // BigInt for numbers
  if (type.startsWith("bytes")) return getDummyBytesValue(type); // Empty bytes
  if (type === "bool") return false;
  if (type === "string") return ""; // Empty string
  throw new Error(`Unsupported ABI type: ${type}`);
};

function getArrayComponents(
  type: string,
): [length: number | null, innerType: string] | undefined {
  const matches = type.match(/^(.*)\[(\d+)?\]$/);
  return matches
    ? [matches[2] ? Number(matches[2]) : null, matches[1]]
    : undefined;
}

// Recursive function to fill dummy values for complex types like tuples
const getDummyValues = (inputs: readonly AbiParameter[]): unknown[] => {
  return inputs.map((input) => {
    const arrayComponents = getArrayComponents(input.type);
    if (arrayComponents) {
      // Recursively fill array components
      const [length, innerType] = arrayComponents;
      if (!length) throw new Error("Dynamic array length is unsupported");
      const arrayValue = Array.from({ length }, () => getDummyValues([{
        ...input,
        type: innerType,
      }]));
      return arrayValue;
    }
    if (input.type.startsWith("tuple")) {
      // Recursively fill tuple components
      const components = input.components;
      if (!components) throw new Error("Tuple without components is unsupported");
      return getDummyValues(components);
    }
    return getDummyValue(input.type);
  });
};

export const getParameterChunkIndex = (
  abiFunction: AbiFunction,
  targetInputIndex: number,
): number => {
  if (targetInputIndex >= abiFunction.inputs.length) {
    throw new Error(`Input index ${targetInputIndex} is out of bounds`);
  }

  const inputs = abiFunction.inputs.slice(0, targetInputIndex);
  const dummyValues = getDummyValues(inputs);
  const encoded = encodeAbiParameters(inputs, dummyValues);
  const chunks = encodedInputToAbiChunks(encoded);
  const chunkIndex = chunks.length;

  return chunkIndex;
};

// ============================================================================
// Format Functions
// ============================================================================

/**
 * Format user-friendly limit preferences to full UsageLimit
 */
export const formatLimitPreferences = (limit: PartialLimit): UsageLimit => {
  /* Just bigint was passed */
  if (typeof limit === "bigint") {
    return {
      limitType: LimitType.Lifetime,
      limit,
      period: 0n,
    };
  }

  /* LimitType was specified */
  if ("limitType" in limit) {
    if (limit.limitType === "lifetime" || limit.limitType === LimitType.Lifetime) {
      return {
        limitType: LimitType.Lifetime,
        limit: limit.limit,
        period: 0n,
      };
    } else if (limit.limitType === "unlimited" || limit.limitType === LimitType.Unlimited) {
      return {
        limitType: LimitType.Unlimited,
        limit: 0n,
        period: 0n,
      };
    } else if (limit.limitType === "allowance" || limit.limitType === LimitType.Allowance) {
      return {
        limitType: LimitType.Allowance,
        limit: limit.limit,
        period: typeof limit.period === "string" ? msStringToSeconds(limit.period) : limit.period,
      };
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    throw new Error(`Invalid limit type: ${(limit as any).limitType}`);
  }

  /* LimitType not selected */
  if (limit.period) {
    return {
      limitType: LimitType.Allowance,
      limit: limit.limit,
      period: typeof limit.period === "string" ? msStringToSeconds(limit.period) : limit.period,
    };
  }
  return {
    limitType: LimitType.Lifetime,
    limit: limit.limit,
    period: 0n,
  };
};

/**
 * Format user-friendly date preferences to bigint timestamp
 */
export const formatDatePreferences = (date: string | bigint | Date): bigint => {
  if (typeof date === "string") {
    const now = Math.floor(new Date().getTime() / 1000);
    const seconds = msStringToSeconds(date);
    return BigInt(now) + seconds;
  }
  if (date instanceof Date) {
    const seconds = Math.floor(date.getTime() / 1000);
    return BigInt(seconds);
  }
  return date;
};

/**
 * Format user-friendly session preferences to full SessionSpec (without signer)
 * @param preferences - User-friendly session configuration
 * @param defaults - Default values for expiry and feeLimit if not specified
 * @returns SessionSpec without the signer field (to be added separately)
 */
export function formatSessionPreferences(
  preferences: SessionPreferences,
  defaults: {
    expiresAt: bigint;
    feeLimit: UsageLimit;
  },
): Omit<SessionSpec, "signer"> {
  return {
    expiresAt: preferences.expiry ? formatDatePreferences(preferences.expiry) : defaults.expiresAt,
    feeLimit: preferences.feeLimit ? formatLimitPreferences(preferences.feeLimit) : defaults.feeLimit,
    callPolicies: preferences.contractCalls?.map((policy) => {
      const allowedStateMutability: ContractWriteMutability[] = ["nonpayable", "payable"];
      const abiFunction = (policy.abi as Abi).find((fn) => fn.type === "function" && fn.name === policy.functionName && (allowedStateMutability as AbiStateMutability[]).includes(fn.stateMutability)) as AbiFunction;
      if (!abiFunction) throw new Error(`Function not found in the provided ABI: ${policy.functionName}`);

      const selector = toFunctionSelector(abiFunction);
      const valueLimit = policy.valueLimit ? formatLimitPreferences(policy.valueLimit) : LimitZero;

      return {
        target: getAddress(policy.address.toLowerCase()),
        maxValuePerUse: policy.maxValuePerUse ?? valueLimit.limit,
        valueLimit,
        selector: selector,
        constraints: policy.constraints?.map((constraint) => {
          const limit = constraint.limit ? formatLimitPreferences(constraint.limit) : LimitUnlimited;
          let condition = ConstraintCondition.Unconstrained;
          if (constraint.condition) {
            condition = ConstraintCondition[constraint.condition];
          } else if (constraint.value !== undefined && constraint.value !== null) {
            condition = ConstraintCondition.Equal;
          }

          const input = abiFunction.inputs[constraint.index];
          if (!input) {
            throw new Error(`Target function parameter not found in the provided ABI function. Provided at function ${policy.functionName}, index ${constraint.index}`);
          }

          const isDynamicType = isDynamicInputType(input.type);
          if (isDynamicType) {
            throw new Error(`Function parameters with dynamic types are not supported for constraint validation. Provided at function ${policy.functionName}, index ${constraint.index}`);
          }

          const isFollowedByDynamicType = isFollowedByDynamicInputType(abiFunction, constraint.index);
          if (isFollowedByDynamicType) {
            throw new Error(`Target function parameter is followed by a dynamic type parameter. Provided at function ${policy.functionName}, index ${constraint.index}`);
          }

          const startingAbiChunkIndex = getParameterChunkIndex(abiFunction, constraint.index);
          if (constraint.value === undefined || constraint.value === null) {
            return {
              index: BigInt(startingAbiChunkIndex),
              condition: ConstraintCondition.Unconstrained,
              refValue: toHex("", { size: 32 }),
              limit,
            };
          }

          const encodedInput = encodeAbiParameters([input], [constraint.value]);
          const abiBytesChunks = encodedInputToAbiChunks(encodedInput);
          const ALLOWED_OVERFLOW_CONDITIONS: ConstraintCondition[] = [
            ConstraintCondition.Unconstrained,
            ConstraintCondition.Equal,
            ConstraintCondition.NotEqual,
          ];
          const ALLOWED_OVERFLOW_LIMIT_TYPES: LimitType[] = [
            LimitType.Unlimited,
          ];

          if (
            abiBytesChunks.length > 1
            && (
              !ALLOWED_OVERFLOW_CONDITIONS.includes(condition) // Can't validate condition (e.g. < >) if value is split across multiple chunks
              || !ALLOWED_OVERFLOW_LIMIT_TYPES.includes(limit.limitType) // Can't validate limit if value is split across multiple chunks
            )
          ) {
            throw new Error(`Encoded input size of parameter at index ${constraint.index} of ${policy.functionName} exceeds the maximum size of 32 bytes: ${abiBytesChunks.length * 32} bytes`);
          };

          return abiBytesChunks.map((abiChunk, index) => ({
            index: BigInt(startingAbiChunkIndex + index),
            condition,
            refValue: abiChunk,
            limit,
          }));
        }).flat() ?? [],
      } as CallPolicy;
    }) ?? [],
    transferPolicies: preferences.transfers?.map((policy) => {
      const valueLimit = policy.valueLimit ? formatLimitPreferences(policy.valueLimit) : LimitZero;
      return {
        target: getAddress(policy.to.toLowerCase()),
        maxValuePerUse: policy.maxValuePerUse ?? valueLimit.limit,
        valueLimit,
      } as TransferPolicy;
    }) ?? [],
  };
}
