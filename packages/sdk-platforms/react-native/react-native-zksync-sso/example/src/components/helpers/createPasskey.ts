import { loadConfig } from './loadConfig';
import sdk from '../../../../src';
import type { AccountInfo, DeployedAccount } from '../types';

/**
 * Creates a passkey for the given account info
 * @param accountInfo Account info to create a passkey for
 * @returns Deployed account information
 */
export const createPasskey = async (
    accountInfo: AccountInfo
): Promise<DeployedAccount> => {
    const config = loadConfig();
    const challenge = sdk.ffi.generateRandomChallenge();
    const account = await sdk.register.registerAccountWithUniqueId(
        {
            name: accountInfo.name,
            userID: accountInfo.userID,
            rp: {
                name: accountInfo.domain,
                id: accountInfo.domain
            }
        },
        challenge,
        config
    );
    console.log("Deployed account:", account);
    return {
        info: accountInfo,
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    };
};