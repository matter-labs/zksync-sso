import { JWT } from "zksync-sso-circuits";

function extractJwt(hashStr: string): JWT {
  const parts = hashStr.replace("#", "").split("&");
  const [rawJwt] = parts
    .map((part) => part.split("="))
    .filter(([prop, _value]) => prop === "id_token")
    .map(([_prop, value]) => value);
  if (!rawJwt) {
    throw new Error("Missing jwt");
  }
  return new JWT(rawJwt);
}

async function waitForJwt(nonce: string, popup: Window): Promise<JWT> {
  const controller = new AbortController();
  return new Promise<JWT>((resolve, reject) => {
    popup.addEventListener("click", () => reject("Window closed"), { signal: controller.signal });
    const receiveMessage = (event: MessageEvent) => {
      if (event.origin !== window.location.origin) {
        return;
      }

      const { data } = event;
      console.log(data);
      if (data.content.type !== "jwt") {
        return;
      }

      const jwt = extractJwt(data.content.hashString);

      if (jwt.nonce !== nonce) {
        return reject("Wrong nonce");
      }

      resolve(jwt);
    };

    window.addEventListener("message", receiveMessage, { signal: controller.signal });
  }).finally(() => controller.abort());
}

async function loginWithGoogle(publicClient: string, nonce: string) {
  const strWindowFeatures = "toolbar=no, menubar=no, width=600, height=700, top=100, left=100";

  const clientId = encodeURIComponent(publicClient);
  const responseType = encodeURIComponent("id_token");
  const scope = encodeURIComponent("openid");
  const redirectUri = encodeURI(`${window.location.origin}/oauth/plain`);

  const query = `?client_id=${clientId}&response_type=${responseType}&scope=${scope}&redirect_uri=${redirectUri}&nonce=${nonce}`;
  const url = `https://accounts.google.com/o/oauth2/v2/auth${query}`;

  const popup = window.open(url, "login with google", strWindowFeatures);

  if (popup === null) {
    throw new Error("Could not open google popup");
  }

  return await waitForJwt(nonce, popup);
}

export function useGoogleOauth() {
  const config = useRuntimeConfig();
  const { execute, inProgress, result, error } = useAsync((nonce: string) => {
    return loginWithGoogle(config.public.googlePublicClient, nonce);
  });

  return {
    startGoogleOauth: execute,
    googleOauthInProgress: inProgress,
    jwt: result,
    error,
  };
}
