import Foundation
import ZKsyncSSO

public func deployAccountForSessionSend(
    uniqueAccountId: String = IntegrationConstants.sessionSendTestUniqueAccountId,
    sessionConfigJson: String = IntegrationConstants.createSessionConfigJson(
        sessionOwner: IntegrationConstants.sessionSendTestSessionOwner,
        expiresAt: "\(IntegrationConstants.sessionSendTestExpiresAt)",
        feeLimitLifetime: "100000000000000000",
        target: IntegrationConstants.sessionSendTestTransferTarget,
        maxValuePerUse: "10000000000000000"
    ),
    config: Config = .default,
    deploySigner: any Signer = IntegrationConstants.sessionSendTestDeploySigner,
) async throws -> DeployedAccountDetails {
    print("\n=== Deploying Account with Initial Session ===")
    
    var config = config
    
    let sessionContractAddress = config.contracts.session
    
    config.deployWallet = DeployWallet(privateKeyHex: deploySigner.privateKeyHex)
    
    let owner = IntegrationConstants.sessionSendTestOwner
    let ownerAddress = owner.address
    
    print("📋 Session config JSON:")
    print(sessionConfigJson)
    
    let sessionModule = SessionModuleArgs(
        location: sessionContractAddress,
        initialSessionConfigJson: sessionConfigJson
    )
    
    let deployArgs = DeployModularAccountParameters(
        installNoDataModules: [],
        owners: [ownerAddress],
        sessionModule: sessionModule,
        paymaster: nil,
        passkeyModule: nil,
        uniqueAccountId: uniqueAccountId
    )
    
    print("🔧 Deploying modular account...")
    let deployResult = try await deployModularAccount(
        params: deployArgs,
        config: config
    )
    let deployedAccountAddress = deployResult.address
    
    print("✅ Account deployed successfully!")
    print("  Address: \(deployedAccountAddress)")
    print("  Unique ID: \(deployResult.uniqueAccountId)")
    
    // Verify the deployed account address matches the expected one
    guard deployedAccountAddress == IntegrationConstants.sessionSendTestDeployedAccountAddress else {
        throw NSError(
            domain: "SessionSendTransactionError",
            code: 9,
            userInfo: [
                NSLocalizedDescriptionKey:
                    "Deployed account address should match expected value. Expected: \(IntegrationConstants.sessionSendTestDeployedAccountAddress), Got: \(deployedAccountAddress)"
            ]
        )
    }
    
    print("✅ Deployed account address matches expected value!")
    
    // Validate that the expected owner is indeed a K1 owner of the deployed account
    try await validateIsK1Owner(
        accountAddress: deployedAccountAddress,
        ownerAddress: IntegrationConstants.sessionSendTestOwnerAddress,
        config: config,
        description: "Owner verification after session send account deployment"
    )
    
    return DeployedAccountDetails(
        address: deployedAccountAddress,
        owner: owner,
        uniqueAccountId: uniqueAccountId,
        sessionConfigJson: sessionConfigJson,
        config: config
    )
}
