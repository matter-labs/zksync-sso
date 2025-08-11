import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct CreateSessionArgs {
    let inner: ZKsyncSSOFFI.CreateSessionArgs

    public init(account: String, sessionConfigJSON: String, ownerPrivateKey: String, paymaster: String?) {
        inner = ZKsyncSSOFFI.CreateSessionArgs(
            account: account,
            sessionConfig: sessionConfigJSON,
            ownerPrivateKey: ownerPrivateKey,
            paymaster: paymaster
        )
    }

    public init(account: String, sessionConfig: SessionSpec, ownerPrivateKey: String, paymaster: String?) {
        let sessionConfigJSON = try! sessionConfig.toJsonString()
        self.init(account: account, sessionConfigJSON: sessionConfigJSON, ownerPrivateKey: ownerPrivateKey, paymaster: paymaster)
    }
}

public func createSession(args: CreateSessionArgs, config: Config) async throws -> String {
    // Pretty print the sessionConfig JSON
    if let jsonData = args.inner.sessionConfig.data(using: .utf8) {
        do {
            let jsonObject = try JSONSerialization.jsonObject(with: jsonData)
            let prettyJsonData = try JSONSerialization.data(
                withJSONObject: jsonObject, options: [.prettyPrinted, .sortedKeys])
            if let prettyJsonString = String(data: prettyJsonData, encoding: .utf8) {
                print("📋 Session Config JSON:")
                print(prettyJsonString)
            }
        } catch {
            print("⚠️ Failed to pretty print sessionConfig JSON: \(error)")
            print("📋 Raw sessionConfig: \(args.inner.sessionConfig)")
        }
    } else {
        print("📋 Raw sessionConfig: \(args.inner.sessionConfig)")
    }

    let args = args.inner
    let createdSession = try await ZKsyncSSOFFI.createSession(args: args, config: config.inner)
    return createdSession.transactionReceiptJson
}
