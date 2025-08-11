import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct GetSessionStateArgs {
    let inner: ZKsyncSSOFFI.GetSessionStateArgs

    public init(account: String, sessionConfig: String) {
        inner = ZKsyncSSOFFI.GetSessionStateArgs(
            account: account,
            sessionConfig: sessionConfig
        )
    }
}

public func getSessionState(args: GetSessionStateArgs, config: Config) async throws -> String {
    let args = args.inner
    let result = try await ZKsyncSSOFFI.getSessionState(
        args: args,
        config: config.inner
    )
    return result.sessionStateJson
}
