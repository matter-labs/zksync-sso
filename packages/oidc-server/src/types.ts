import type { Hex } from "viem";
import { z } from "zod";

export const baseKeySchema = z.object({
  kid: z.string(),
  n: z.string(),
});

export type BaseKey = z.infer<typeof baseKeySchema>;

export const hexType = z.string()
  .regex(/^0x[0-9a-fA-F]+$/)
  .transform((string) => string as Hex);

export const keySchema = z.object({
  issHash: hexType, // Issuer hash (as bytes32)
  kid: hexType, // Key ID (as bytes32)
  rsaModulus: z.tuple([
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
    z.bigint(),
  ]), // 17 chunks
});

export const validNetworks = z.enum(["local", "sepolia", "mainnet"]);
export type ValidNetworks = z.infer<typeof validNetworks>;

export type Key = z.infer<typeof keySchema>;

export interface KeyFetcher {
  fetchKeys(): Promise<BaseKey[]>;
}
