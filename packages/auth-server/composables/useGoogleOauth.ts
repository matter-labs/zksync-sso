// TODO: OIDC/Google OAuth is not yet available in zksync-sso-4337
// This composable has been commented out until OIDC support is added to sdk-4337

/*
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
      if (data?.content?.type !== "jwt") {
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

export class PopupNotAllowed extends Error {}

async function loginWithGoogle(
  publicClient: string,
  nonce: string,
  loginHint: null | string = null,
  askForEmail = false,
): Promise<JWT> {
  const strWindowFeatures = "toolbar=no, menubar=no, width=600, height=700, top=100, left=100";

  const query = new URLSearchParams();
  query.set("client_id", publicClient);
  query.set("response_type", "id_token");
  query.set("scope", askForEmail ? "openid email" : "openid");
  const redirectUri = `${window.location.origin}/oauth/plain`;
  query.set("redirect_uri", redirectUri);
  query.set("nonce", nonce);

  if (loginHint !== null) {
    query.set("login_hint", loginHint);
  }

  const url = `https://accounts.google.com/o/oauth2/v2/auth?${query.toString()}`;
  const popup = window.open(url, "login with google", strWindowFeatures);

  if (popup === null) {
    throw new PopupNotAllowed("Could not open google popup");
  }

  return await waitForJwt(nonce, popup);
}

export function useGoogleOauth() {
  const { googlePublicClient } = useOidcConfig();
  const { execute, inProgress, result, error } = useAsync((nonce: string, hint: string | null = null, askForemail = false) => {
    return loginWithGoogle(googlePublicClient(), nonce, hint, askForemail);
  });

  return {
    startGoogleOauth: execute,
    googleOauthInProgress: inProgress,
    jwt: result,
    error,
  };
}
*/
