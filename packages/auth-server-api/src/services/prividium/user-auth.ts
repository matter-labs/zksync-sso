import type { UserAuthResult, UserProfile } from "./types.js";

/**
 * Verifies a user's Prividium authorization by calling the /api/profiles/me endpoint.
 * This both validates the token and returns the user's ID.
 *
 * @param authorizationHeader The full Authorization header value (e.g., "Bearer <token>")
 * @param permissionsApiUrl The base URL for the Prividium permissions API
 * @returns UserAuthResult with validity, userId if valid, and error type if invalid
 */
export async function verifyUserAuth(
  authorizationHeader: string,
  permissionsApiUrl: string,
): Promise<UserAuthResult> {
  let response: Response;
  try {
    response = await fetch(`${permissionsApiUrl}/api/profiles/me`, {
      method: "GET",
      headers: {
        Authorization: authorizationHeader,
        "Content-Type": "application/json",
      },
    });
  } catch (error) {
    // Network error (server down, DNS failure, connection refused, etc.)
    console.error("Network error while verifying user auth:", error);
    return { valid: false, error: "network_error" };
  }

  if (!response.ok) {
    // 401/403 = invalid token, 5xx = server error
    if (response.status === 401 || response.status === 403) {
      return { valid: false, error: "invalid_token" };
    }
    console.error(`Server error while verifying user auth: ${response.status}`);
    return { valid: false, error: "server_error" };
  }

  try {
    const profile = (await response.json()) as UserProfile;

    if (!profile.id) {
      return { valid: false, error: "invalid_token" };
    }

    return {
      valid: true,
      userId: profile.id,
    };
  } catch (error) {
    console.error("Failed to parse user profile response:", error);
    return { valid: false, error: "server_error" };
  }
}
