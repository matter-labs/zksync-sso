// @ts-ignore
import {
    type Config,
    type Account,
    type PasskeyParameters,
    deployAccountWithUniqueId,
    deployAccount
} from 'react-native-zksync-sso';
import {
    Passkey,
    type PasskeyCreateResult,
    type PasskeyCreateRequest
} from 'react-native-passkey';
import {
    base64ToArrayBuffer
} from './utils';

/**
 * Information about the relying party (RP) for passkey registration
 */
export interface RPInfo {
    name: string;
    id: string;
}

/**
 * Information about the account being registered
 */
export interface AccountInfo {
    name: string;
    userID: string;
    rp: RPInfo;
}

/**
 * Registers a new account using a platform passkey and deploys it.
 * This function handles the creation of a new passkey and the deployment of the account
 * with the generated credentials.
 * 
 * @param accountInfo - Information about the account to register
 * @param challenge - Challenge string for passkey creation
 * @param config - Configuration for deployment
 * @returns A Promise that resolves to the deployed Account
 */
export const registerAccountWithUniqueId = async (
    accountInfo: AccountInfo,
    challenge: string,
    config: Config
): Promise<Account> => {
    const requestJson: PasskeyCreateRequest = {
        challenge: challenge,
        rp: accountInfo.rp,
        user: {
            id: accountInfo.userID,
            name: accountInfo.name,
            displayName: accountInfo.name
        },
        pubKeyCredParams: [],
    };
    const result: PasskeyCreateResult = await Passkey.createPlatformKey(
        requestJson
    );
    console.log("result: ", result);

    const rpId = accountInfo.rp.id;
    const uniqueAccountId = accountInfo.userID;
    const credentialRawAttestationObject = base64ToArrayBuffer(
        result.response.attestationObject
    );
    const credentialRawClientDataJson = base64ToArrayBuffer(
        result.response.clientDataJSON
    );
    const credentialId = base64ToArrayBuffer(result.id);
    const passkeyParameters: PasskeyParameters = {
        credentialRawAttestationObject,
        credentialRawClientDataJson,
        credentialId,
        rpId,
    };
    const deployedAccount: Account = await deployAccountWithUniqueId(
        passkeyParameters,
        uniqueAccountId,
        config,
    );
    console.log("Deployed account:", deployedAccount);
    return deployedAccount;
};

export const registerAccount = async (
    accountInfo: AccountInfo,
    challenge: string,
    config: Config
): Promise<Account> => {
    const requestJson: PasskeyCreateRequest = {
        challenge: challenge,
        rp: accountInfo.rp,
        user: {
            id: accountInfo.userID,
            name: accountInfo.name,
            displayName: accountInfo.name
        },
        pubKeyCredParams: [],
    };
    const result: PasskeyCreateResult = await Passkey.createPlatformKey(
        requestJson
    );
    console.log("result: ", result);

    const rpId = accountInfo.rp.id;
    const uniqueAccountId = accountInfo.userID;
    const credentialRawAttestationObject = base64ToArrayBuffer(
        result.response.attestationObject
    );
    const credentialRawClientDataJson = base64ToArrayBuffer(
        result.response.clientDataJSON
    );
    const credentialId = base64ToArrayBuffer(
        result.id
    );
    const passkeyParameters: PasskeyParameters = {
        credentialRawAttestationObject,
        credentialRawClientDataJson,
        credentialId,
        rpId,
    };
    const deployedAccount: Account = await deployAccount(
        passkeyParameters,
        config,
    );
    console.log("Deployed account:", deployedAccount);
    return deployedAccount;
};