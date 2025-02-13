import type { RegisterNewPasskeyReturnType } from "zksync-sso/client/passkey";
import { registerNewPasskey } from "zksync-sso/client/passkey";

export const usePasskeyRegister = () => {
  const generatePasskeyName = () => {
    let name = `ZKsync SSO ${(new Date()).toLocaleDateString("en-US")}`;
    name += ` ${(new Date()).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
    return name;
  };

  const { inProgress, error, execute: registerPasskey } = useAsync(async (): Promise<RegisterNewPasskeyReturnType> => {
    const name = generatePasskeyName();
    return await registerNewPasskey({
      userName: name,
      userDisplayName: name,
    });
  });

  return {
    inProgress,
    error,
    registerPasskey,
  };
};
