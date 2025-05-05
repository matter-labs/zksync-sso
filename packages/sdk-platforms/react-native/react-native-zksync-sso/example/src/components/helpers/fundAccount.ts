import { loadConfig } from './loadConfig';
import { fundAccount as sdkFundAccount } from '../../../../src';

/**
 * Funds an account with test ETH
 * @param address The account address to fund
 * @returns The new balance after funding
 */
export const fundAccount = async (address: string): Promise<void> => {
    const config = loadConfig();
    await sdkFundAccount(address, config);
}; 