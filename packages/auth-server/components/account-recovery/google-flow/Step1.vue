<template>
  <p class="text-center text-gray-600 dark:text-gray-400">
    Google recovery allows you to link your Google account for secure account recovery in case you lose access.
  </p>
  <ZkButton
    type="primary"
    class="w-full md:max-w-48 mt-4"
    @click="loginWithGoogle"
  >
    Continue
  </ZkButton>
  <ZkButton
    type="secondary"
    class="w-full md:max-w-48"
    @click="$emit('back')"
  >
    Back
  </ZkButton>
</template>

<script setup lang="ts">
import { toHex } from "viem";
import { JWT } from "zksync-sso-circuits";

const emit = defineEmits<{
  (e: "next" | "back"): void;
}>();

const runtimeConfig = useRuntimeConfig();
const { setJwt } = useRecoveryOidc();

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

async function loginWithGoogle() {
  const strWindowFeatures = "toolbar=no, menubar=no, width=600, height=700, top=100, left=100";

  const randomValues = new Uint8Array(32);
  const nonce = toHex(crypto.getRandomValues(randomValues));

  const clientId = encodeURIComponent(runtimeConfig.public.googlePubblicClient);
  const responseType = encodeURIComponent("id_token");
  const scope = encodeURIComponent("openid");
  const redirectUri = encodeURI(`${window.location.origin}/oauth/plain`);

  const query = `?client_id=${clientId}&response_type=${responseType}&scope=${scope}&redirect_uri=${redirectUri}&nonce=${nonce}`;
  const url = `https://accounts.google.com/o/oauth2/v2/auth${query}`;

  const popup = window.open(url, "login with google", strWindowFeatures);

  if (popup === null) {
    throw new Error("Could not open google popup");
  }

  const jwt = await waitForJwt(nonce, popup);
  console.log(jwt);
  setJwt(jwt);
  emit("next");
}
</script>
