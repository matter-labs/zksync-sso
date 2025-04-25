import {
    Passkey,
    type PasskeyGetRequest,
    type PasskeyGetResult
} from 'react-native-passkey';
import {
    stringToBase64,
    base64urlToBytes,
    bytesToBase64,
    arrayBufferToBase64Url
} from './utils';

/**
 * Authenticates a user using their platform passkey and returns the authentication data.
 * 
 * @param message - The challenge message to authenticate against, as an ArrayBuffer
 * @param rpId - The relying party ID used for passkey authentication
 * @returns A Promise that resolves to an ArrayBuffer containing the encoded authentication payload
 */
export const authenticateWithPasskey = async (
    message: ArrayBuffer,
    rpId: string
): Promise<ArrayBuffer> => {
    console.log("authenticateWithPasskey message:", message);

    const challenge = arrayBufferToBase64Url(message);

    const requestJson: PasskeyGetRequest = {
        challenge: challenge,
        rpId: rpId,
    };

    const result: PasskeyGetResult = await Passkey.get(requestJson);

    console.log("authenticateWithPasskey result:", result);

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
    console.log("Original rawAuthenticatorData (base64url):", result.response.authenticatorData);
    const rawAuthDataBytes = base64urlToBytes(result.response.authenticatorData);
    console.log("Decoded rawAuthenticatorData bytes:", Array.from(rawAuthDataBytes).slice(0, 10), "...");
    const rawAuthenticatorData = bytesToBase64(rawAuthDataBytes);
    console.log("Re-encoded rawAuthenticatorData (standard base64):", rawAuthenticatorData);

    // Process userID
    const userID = result.response.userHandle ? stringToBase64(result.response.userHandle) : '';
    console.log("Encoded userID:", userID);

    // Process signature
    console.log("Original signature (base64url):", result.response.signature);
    const signatureBytes = base64urlToBytes(result.response.signature);
    console.log("Decoded signature bytes:", Array.from(signatureBytes).slice(0, 10), "...");
    const signature = bytesToBase64(signatureBytes);
    console.log("Re-encoded signature (standard base64):", signature);

    // Process credentialID
    console.log("Original credentialID (base64url):", result.id);
    const credentialIDBytes = base64urlToBytes(result.id);
    console.log("Decoded credentialID bytes:", Array.from(credentialIDBytes).slice(0, 10), "...");
    const credentialID = bytesToBase64(credentialIDBytes);
    console.log("Re-encoded credentialID (standard base64):", credentialID);

    // Process rawClientDataJSON
    console.log("Original rawClientDataJSON (base64url):", result.response.clientDataJSON);
    const rawClientDataBytes = base64urlToBytes(result.response.clientDataJSON);
    console.log("Decoded rawClientDataJSON bytes:", Array.from(rawClientDataBytes).slice(0, 10), "...");
    const rawClientDataJSON = bytesToBase64(rawClientDataBytes);
    console.log("Re-encoded rawClientDataJSON (standard base64):", rawClientDataJSON);

    const payload: AuthorizationPlatformPublicKeyCredentialAssertion = {
        attachment,
        rawAuthenticatorData,
        userID,
        signature,
        credentialID,
        rawClientDataJSON
    };

    const payloadJson = JSON.stringify(payload);
    console.log("Encoded payload:", payloadJson);

    // Ensure we encode the JSON string with UTF-8
    const encoder = new TextEncoder();
    const payloadBuffer = encoder.encode(payloadJson);
    console.log("Encoded bytes (first 20):", Array.from(payloadBuffer).slice(0, 20), "...");

    return payloadBuffer.buffer as ArrayBuffer;
};