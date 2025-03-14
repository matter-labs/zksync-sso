import Foundation
import ZKsyncSSOFFI

public struct AccountClient: Sendable {
    
    public let authenticator: any PasskeyAuthenticator & Sendable
    
    public let account: Account
    
    public init(
        account: Account,
        authenticator: any PasskeyAuthenticator & Sendable
    ) {
        self.account = account
        self.authenticator = authenticator
    }
    
    public func getAccountBalance() async throws -> String {
        let accountBalance = try await ZKsyncSSOFFI.getBalance(
            address: account.address, config: Config.default.inner
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
            to: to,
            value: amount,
            from: account.address
        )
        let result = try await ZKsyncSSOFFI.sendTransaction(
            transaction: tx,
            authenticator: authenticator,
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
