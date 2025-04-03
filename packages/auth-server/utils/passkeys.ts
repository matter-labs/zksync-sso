export async function getPasskeyCredential() {
  const credential = await navigator.credentials.get({
    publicKey: {
      challenge: new Uint8Array(32),
      userVerification: "discouraged",
    },
  }) as PublicKeyCredential | null;
  return credential;
}
