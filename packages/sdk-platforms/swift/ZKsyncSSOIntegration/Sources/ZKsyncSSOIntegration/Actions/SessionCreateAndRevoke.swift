import Foundation

public func sessionCreateAndRevoke(
    deployedAccount: DeployedAccountDetails
) async throws {
    print("🔑 Starting session creation and revocation flow...")
    
    
    // Step 1: Create the session using the centralized createSession function
    print("\n--- Creating session ---")
    let sessionId = try await createSession(
        deployedAccount: deployedAccount
    )
    
    print("✅ Session created with ID: \(sessionId)")
    
    // Step 2: Revoke the session using the centralized revokeSession function
    print("\n--- Revoking session ---")
    try await revokeSession(
        deployedAccount: deployedAccount,
        sessionId: sessionId,
    )
    
    print("✅ Session create and revoke flow completed successfully!")
}
