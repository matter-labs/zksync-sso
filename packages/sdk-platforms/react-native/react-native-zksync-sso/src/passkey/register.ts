// @ts-ignore
import {
    type Config,
    type Account,
    type PasskeyParameters,
    type RpId,
    deployAccountWithUniqueId,
    deployAccount
} from 'react-native-zksync-sso';
import {
    Passkey,
    type PasskeyGetRequest,
    type PasskeyGetResult,
    type PasskeyCreateResult,
    type PasskeyCreateRequest
} from 'react-native-passkey';
import { Platform } from 'react-native';
import {
    base64ToArrayBuffer,
    arrayBufferToHexString,
    type RPInfo,
    type AccountInfo,
    getRpIdString
} from './utils';
import { addCredential, getCredentials } from './credentialStore';
import { register_passkey } from './passkey_utils';

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
    console.log("registerAccountWithUniqueId - accountInfo: ", accountInfo);
    console.log("registerAccountWithUniqueId - challenge: ", challenge);
    console.log("registerAccountWithUniqueId - config: ", config);

    const passkeyCreationResult = await register_passkey(challenge, accountInfo);
    console.log("registerAccountWithUniqueId - passkeyCreationResult: ", passkeyCreationResult);

    addCredential(
        {
            id: passkeyCreationResult.id,
            type: passkeyCreationResult.type,
            transports: passkeyCreationResult.response.transports
        }
    );

    const rpId = accountInfo.rp.id;

    const uniqueAccountId = accountInfo.userID;
    const credentialRawAttestationObject: ArrayBuffer = base64ToArrayBuffer(
        passkeyCreationResult.response.attestationObject
    );
    console.log("registerAccountWithUniqueId - credentialRawAttestationObject (hex): ", arrayBufferToHexString(credentialRawAttestationObject));
    const credentialRawClientDataJson: ArrayBuffer = base64ToArrayBuffer(
        passkeyCreationResult.response.clientDataJSON
    );
    console.log("registerAccountWithUniqueId - credentialRawClientDataJson (hex): ", arrayBufferToHexString(credentialRawClientDataJson));
    const credentialId: ArrayBuffer = base64ToArrayBuffer(passkeyCreationResult.id);
    console.log("registerAccountWithUniqueId - credentialId (hex): ", arrayBufferToHexString(credentialId));

    const passkeyParameters: PasskeyParameters = {
        credentialRawAttestationObject,
        credentialRawClientDataJson,
        credentialId,
        rpId,
    };
    console.log("registerAccountWithUniqueId - passkeyParameters: ", JSON.stringify({
        ...passkeyParameters,
        credentialRawAttestationObject: arrayBufferToHexString(passkeyParameters.credentialRawAttestationObject),
        credentialRawClientDataJson: arrayBufferToHexString(passkeyParameters.credentialRawClientDataJson),
        credentialId: arrayBufferToHexString(passkeyParameters.credentialId),
    }, null, 2));
    console.log("registerAccountWithUniqueId - uniqueAccountId: ", uniqueAccountId);
    console.log("registerAccountWithUniqueId - config (full): ", JSON.stringify(config, null, 2));

    const deployedAccount: Account = await deployAccountWithUniqueId(
        passkeyParameters,
        uniqueAccountId,
        config,
    );
    console.log("registerAccountWithUniqueId - deployedAccount: ", deployedAccount);
    return deployedAccount;
};

export const registerAccount = async (
    accountInfo: AccountInfo,
    challenge: string,
    config: Config
): Promise<Account> => {
    const rpId = accountInfo.rp.id;

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
    console.log("registerAccount - result: ", result);
    console.log("registerAccount - result json: ", JSON.stringify(result, null, 2));

    addCredential(
        {
            id: result.id,
            type: result.type,
            transports: result.response.transports
        }
    );

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