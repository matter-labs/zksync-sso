export type PrividiumApiAuth = {
  getAuthHeaders(): Record<string, string> | null;
  authorize(): Promise<unknown>;
};

function withAuthHeaders(init: RequestInit | undefined, authHeaders: Record<string, string>): RequestInit {
  const headers = new Headers(init?.headers);
  for (const [key, value] of Object.entries(authHeaders)) {
    headers.set(key, value);
  }

  return {
    ...init,
    headers,
  };
}

async function getFreshAuthHeaders(auth: PrividiumApiAuth): Promise<Record<string, string>> {
  const authHeaders = auth.getAuthHeaders();
  if (authHeaders) {
    return authHeaders;
  }

  await auth.authorize();
  const refreshedHeaders = auth.getAuthHeaders();
  if (!refreshedHeaders) {
    throw new Error("Failed to get auth headers");
  }
  return refreshedHeaders;
}

export async function authenticatedFetch(
  auth: PrividiumApiAuth,
  input: Parameters<typeof fetch>[0],
  init?: Parameters<typeof fetch>[1],
): Promise<Response> {
  const send = async () => fetch(input, withAuthHeaders(init, await getFreshAuthHeaders(auth)));

  const response = await send();
  if (response.status !== 401) {
    return response;
  }

  await auth.authorize();
  return send();
}
