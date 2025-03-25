// @ts-ignore
import { type PasskeyAuthenticatorAsync } from 'react-native-zksync-sso';
import { authenticateWithPasskey } from '../authenticate';

export class Authenticator implements PasskeyAuthenticatorAsync {
    private rpId: string;

    constructor(rpId: string) {
        this.rpId = rpId;
    }

    async signMessage(
        message: ArrayBuffer,
    ): Promise<ArrayBuffer> {
        console.log("signMessage message:", message);
        const result = await authenticateWithPasskey(message, this.rpId);
        console.log("signMessage result:", result);
        return result;
    }
}