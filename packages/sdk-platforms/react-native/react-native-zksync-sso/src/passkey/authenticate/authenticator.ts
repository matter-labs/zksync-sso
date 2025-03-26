// @ts-ignore
import { type PasskeyAuthenticatorAsync } from 'react-native-zksync-sso';
import { authenticateWithPasskey } from '../authenticate';

/**
 * Authenticator class that implements passkey-based authentication.
 * This class handles the signing of messages using platform-specific passkey authentication.
 */
export class Authenticator implements PasskeyAuthenticatorAsync {
    private rpId: string;

    /**
     * Creates a new Authenticator instance.
     * @param rpId - The relying party ID used for passkey authentication
     */
    constructor(rpId: string) {
        this.rpId = rpId;
    }

    /**
     * Signs a message using the platform's passkey authentication.
     * @param message - The message to sign as an ArrayBuffer
     * @returns A Promise that resolves to the signed message as an ArrayBuffer
     */
    async signMessage(
        message: ArrayBuffer,
    ): Promise<ArrayBuffer> {
        console.log("signMessage message:", message);
        const result = await authenticateWithPasskey(message, this.rpId);
        console.log("signMessage result:", result);
        return result;
    }
}