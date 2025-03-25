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

export interface RPInfo {
    name: string;
    id: string;
}

export interface AccountInfo {
    name: string;
    userID: string;
    rp: RPInfo;
}

export const registerAccountWithUniqueId = async (
    accountInfo: AccountInfo,
    secretAccountSalt: string,
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
        secretAccountSalt,
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