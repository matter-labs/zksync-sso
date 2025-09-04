import Foundation
import ZKsyncSSO

/// Creates a session on the deployed account
/// Returns the session hash as a string
public func createSession(
    deployedAccount: DeployedAccountDetails,
    sessionOwner: any Signer = IntegrationConstants.secondSessionOwner
) async throws -> String {
    print("🔑 Starting session creation...")

    let accountAddress = deployedAccount.address
    let accountOwnerAddress = deployedAccount.owner.address
    let config = deployedAccount.config

    let paymaster = config.contracts.accountPaymaster

    let sessionConfigJson =
        IntegrationConstants
        .createSessionConfigJson(
            sessionOwner: sessionOwner,
            expiresAt: "\(IntegrationConstants.expiresAt)",
            feeLimitLifetime: IntegrationConstants.secondSessionFeeLimit,
            target: IntegrationConstants.vitalikAddress,
            maxValuePerUse: IntegrationConstants.secondSessionMaxValuePerUse,
        )

    print("📋 Session config JSON:")
    print(sessionConfigJson)

    let sessionHash = try ZKsyncSSO.getSessionHash(sessionConfigJson: sessionConfigJson)
    let sessionHashStr = "\(sessionHash)"

    print("🔍 Session hash to create: \(sessionHashStr)")

    let expectedHash = IntegrationConstants.expectedSecondSessionHash
    guard sessionHashStr == expectedHash else {
        throw NSError(
            domain: "CreateSessionError",
            code: 1,
            userInfo: [
                NSLocalizedDescriptionKey:
                    "Session hash does not match expected value. Expected: \(expectedHash), Got: \(sessionHashStr)"
            ]
        )
    }
    print("✅ Session hash matches expected value")

    // Validate that the owner is indeed a K1 owner before creating session
    try await validateIsK1Owner(
        accountAddress: accountAddress,
        ownerAddress: accountOwnerAddress,
        description: "K1 owner validation before session creation"
    )

    let createArgs = CreateSessionArgs(
        account: accountAddress,
        sessionConfigJSON: sessionConfigJson,
        ownerPrivateKey: sessionOwner.privateKeyHex,
        paymaster: paymaster
    )

    print("🔧 Attempting to create session...")
    print("  Account address: \(accountAddress)")
    print("  Owner address: \(accountOwnerAddress)")
    print("  Paymaster: \(paymaster)")

    let createResult = try await ZKsyncSSO.createSession(
        args: createArgs,
        config: config
    )

    print("✅ Session creation successful!")
    print("  Session ID: \(sessionHashStr)")
    print("  Account: \(accountAddress)")
    print("  Transaction receipt: \(createResult)")

    // Verify session state after creation using validation system
    let expectedFeeLimit = IntegrationConstants.secondSessionFeeLimit
    try await validateSessionState(
        accountAddress: accountAddress,
        sessionConfig: sessionConfigJson,
        predicates: [
            sessionStateActive(),
            sessionStateFeesRemainingEquals(expectedFeeLimit),
        ],
        description: "Session state after creation"
    )

    print("✅ Session successfully created and verified")

    return sessionHashStr
}
