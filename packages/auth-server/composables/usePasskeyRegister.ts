import type { Hex } from "viem";
import { createWebAuthnCredential } from "zksync-sso-4337/client";

// Return type matching what components expect
export type RegisterNewPasskeyReturnType = {
  credentialId: Hex;
  credentialIdBase64url: string;
  credentialPublicKey: { x: Hex; y: Hex };
};

export const usePasskeyRegister = () => {
  const generatePasskeyName = () => {
    let name = `ZKsync SSO ${(new Date()).toLocaleDateString("en-US")}`;
    name += ` ${(new Date()).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
    return name;
  };

  const { inProgress, error, execute: registerPasskey } = useAsync(async (): Promise<RegisterNewPasskeyReturnType> => {
    const name = generatePasskeyName();
    const result = await createWebAuthnCredential({
      rpId: window.location.hostname,
      rpName: "ZKsync SSO",
      name: name,
      displayName: name,
    });

    return {
      credentialId: result.credentialId,
      credentialIdBase64url: result.credentialIdBase64url,
      credentialPublicKey: result.publicKey,
    };
  });

  return {
    inProgress,
    error,
    registerPasskey,
  };
};
