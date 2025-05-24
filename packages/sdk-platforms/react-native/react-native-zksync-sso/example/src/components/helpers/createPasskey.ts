import { loadConfig } from './loadConfig';
import sdk from '../../../../src';
import type { RpId } from '../../../../src';
import type { AccountInfo, DeployedAccount } from '../types';
import { RpIdHelper } from '../types';

/**
 * Creates a passkey for the given account info
 * @param accountInfo Account info to create a passkey for
 * @returns Deployed account information
 */
export const createPasskey = async (
    accountInfo: AccountInfo
): Promise<DeployedAccount> => {
    console.log("createPasskey - accountInfo:", accountInfo);
    const config = loadConfig();
    console.log("createPasskey - config:", config);
    const challenge = sdk.ffi.generateRandomChallenge();
    console.log("createPasskey - challenge:", challenge);
    const account = await sdk.register.registerAccountWithUniqueId(
        {
            name: accountInfo.name,
            userID: accountInfo.userID,
            rp: {
                name: RpIdHelper.rpId(accountInfo.rpId),
                id: RpIdHelper.toFfiRpId(accountInfo.rpId)
            }
        },
        challenge,
        config
    );
    console.log("createPasskey - account deployed:", account);
    return {
        info: accountInfo,
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    };
};