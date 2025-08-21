import Foundation
import Testing
import ZKsyncSSO
@testable import ZKsyncSSOIntegration

@Test(.disabled("Manual test only"))
func testDeployAccount() async throws {
    _ = try await deployAccount()
}
