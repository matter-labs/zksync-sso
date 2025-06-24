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
    const rpId = getRpIdString(accountInfo.rp.id);
    const rpName = accountInfo.rp.name;
    const userId = accountInfo.userID;
    const userName = accountInfo.name;
    const displayName = accountInfo.name;
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
    return await Passkey.createPlatformKey(requestJson);
};

/**
 * Android-specific passkey registration
 */
export const register_passkey_android = async (
    challenge: string,
    accountInfo: AccountInfo
): Promise<PasskeyCreateResult> => {
    const rpId = getRpIdString(accountInfo.rp.id);
    const rpName = accountInfo.rp.name;
    const userId = accountInfo.userID;
    const userName = accountInfo.name;
    const displayName = accountInfo.name;
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
    return await Passkey.create(requestJson);
};

/**
 * Authenticates with a passkey
 */
export const authenticate_passkey = async (
    message: ArrayBuffer,
    rpId: RpId
): Promise<PasskeyGetResult> => {
    const challenge = arrayBufferToBase64Url(message);
    const rpIdString = getRpIdString(rpId);
    const requestJson: PasskeyGetRequest = {
        challenge: challenge,
        rpId: rpIdString,
    };
    return await Passkey.get(requestJson);
};
