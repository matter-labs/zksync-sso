import Foundation
import ZKsyncSSO
import ZKsyncSSOIntegration

extension SessionDemoActions {
    static func deployAccount() async throws -> DeployedAccountDetails {
        do {
            return try await ZKsyncSSOIntegration.deployAccount()
        } catch {
            print("❌ Error deploying account: \(error)")
            throw error
        }
    }
}
