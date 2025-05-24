import {
    Passkey,
    type PasskeyCreateResult,
    type PasskeyCreateRequest,
    type PasskeyGetRequest,
    type PasskeyGetResult
} from 'react-native-passkey';
import { Platform } from 'react-native';
// @ts-ignore
import { type RpId } from 'react-native-zksync-sso';
import {
    type AccountInfo,
    getRpIdString,
    arrayBufferToBase64Url
} from './utils';
import { getCredentials } from './credentialStore';

/**
 * Registers a passkey for the given account info, using platform-specific logic
 */
export const register_passkey = async (
    challenge: string,
    accountInfo: AccountInfo
): Promise<PasskeyCreateResult> => {
    if (Platform.OS === 'ios') {
        return await register_passkey_apple(challenge, accountInfo);
    } else {
        return await register_passkey_android(challenge, accountInfo);
    }
};

/**
 * Apple-specific passkey registration
 */
export const register_passkey_apple = async (
    challenge: string,
    accountInfo: AccountInfo
): Promise<PasskeyCreateResult> => {
    console.log("register_passkey_apple - accountInfo: ", accountInfo);

    const rpId = getRpIdString(accountInfo.rp.id);
    console.log("register_passkey_apple - rpId: ", rpId);

    const rpName = accountInfo.rp.name;
    console.log("register_passkey_apple - name: ", rpName);

    const userId = accountInfo.userID;
    console.log("register_passkey_apple - userId: ", userId);

    const userName = accountInfo.name;
    console.log("register_passkey_apple - userName: ", userName);

    const displayName = accountInfo.name;
    console.log("register_passkey_apple - displayName: ", displayName);

    const requestJson: PasskeyCreateRequest = {
        challenge: challenge,
        rp: {
            id: rpId,
            name: rpName
        },
        user: {
            id: userId,
            name: userName,
            displayName: displayName
        },
        pubKeyCredParams: []
    };
    console.log("register_passkey_apple - requestJson: ", requestJson);

    const passkeyCreationResult: PasskeyCreateResult = await Passkey.createPlatformKey(requestJson);

    console.log("register_passkey_apple - passkeyCreationResult: ", passkeyCreationResult);
    console.log("register_passkey_apple - passkeyCreationResult json: ", JSON.stringify(passkeyCreationResult, null, 2));
    
    return passkeyCreationResult;
};

/**
 * Android-specific passkey registration
 */
export const register_passkey_android = async (
    challenge: string,
    accountInfo: AccountInfo
): Promise<PasskeyCreateResult> => {
    console.log("register_passkey_android - accountInfo: ", accountInfo);

    const rpId = getRpIdString(accountInfo.rp.id);
    console.log("register_passkey_android - rpId: ", rpId);

    const rpName = accountInfo.rp.name;
    console.log("register_passkey_android - name: ", rpName);

    const userId = accountInfo.userID;
    console.log("register_passkey_android - userId: ", userId);

    const userName = accountInfo.name;
    console.log("register_passkey_android - userName: ", userName);

    const displayName = accountInfo.name;
    console.log("register_passkey_android - displayName: ", displayName);

    const allowCredentials = getCredentials().map(cred => ({
        id: cred.id,
        type: cred.type,
        transports: cred.transports.map(t => t)
    }));

    console.log("register_passkey_android - allowCredentials: ", allowCredentials);

    const requestJson: PasskeyCreateRequest = {
        challenge: challenge,
        rp: {
            id: rpId,
            name: rpName
        },
        user: {
            id: userId,
            name: userName,
            displayName: displayName
        },
        pubKeyCredParams: [
            {
                type: "public-key",
                alg: -7
            }
        ],
        timeout: 1800000,
        attestation: "none",
        excludeCredentials: [],
        authenticatorSelection: {
            residentKey: "required",
            userVerification: "preferred"
        }
    };

    console.log("register_passkey_android - requestJson: ", requestJson);
    const passkeyCreationResult: PasskeyCreateResult = await Passkey.create(requestJson);
    console.log("register_passkey_android - passkeyCreationResult: ", passkeyCreationResult);
    console.log("register_passkey_android - passkeyCreationResult json: ", JSON.stringify(passkeyCreationResult, null, 2));

    return passkeyCreationResult;
};

/**
 * Authenticates with a passkey using platform-specific logic
 */
export const authenticate_passkey = async (
    message: ArrayBuffer,
    rpId: RpId
): Promise<PasskeyGetResult> => {
    if (Platform.OS === 'ios') {
        return await authenticate_passkey_apple(message, rpId);
    } else {
        return await authenticate_passkey_android(message, rpId);
    }
};

/**
 * Apple-specific passkey authentication
 */
export const authenticate_passkey_apple = async (
    message: ArrayBuffer,
    rpId: RpId
): Promise<PasskeyGetResult> => {
    console.log("authenticate_passkey_apple message:", message);
    console.log("authenticate_passkey_apple rpId:", rpId);

    const challenge = arrayBufferToBase64Url(message);

    const rpIdString = getRpIdString(rpId);
    console.log("authenticate_passkey_apple - rpIdString: ", rpIdString);

    const requestJson: PasskeyGetRequest = {
        challenge: challenge,
        rpId: rpIdString,
    };

    console.log("authenticate_passkey_apple - requestJson: ", requestJson);

    const result: PasskeyGetResult = await Passkey.get(requestJson);
    console.log("authenticate_passkey_apple result:", result);
    console.log("authenticate_passkey_apple result json:", JSON.stringify(result, null, 2));

    return result;
};

/**
 * Android-specific passkey authentication
 */
export const authenticate_passkey_android = async (
    message: ArrayBuffer,
    rpId: RpId
): Promise<PasskeyGetResult> => {
    console.log("authenticate_passkey_android message:", message);
    console.log("authenticate_passkey_android rpId:", rpId);

    const challenge = arrayBufferToBase64Url(message);

    const allowCredentials = getCredentials().map(cred => ({
        id: cred.id,
        type: cred.type,
        transports: cred.transports.map(t => t)
    }));

    console.log("authenticate_passkey_android - allowCredentials: ", allowCredentials);

    const rpIdString = getRpIdString(rpId);
    console.log("authenticate_passkey_android - rpIdString: ", rpIdString);

    const requestJson: PasskeyGetRequest = {
        challenge: challenge,
        rpId: rpIdString,
        timeout: 1800000,
        userVerification: "required",
        allowCredentials
    };

    console.log("authenticate_passkey_android - requestJson: ", requestJson);

    const result: PasskeyGetResult = await Passkey.get(requestJson);
    console.log("authenticate_passkey_android result:", result);
    console.log("authenticate_passkey_android result json:", JSON.stringify(result, null, 2));

    return result;
};

