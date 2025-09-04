import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct AccountClient: Sendable {
    public let authenticatorAsync: any PasskeyAuthenticatorAsync & Sendable

    public let account: Account

    public init(
        account: Account,
        authenticatorAsync: any PasskeyAuthenticatorAsync & Sendable
    ) {
        self.account = account
        self.authenticatorAsync = authenticatorAsync
    }

    public func getAccountBalance() async throws -> String {
        let accountBalance = try await ZKsyncSSOFFI.getBalance(
            address: account.address,
            config: Config.default.inner
        )
        return accountBalance.balance
    }

    public func fundAccount(amount: String) async throws {
        try await ZKsyncSSOFFI.fundAccount(
            address: account.address,
            amount: amount,
            config: Config.default.inner
        )
    }

    @discardableResult
    public func send(
        transaction: TransactionRequest
    ) async throws -> String {
        let tx = Transaction.from(
            request: transaction,
            account: account.address
        )
        let result = try await ZKsyncSSOFFI.sendTransactionAsyncSigner(
            transaction: tx,
            authenticator: authenticatorAsync,
            config: Config.default.inner
        )
        return result.txHash
    }

    public func prepare(
        transaction: TransactionRequest
    ) async throws -> PreparedTransaction {
        let tx = Transaction.from(
            request: transaction,
            account: account.address
        )
        let preparedTransaction = try await ZKsyncSSOFFI.prepareSendTransaction(
            transaction: tx,
            config: Config.default.inner
        )
        return preparedTransaction.wrappedValue
    }
}
