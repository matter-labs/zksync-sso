import { http } from "viem";

/**
 * Creates an HTTP transport configured to use the Prividium RPC proxy.
 * All requests will include the admin's authorization token.
 *
 * @param proxyUrl The Prividium RPC proxy URL
 * @param adminToken The authenticated admin's JWT token
 * @returns A viem HTTP transport configured for the proxy
 */
export function createProxyTransport(proxyUrl: string, adminToken: string) {
  return http(proxyUrl, {
    fetchOptions: {
      headers: {
        Authorization: `Bearer ${adminToken}`,
      },
    },
  });
}
