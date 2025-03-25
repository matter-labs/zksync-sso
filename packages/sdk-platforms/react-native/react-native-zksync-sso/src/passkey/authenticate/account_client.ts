// @ts-ignore
import {
    type Config,
    type Account,
    type PreparedTransaction,
    type Transaction,
    type SendTransactionResult,
    sendTransactionAsyncSigner,
    prepareSendTransaction,
} from 'react-native-zksync-sso';
import { Authenticator } from './authenticator';
export { type PreparedTransaction };

/**
 * Helper class for account operations like transaction preparation and sending
 */
export class AccountClient {
    private account: Account;
    private rpId: string;
    private config: Config;

    constructor(account: Account, rpId: string, config: Config) {
        this.account = account;
        this.rpId = rpId;
        this.config = config;
    }

    /**
     * Prepares a transaction for sending
     * @param transaction The transaction to prepare
     * @returns Prepared transaction with fee information
     */
    async prepareTransaction(to: string, value: string): Promise<PreparedTransaction> {
        const from = this.account.address;
        const transaction: Transaction = {
            to,
            value,
            from,
        };
        console.log("transaction: ", transaction);
        const preparedTransaction = await prepareSendTransaction(
            transaction,
            from,
            this.config
        );
        console.log("preparedTransaction: ", preparedTransaction);
        return preparedTransaction;
    }

    /**
     * Sends a transaction
     * @param to Recipient address
     * @param amount Amount in wei as string
     * @returns Transaction hash
     */
    async sendTransaction(to: string, value: string): Promise<SendTransactionResult> {
        const authenticator = new Authenticator(this.rpId);
        const prepared = await this.prepareTransaction(to, value);
        const result = await sendTransactionAsyncSigner(
            prepared,
            authenticator,
            this.config
        );
        console.log("result: ", result);
        return result;
    }
}