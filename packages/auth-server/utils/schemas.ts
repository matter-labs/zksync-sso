import { getAddress } from "viem";
import { z } from "zod";

export const AddressSchema = z.string().transform((val, ctx) => {
  try {
    return getAddress(val);
  } catch {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: "Not a valid address",
    });
    return z.NEVER;
  }
});
