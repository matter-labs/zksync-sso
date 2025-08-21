import Foundation
import ZKsyncSSO

public func revokeSession(
    deployedAccount: DeployedAccountDetails,
    sessionId: String,
    sessionOwner: any Signer = IntegrationConstants.secondSessionOwner,
) async throws {
    let config = deployedAccount.config
    let accountAddress = deployedAccount.address
    let accountOwnerPrivateKeyHex = deployedAccount.ownerPrivateKeyHex

    print("🔒 Starting session revocation...")
    print("  Session ID: \(sessionId)")
    print("  Account: \(accountAddress)")

    let revokeArgs = RevokeSessionArgs(
        account: accountAddress,
        sessionId: sessionId,
        ownerPrivateKey: accountOwnerPrivateKeyHex
    )

    print("🔧 Attempting to revoke session...")

    let revokeResult = try await ZKsyncSSO.revokeSession(
        args: revokeArgs,
        config: config
    )

    print("✅ Session revocation successful!")
    print("  Session ID: \(sessionId)")
    print("  Account: \(accountAddress)")
    print("  Transaction receipt: \(revokeResult)")

    // Verify session state after revocation
    print("\n--- Verifying session state after revocation ---")
    
    let sessionConfigJson =
        IntegrationConstants
        .createSessionConfigJson(
            sessionOwner: sessionOwner,
            expiresAt: "\(IntegrationConstants.expiresAt)",
            feeLimitLifetime: IntegrationConstants.secondSessionFeeLimit,
            target: IntegrationConstants.vitalikAddress,
            maxValuePerUse: IntegrationConstants.secondSessionMaxValuePerUse,
        )

    try await validateSessionState(
        accountAddress: accountAddress,
        sessionConfig: sessionConfigJson,
        predicates: [
            sessionStateClosed()
        ],
        config: config,
        description: "Session state after revocation"
    )

    print("✅ Session successfully revoked and verified")
}
