import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct SessionClient {
    public var accountAddress: String
    public var sessionKey: String
    public var sessionConfig: String
    public var config: Config

    public init(
        accountAddress: String,
        sessionKey: String,
        sessionConfig: String,
        config: Config
    ) {
        self.accountAddress = accountAddress
        self.sessionKey = sessionKey
        self.sessionConfig = sessionConfig
        self.config = config
    }

    @discardableResult
    public func sendTransaction(
        transaction: TransactionRequest
    ) async throws -> String {
        try await sendSessionTransaction(
            accountAddress: accountAddress,
            sessionKeyHex: sessionKey,
            sessionConfigJson: sessionConfig,
            config: config.inner,
            transaction: Transaction(
                from: accountAddress,
                to: transaction.to,
                value: transaction.value,
                input: transaction.input
            )
        )
    }

    @discardableResult
    public func revokeSession() async throws -> String {
        //    try await ZKsyncSSOFFI.revokeSession(
        //      accountAddress: accountAddress,
        //      sessionKeyHex: sessionKey,
        //      sessionConfigJson: sessionConfig,
        //      config: config.inner
        //    )
        fatalError()
    }
}
