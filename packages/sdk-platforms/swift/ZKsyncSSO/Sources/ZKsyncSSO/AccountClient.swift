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
    
    public func fundAccount() async throws {
        try await ZKsyncSSOFFI.fundAccount(
            address: account.address,
            config: Config.default.inner
        )
    }
    
    public func sendTransaction(
        to: String,
        amount: String
    ) async throws {
        let tx = Transaction(
            from: account.address,
            to: to,
            value: amount,
            input: nil
        )
        let result = try await ZKsyncSSOFFI.sendTransactionAsyncSigner(
            transaction: tx,
            authenticator: authenticatorAsync,
            config: Config.default.inner
        )
        print(result)
    }
    
    public func prepareTransaction(
        transaction: TransactionRequest
    ) async throws -> PreparedTransaction {
        let from = account.address
        let tx = try await ZKsyncSSOFFI.prepareSendTransaction(
            transaction: transaction.inner,
            from: from,
            config: Config.default.inner
        )
        return tx.wrappedValue
    }
}
