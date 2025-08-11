import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct RevokeSessionArgs {
    let inner: ZKsyncSSOFFI.RevokeSessionArgs

    public init(account: String, sessionId: String, ownerPrivateKey: String) {
        inner = ZKsyncSSOFFI.RevokeSessionArgs(
            account: account,
            sessionId: sessionId,
            ownerPrivateKey: ownerPrivateKey
        )
    }
}

public func revokeSession(args: RevokeSessionArgs, config: Config) async throws -> String {
    let args = args.inner
    let createdSession = try await ZKsyncSSOFFI.revokeSession(
        args: args,
        config: config.inner
    )
    return createdSession.transactionReceiptJson
}
