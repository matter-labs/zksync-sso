import Testing
import Foundation
@testable import ZKsyncSSOIntegration

// MARK: - Combined Integration Test

@Test(.disabled("Manual test only"))
func testCompleteZKsyncSSOIntegration() async throws {
    print("\n" + String(repeating: "=", count: 100))
    print("🚀 RUNNING COMPLETE ZKSYNC SSO INTEGRATION TEST 🚀")
    print(String(repeating: "=", count: 100))
    
    // Step 1: Deploy Account
    print("\n📦 STEP 1: Deploy Account")
    let deployedAccount = try await deployAccount()
    
    // Step 2: Create and Revoke Session
    print("\n🔑 STEP 2: Create and Revoke Session")
    try await sessionCreateAndRevoke(deployedAccount: deployedAccount)
    
    // Step 3: Session Send Transaction (deploys its own account)
    print("\n💸 STEP 3: Session Send Transaction")
    try await sessionSendTransaction()
    
    print("\n" + String(repeating: "=", count: 100))
    print("✅ ALL INTEGRATION TESTS COMPLETED SUCCESSFULLY ✅")
    print(String(repeating: "=", count: 100))
    print("Integration Test Summary:")
    print("1. ✅ Deployed modular account with session module")
    print("2. ✅ Created and revoked session successfully")
    print("3. ✅ Deployed account with session and sent transaction")
    print("4. ✅ All session states verified correctly")
    print("5. ✅ All blockchain interactions successful")
}
