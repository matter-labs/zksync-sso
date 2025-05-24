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
    console.log("authenticateWithPasskey - delegating to platform-specific implementation");
    console.log("authenticateWithPasskey message:", message);
    console.log("authenticateWithPasskey rpId:", rpId);
    console.log("authenticateWithPasskey rpId type:", typeof rpId);
    console.log("authenticateWithPasskey rpId JSON:", JSON.stringify(rpId, null, 2));

    const result = await authenticate_passkey(message, rpId);
    console.log("authenticateWithPasskey - processing authentication result");

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
    console.log("processAuthenticationResult - Original rawAuthenticatorData (base64url): ", result.response.authenticatorData);
    const rawAuthDataBytes = base64urlToBytes(result.response.authenticatorData);
    console.log("processAuthenticationResult - Decoded rawAuthenticatorData bytes: ", Array.from(rawAuthDataBytes).slice(0, 10), "...");
    const rawAuthenticatorData = bytesToBase64(rawAuthDataBytes);
    console.log("processAuthenticationResult - Re-encoded rawAuthenticatorData (standard base64): ", rawAuthenticatorData);

    // Process userID
    const userID = result.response.userHandle ? stringToBase64(result.response.userHandle) : '';
    console.log("processAuthenticationResult - Encoded userID: ", userID);

    // Process signature
    console.log("processAuthenticationResult - Original signature (base64url): ", result.response.signature);
    const signatureBytes = base64urlToBytes(result.response.signature);
    console.log("processAuthenticationResult - Decoded signature bytes: ", Array.from(signatureBytes).slice(0, 10), "...");
    const signature = bytesToBase64(signatureBytes);
    console.log("processAuthenticationResult - Re-encoded signature (standard base64): ", signature);

    // Process credentialID
    console.log("processAuthenticationResult - Original credentialID (base64url): ", result.id);
    const credentialIDBytes = base64urlToBytes(result.id);
    console.log("processAuthenticationResult - Decoded credentialID bytes: ", Array.from(credentialIDBytes).slice(0, 10), "...");
    const credentialID = bytesToBase64(credentialIDBytes);
    console.log("processAuthenticationResult - Re-encoded credentialID (standard base64): ", credentialID);

    // Process rawClientDataJSON
    console.log("processAuthenticationResult - Original rawClientDataJSON (base64url): ", result.response.clientDataJSON);
    const rawClientDataBytes = base64urlToBytes(result.response.clientDataJSON);
    console.log("processAuthenticationResult - Decoded rawClientDataJSON bytes: ", Array.from(rawClientDataBytes).slice(0, 10), "...");
    const rawClientDataJSON = bytesToBase64(rawClientDataBytes);
    console.log("processAuthenticationResult - Re-encoded rawClientDataJSON (standard base64): ", rawClientDataJSON);

    const payload: AuthorizationPlatformPublicKeyCredentialAssertion = {
        attachment,
        rawAuthenticatorData,
        userID,
        signature,
        credentialID,
        rawClientDataJSON
    };

    const payloadJson = JSON.stringify(payload);
    console.log("processAuthenticationResult - Encoded payload:", payloadJson);

    // Ensure we encode the JSON string with UTF-8
    const encoder = new TextEncoder();
    const payloadBuffer = encoder.encode(payloadJson);
    console.log("processAuthenticationResult - Encoded bytes (first 20):", Array.from(payloadBuffer).slice(0, 20), "...");

    return payloadBuffer.buffer as ArrayBuffer;
};