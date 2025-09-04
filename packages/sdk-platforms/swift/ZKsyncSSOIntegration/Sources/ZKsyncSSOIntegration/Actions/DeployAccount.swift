import Foundation
import ZKsyncSSO

public struct DeployedAccountDetails {

    public var ownerAddress: String {
        owner.address
    }

    public var ownerPrivateKeyHex: String {
        owner.privateKeyHex
    }

    public let sessionConfigJson: String?

    public let address: String
    public let owner: any Signer
    public let config: Config
    
    public let uniqueAccountId: String?

    public init(
        address: String,
        owner: any Signer,
        uniqueAccountId: String?,
        sessionConfigJson: String?,
        config: Config
    ) {
        self.address = address
        self.owner = owner
        self.uniqueAccountId = uniqueAccountId
        self.sessionConfigJson = sessionConfigJson
        self.config = config
    }
}

extension DeployedAccountDetails {
    public static var `default`: DeployedAccountDetails {
        DeployedAccountDetails(
            address: "ADDRESS",
            owner: RichWallet.one,
            uniqueAccountId: "uniqueAccountId",
            sessionConfigJson: "sessionConfigJson",
            config: .default
        )
    }
}

public func deployAccount(
    owner: any Signer = IntegrationConstants.accountOwner,
    uniqueAccountId: String = IntegrationConstants.randomSaltStr,
    paymaster: PaymasterParams? = nil,
    passkeyModule: PasskeyModuleArgs? = nil,
    initialSessionConfigJson: String? = IntegrationConstants.createSessionConfigJson(
        sessionOwner: IntegrationConstants.sessionOwner,
        expiresAt: "\(IntegrationConstants.expiresAt)",
        feeLimitLifetime: IntegrationConstants.feeLimitLifetime,
        target: IntegrationConstants.transferSessionTarget,
        maxValuePerUse: "\(IntegrationConstants.maxValuePerUse)"
    ),
    config: Config = .default,
    expectedDeployedAccountAddress: String = IntegrationConstants.deployedAccountAddress
) async throws -> DeployedAccountDetails {
    print("🚀 Starting account deployment...")

    let ownerAddress = owner.address

    let sessionModule: SessionModuleArgs? = {
        guard let initialSessionConfigJson else { return nil }

        print("📋 Session config JSON:")
        print(initialSessionConfigJson)

        return SessionModuleArgs(
            location: config.contracts.session,
            initialSessionConfigJson: initialSessionConfigJson
        )
    }()

    let deployArgs = DeployModularAccountParameters(
        installNoDataModules: [],
        owners: [ownerAddress],
        sessionModule: sessionModule,
        paymaster: paymaster,
        passkeyModule: passkeyModule,
        uniqueAccountId: uniqueAccountId
    )

    print("🔧 Deploying modular account...")
    let result = try await deployModularAccount(
        params: deployArgs,
        config: config
    )

    print("✅ Account deployed successfully!")
    print("  Address: \(result.address)")
    print("  Unique ID: \(result.uniqueAccountId)")

    // Validate deployed address matches expected
    guard result.address == expectedDeployedAccountAddress else {
        throw NSError(
            domain: "DeployAccountError",
            code: 1,
            userInfo: [
                NSLocalizedDescriptionKey:
                    "Deployed address does not match expected: \(expectedDeployedAccountAddress), got: \(result.address)"
            ]
        )
    }

    print("\n💰 Funding account with 1 ETH...")
    try await ZKsyncSSO.fundAccount(
        address: result.address,
        amount: "1.0",
        config: config
    )
    print("✅ Account funded successfully!")

    // Validate that the expected owner is indeed a K1 owner of the deployed account
    try await validateIsK1Owner(
        accountAddress: result.address,
        ownerAddress: ownerAddress,
        config: config,
        description: "Owner verification after account deployment"
    )

    return DeployedAccountDetails(
        address: result.address,
        owner: owner,
        uniqueAccountId: uniqueAccountId,
        sessionConfigJson: initialSessionConfigJson,
        config: config
    )
}
