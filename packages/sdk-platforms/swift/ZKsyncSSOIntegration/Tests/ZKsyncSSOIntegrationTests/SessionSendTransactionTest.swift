import Foundation
import Testing
@testable import ZKsyncSSOIntegration

@Test(.disabled("Manual test only"))
func testSessionSendTransaction() async throws {
    try await sessionSendTransaction()
}
