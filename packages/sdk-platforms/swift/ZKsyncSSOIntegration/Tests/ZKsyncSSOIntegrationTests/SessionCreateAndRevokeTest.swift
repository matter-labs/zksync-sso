import Testing
import Foundation
@testable import ZKsyncSSOIntegration

@Test(.disabled("Manual test only"))
func testSessionCreateAndRevoke() async throws {
    let deployedAccount = try await deployAccount()
    try await sessionCreateAndRevoke(deployedAccount: deployedAccount)
}
