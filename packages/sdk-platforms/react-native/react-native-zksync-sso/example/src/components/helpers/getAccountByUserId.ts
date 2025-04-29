import { type Account, getAccountByUserId } from '../../../../src';
import { createSecretAccountSalt } from './createSecretAccountSalt';
import { loadConfig } from './loadConfig';

export const getAccountByUserIdWrapper = async (
    uniqueAccountId: string
): Promise<Account> => {
    const config = loadConfig();
    const secretAccountSalt = createSecretAccountSalt();
    const account: Account = await getAccountByUserId(
        uniqueAccountId,
        secretAccountSalt,
        config
    );
    console.log('Account implementation returned:', account);
    return account;
}; 