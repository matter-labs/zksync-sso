import Foundation
@preconcurrency import ZKsyncSSOFFI

public enum GetSessionStateError: Error {
    case decodingError(String)
}

public struct GetSessionStateArgs {
    let inner: ZKsyncSSOFFI.GetSessionStateArgs

    public init(account: String, sessionConfig: String) {
        inner = ZKsyncSSOFFI.GetSessionStateArgs(
            account: account,
            sessionConfig: sessionConfig
        )
    }
}

public func getSessionState(args: GetSessionStateArgs, config: Config) async throws -> SessionState {
    let args = args.inner
    let result = try await ZKsyncSSOFFI.getSessionState(
        args: args,
        config: config.inner
    )
    guard let data = result.sessionStateJson.data(using: .utf8) else {
        throw GetSessionStateError.decodingError("Failed to convert session state JSON to data")
    }
    do {
        return try JSONDecoder().decode(SessionState.self, from: data)
    } catch {
        throw GetSessionStateError.decodingError("Failed to decode SessionState: \(error)")
    }
}
