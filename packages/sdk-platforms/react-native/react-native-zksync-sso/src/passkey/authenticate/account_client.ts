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
    async prepareTransaction(transaction: Transaction): Promise<PreparedTransaction> {
        const from = this.account.address;
        const transaction: Transaction = {
            to,
            value,
            from,
        };
        const preparedTransaction = await prepareSendTransaction(
            transaction,
            from,
            this.config
        );
        return preparedTransaction;
    }

    /**
     * Sends a transaction
     * @param transaction The transaction to send
     * @returns Transaction hash
     */
    async sendTransaction(to: Transaction): Promise<SendTransactionResult> {
        const authenticator = new Authenticator(this.rpId);
        const result = await sendTransactionAsyncSigner(
            prepared,
            authenticator,
            this.config
        );
        console.log("result: ", result);
        return result;
    }
}