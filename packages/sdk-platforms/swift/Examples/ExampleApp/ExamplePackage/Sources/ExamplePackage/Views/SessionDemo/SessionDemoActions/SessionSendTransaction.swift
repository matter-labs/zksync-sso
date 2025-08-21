import Foundation
import ZKsyncSSOIntegration

extension SessionDemoActions {
    static func sessionSendTransaction() async throws {
        do {
            try await ZKsyncSSOIntegration.sessionSendTransaction()
        } catch {
            print("❌ Error in session send transaction: \(error)")
            throw error
        }
    }
}
