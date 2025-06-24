import {
    type PasskeyGetResult
} from 'react-native-passkey';
// @ts-ignore
import { type RpId } from 'react-native-zksync-sso';
import {
    stringToBase64,
    base64urlToBytes,
    bytesToBase64
} from './utils';
import { authenticate_passkey } from './passkey_utils';

/**
 * Authenticates a user using their platform passkey and returns the authentication data.
 * This function handles platform-specific authentication logic by delegating to the appropriate
 * implementation based on the current platform.
 * 
 * @param message - The challenge message to authenticate against, as an ArrayBuffer
 * @param rpId - The relying party ID used for passkey authentication
 * @returns A Promise that resolves to an ArrayBuffer containing the encoded authentication payload
 */
export const authenticateWithPasskey = async (
    message: ArrayBuffer,
    rpId: RpId
): Promise<ArrayBuffer> => {
    const result = await authenticate_passkey(message, rpId);
    return processAuthenticationResult(result);
};

/**
 * Processes the PasskeyGetResult and returns the encoded authentication payload
 */
export const processAuthenticationResult = (result: PasskeyGetResult): ArrayBuffer => {
    type attachment = "platform" | "crossPlatform";

    interface AuthorizationPlatformPublicKeyCredentialAssertion {
        attachment: attachment;
        rawAuthenticatorData: string;
        userID: string;
        signature: string;
        credentialID: string;
        rawClientDataJSON: string;
    }

    const attachment: attachment = "platform";

    // Process rawAuthenticatorData
    const rawAuthDataBytes = base64urlToBytes(result.response.authenticatorData);
    const rawAuthenticatorData = bytesToBase64(rawAuthDataBytes);

    // Process userID
    const userID = result.response.userHandle ? stringToBase64(result.response.userHandle) : '';

    // Process signature
    const signatureBytes = base64urlToBytes(result.response.signature);
    const signature = bytesToBase64(signatureBytes);
    const credentialIDBytes = base64urlToBytes(result.id);
    const credentialID = bytesToBase64(credentialIDBytes);
    const rawClientDataBytes = base64urlToBytes(result.response.clientDataJSON);
    const rawClientDataJSON = bytesToBase64(rawClientDataBytes);
    const payload: AuthorizationPlatformPublicKeyCredentialAssertion = {
        attachment,
        rawAuthenticatorData,
        userID,
        signature,
        credentialID,
        rawClientDataJSON
    };

    const payloadJson = JSON.stringify(payload);

    // Ensure we encode the JSON string with UTF-8
    const encoder = new TextEncoder();
    const payloadBuffer = encoder.encode(payloadJson);

    return payloadBuffer.buffer as ArrayBuffer;
};