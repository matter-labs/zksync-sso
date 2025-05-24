import type { Config, Account, RpId as FfiRpId } from '../../../src';
import { Platform } from 'react-native';

export interface AndroidRpId {
    rp_id: string;
    origin: string;
}

export type RpId =
    | { platform: 'apple'; value: string }
    | { platform: 'android'; value: AndroidRpId };

export class RpIdHelper {
    static createApple(rpId: string): RpId {
        return { platform: 'apple', value: rpId };
    }

    static createAndroid(rpId: string, origin: string): RpId {
        return { platform: 'android', value: { rp_id: rpId, origin } };
    }

    static origin(rpId: RpId): string {
        switch (rpId.platform) {
            case 'apple':
                return rpId.value;
            case 'android':
                return rpId.value.origin;
        }
    }

    static rpId(rpId: RpId): string {
        switch (rpId.platform) {
            case 'apple':
                return rpId.value;
            case 'android':
                return rpId.value.rp_id;
        }
    }

    static createForCurrentPlatform(rpId: string, androidOrigin: string): RpId {
        return Platform.OS === 'ios'
            ? RpIdHelper.createApple(rpId)
            : RpIdHelper.createAndroid(rpId, androidOrigin);
    }

    /**
     * Converts our local RpId type to the FFI RpId type
     */
    static toFfiRpId(rpId: RpId): FfiRpId {
        // Import the FFI RpId constructors
        const { RpId: FfiRpIdClass } = require('../../../src');

        switch (rpId.platform) {
            case 'apple':
                return FfiRpIdClass.Apple.new(rpId.value);
            case 'android':
                return FfiRpIdClass.Android.new({
                    rpId: rpId.value.rp_id,
                    origin: rpId.value.origin
                });
        }
    }
}

export interface AccountInfo {
    name: string;
    userID: string;
    rpId: RpId;
}

export interface DeployedAccount {
    info: AccountInfo;
    address: string;
    uniqueAccountId: string;
}

export interface AccountDetails {
    info: AccountInfo;
    address: string;
    shortAddress: string;
    uniqueAccountId: string;
    explorerURL: string;
    balance?: string;
}

export function createAccountDetails(
    accountInfo: AccountInfo,
    deployedAccount: Account,
    balance?: string
): AccountDetails {
    const address = deployedAccount.address;
    return {
        info: accountInfo,
        address,
        shortAddress: shortenAddress(address),
        uniqueAccountId: deployedAccount.uniqueAccountId,
        explorerURL: `https://explorer.zksync.io/address/${address}`,
        balance
    };
}

function shortenAddress(address: string): string {
    if (!address || address.length < 10) return address;
    return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
}

export type { Config };