import { z } from "zod";

// Zod schema for UsageLimit
export const usageLimitSchema = z.object({
  limitType: z.string(),
  limit: z.string(),
  period: z.string(),
});

// Zod schema for Constraint
export const constraintSchema = z.object({
  condition: z.string(),
  index: z.string(),
  refValue: z.string().startsWith("0x"),
  limit: usageLimitSchema,
});

// Zod schema for CallPolicy
export const callPolicySchema = z.object({
  target: z.string().startsWith("0x"),
  selector: z.string().startsWith("0x"),
  maxValuePerUse: z.string(),
  valueLimit: usageLimitSchema,
  constraints: z.array(constraintSchema),
});

// Zod schema for TransferPolicy
export const transferPolicySchema = z.object({
  target: z.string().startsWith("0x"),
  maxValuePerUse: z.string(),
  valueLimit: usageLimitSchema,
});

// Zod schema for SessionSpecJSON (without signer)
export const sessionSpecJSONSchema = z.object({
  expiresAt: z.string(),
  feeLimit: usageLimitSchema,
  callPolicies: z.array(callPolicySchema),
  transferPolicies: z.array(transferPolicySchema),
});

// Request validation schema
export const deployAccountSchema = z.object({
  chainId: z.number(),
  credentialId: z.string().startsWith("0x"),
  credentialPublicKey: z.object({
    x: z.string().startsWith("0x"),
    y: z.string().startsWith("0x"),
  }),
  originDomain: z.string(),
  // Accept session as either a JSON string (from client helper) or an object
  session: z.union([z.string(), sessionSpecJSONSchema]).optional(),
  userId: z.string().startsWith("0x").optional(),
  eoaSigners: z.array(z.string().startsWith("0x")).optional(),
  paymaster: z.string().startsWith("0x").optional(),
});
