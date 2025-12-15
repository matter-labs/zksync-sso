import type { Address } from "viem";
import type { BundlerClient } from "viem/account-abstraction";

/**
 * Wraps a bundler client to automatically inject paymaster address into all sendUserOperation calls.
 * This enables sponsored transactions without requiring callers to pass paymaster on every call.
 *
 * @param bundlerClient - The original bundler client
 * @param paymasterAddress - The paymaster contract address
 * @returns A wrapped bundler client that injects paymaster into sendUserOperation
 */
export function createPaymasterBundlerClient(
  bundlerClient: BundlerClient,
  paymasterAddress: Address,
): BundlerClient {
  // Create a proxy that intercepts sendUserOperation to add paymaster
  return new Proxy(bundlerClient, {
    get(target, prop, receiver) {
      if (prop === "sendUserOperation") {
        return async (args: Parameters<BundlerClient["sendUserOperation"]>[0]) => {
          // Inject paymaster if not already provided
          const argsWithPaymaster = {
            ...args,
            paymaster: args.paymaster ?? paymasterAddress,
          };
          console.log(`[Paymaster Middleware] Injecting paymaster: ${paymasterAddress}`);
          console.log("[Paymaster Middleware] Args with paymaster:", {
            hasPaymaster: !!argsWithPaymaster.paymaster,
            paymaster: argsWithPaymaster.paymaster,
          });
          return target.sendUserOperation(argsWithPaymaster);
        };
      }
      return Reflect.get(target, prop, receiver);
    },
  }) as BundlerClient;
}
