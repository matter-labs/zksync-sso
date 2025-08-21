import Foundation
import ZKsyncSSOIntegration

extension SessionDemoActions {
    static func createAndRevokeSession(deployedAccount: DeployedAccountDetails) async throws {
        do {
            try await ZKsyncSSOIntegration.sessionCreateAndRevoke(
                deployedAccount: deployedAccount
            )
        } catch {
            print("❌ Error in create and revoke session: \(error)")
            throw error
        }
    }
}
