import { type Account, getAccountByUserId } from '../../../../src';
import { loadConfig } from './loadConfig';

export const getAccountByUserIdWrapper = async (
    uniqueAccountId: string
): Promise<Account> => {
    const config = loadConfig();
    const account: Account = await getAccountByUserId(
        uniqueAccountId,
        config
    );
    console.log('Account implementation returned:', account);
    return account;
}; 