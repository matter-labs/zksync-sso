import type { Config, Account } from '../../../src';

export interface AccountInfo {
    name: string;
    userID: string;
    domain: string;
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